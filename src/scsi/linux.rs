//! Safe-rust wrapper over the "SCSI-Generic" linux userspace layer

use super::{DataDirection, Error, SenseData, Transport};
use bitflags::bitflags;
use nix::ioctl_readwrite_bad;
use std::{
    fmt,
    fs::{File, OpenOptions},
    io,
    os::{fd::AsRawFd, raw::c_void},
    path::Path,
};
use tracing::{debug, instrument};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct SgIoHdr {
    /// [input] Interface ID: Must be 'S'
    interface_id: i32,
    /// [input] Data Trasfer Direction
    dxfer_direction: Direction,
    /// [input] The length of bytes of the SCSI command in `cmdp`
    cmd_len: u8,
    /// [input] Max size that can be written back to the `sbp` pointer
    mx_sb_len: u8,
    /// [input] Number of scatter/gather elements in an array pointed to by `dxferp`. 0 implies s/g is not used and `dxferp` points to the data transfer buffer
    iovec_count: u16,
    /// [input] number of bytes to be moved in the data transfer associated with this command
    dxfer_len: u32,
    /// [input/output]  data transfer memory or scatter gather list
    dxferp: *mut c_void,
    /// [input] the SCSI command to execute. must be `cmd_bytes` long. This memory is read-only.
    cmdp: *mut u8,
    /// [output] sense buffer memory (SCSI error information) of at most `mx_sb_len` bytes long
    sbp: *mut u8,
    /// [input] timeout in milliseconds. u32::MAX for no timeout
    timeout: u32,
    /// [input] SCSI flags
    flags: Flags,
    /// [input] user-provided command id that will be present in the response to help matching requests in a queue
    pack_id: i32,
    /// [input] user-provided pointer to something that you might need in the response (to hold some state information)
    usr_ptr: *mut c_void,
    /// [output] SCSI-standard status byte. Bits 0,6,and 7 can contain vendor information
    status: u8,
    /// [output] `status`, except (status & 0x3e) >> 1). So, stripped of vendor info to match Linux status code
    ///
    /// Kept as a raw byte (not `MaskedStatus`) because the kernel writes this value directly;
    /// an enum here would be UB the moment the driver reports a discriminant we didn't list.
    masked_status: u8,
    /// [output] "messaging level". Most modern chipsets hide this and will return zero.
    msg_status: u8,
    /// [output] The actual number of bytes written to the `sbp`. Will always be <= `mx_sb_len`
    sb_len_wr: u8,
    /// [output] errors from the host adapter, raw for the same reason as `masked_status`
    host_status: u16,
    /// [output] errors from the software driver (needs better formatting here because it's fixed fields and flags)
    driver_status: u16,
    /// [output] Data transfer length residual, dxfer_len - number of bytes actually transfered. Only reports underruns. Apparently some adapters report an incorrect number so you shouldn't trust this by default.
    resid: i32,
    /// [output] duration in milliseconds from the SCSI command being sent until when sg was informed it completed
    duration: u32,
    /// [output] A bunch of flags for useful info
    info: Info,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
// `ToFromDev`/`Unknown` are real kernel-defined values, kept for
// completeness, but we only ever construct this from our own
// `DataDirection`, which never asks for either.
#[allow(dead_code)]
enum Direction {
    /// SCSI Test Unit Ready, or similar commands where there is no data transfer associated with it
    None = -1,
    /// WRITE, user memory to device
    ToDev = -2,
    /// READ, device to user memory
    FromDev = -3,
    /// READ except during indirect io the user buffer is copied to the kernel buffers before transfer
    ToFromDev = -4,
    /// Unknown data direction (probably unused)
    Unknown = -5,
}

bitflags! {
    /// SCSI Flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Flags: u32 {
        const DIRECT_IO = 1;
        const UNUSED_LUN_INHIBIT =2;
        const MMAP_IO = 4;
        /// For testing bus speed
        const NO_DXFER = 0x10000;
        /// Q_AT_HEAD for this driver, Q_AT_TAIL for block devices
        const Q_AT_TAIL = 0x10;
        const Q_AT_HEAD = 0x20;
    }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Original Linux SCSI status code
/// NOTE: Not the same as the SCSI standard code d.t. vendor bytes
enum MaskedStatus {
    Good = 0x00,
    CheckCondition = 0x01,
    ConditionGood = 0x02,
    Busy = 0x04,
    IntermediateGood = 0x08,
    IntermediateCGood = 0x0A,
    ReservationConflict = 0x0C,
    CommandTerminated = 0x11,
    QueueFull = 0x14,
    AcaActive = 0x18,
    TaskAborted = 0x20,
}

impl TryFrom<u8> for MaskedStatus {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        Ok(match value {
            0x00 => Self::Good,
            0x01 => Self::CheckCondition,
            0x02 => Self::ConditionGood,
            0x04 => Self::Busy,
            0x08 => Self::IntermediateGood,
            0x0A => Self::IntermediateCGood,
            0x0C => Self::ReservationConflict,
            0x11 => Self::CommandTerminated,
            0x14 => Self::QueueFull,
            0x18 => Self::AcaActive,
            0x20 => Self::TaskAborted,
            other => return Err(other),
        })
    }
}

#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum HostStatus {
    /// No Error
    Ok = 0x00,
    /// Couldn't connect before timeout period
    NoConnect = 0x01,
    /// Bus stayed busy through time out period
    BusBusy = 0x02,
    /// Timed out for other reason
    Timeout = 0x03,
    /// Bad target, device may not be responding
    BadTarget = 0x04,
    /// Told to abort for some other reason
    Abort = 0x05,
    /// Parity error
    Parity = 0x06,
    /// Internal error detected in the host adapter
    Error = 0x07,
    /// The SCSI bus or the device has been reset
    Reset = 0x08,
    /// Got an interrupt we weren't expecting
    BadIntr = 0x09,
    /// Force command past mid-layer
    Passthrough = 0x0A,
    /// The low-level driver wants a retry
    SoftError = 0x0B,
}

impl TryFrom<u16> for HostStatus {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, u16> {
        Ok(match value {
            0x00 => Self::Ok,
            0x01 => Self::NoConnect,
            0x02 => Self::BusBusy,
            0x03 => Self::Timeout,
            0x04 => Self::BadTarget,
            0x05 => Self::Abort,
            0x06 => Self::Parity,
            0x07 => Self::Error,
            0x08 => Self::Reset,
            0x09 => Self::BadIntr,
            0x0A => Self::Passthrough,
            0x0B => Self::SoftError,
            other => return Err(other),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Info(u32);

impl Info {
    pub const OK: u32 = 0x0;
    pub const CHECK: u32 = 0x1;
    pub const INDIRECT_IO: u32 = 0x0;
    pub const DIRECT_IO: u32 = 0x2;
    pub const MIXED_IO: u32 = 0x4;

    pub const fn check_status(self) -> u32 {
        self.0 & 0x1
    }

    pub const fn io_type(self) -> u32 {
        self.0 & 0x6
    }
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let check = match self.check_status() {
            Self::CHECK => "CHECK",
            _ => "OK",
        };
        let io = match self.io_type() {
            Self::DIRECT_IO => "DIRECT_IO",
            Self::MIXED_IO => "MIXED_IO",
            _ => "INDIRECT_IO",
        };
        write!(f, "{check} | {io}")
    }
}

impl fmt::Debug for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// The SG_IO request code, from `<scsi/sg.h>`. This predates the `_IOC` encoding
/// convention, so it's a fixed literal rather than something built from
/// direction/size bits - hence the "bad" ioctl flavor below.
const SG_IO: u16 = 0x2285;

ioctl_readwrite_bad!(sg_io, SG_IO, SgIoHdr);

/// Default command timeout, in milliseconds.
const DEFAULT_TIMEOUT_MS: u32 = 20_000;

// ---- High level interface

pub struct SgDevice(File);

impl SgDevice {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        Ok(Self(file))
    }
}

impl Transport for SgDevice {
    #[instrument(skip_all, fields(cdb = ?cdb, ?direction, data_len = data.len()))]
    fn execute(
        &mut self,
        cdb: &[u8],
        direction: DataDirection,
        data: &mut [u8],
        sense: &mut [u8],
    ) -> Result<(), Error> {
        let mut cmd = cdb.to_vec();

        let mut hdr = SgIoHdr {
            interface_id: b'S' as i32,
            dxfer_direction: match direction {
                DataDirection::None => Direction::None,
                DataDirection::Read => Direction::FromDev,
                DataDirection::Write => Direction::ToDev,
            },
            cmd_len: cmd.len() as u8,
            mx_sb_len: sense.len() as u8,
            iovec_count: 0,
            dxfer_len: data.len() as u32,
            dxferp: data.as_mut_ptr() as *mut c_void,
            cmdp: cmd.as_mut_ptr(),
            sbp: sense.as_mut_ptr(),
            timeout: DEFAULT_TIMEOUT_MS,
            flags: Flags::empty(),
            pack_id: 0,
            usr_ptr: std::ptr::null_mut(),
            status: 0,
            masked_status: 0,
            msg_status: 0,
            sb_len_wr: 0,
            host_status: 0,
            driver_status: 0,
            resid: 0,
            duration: 0,
            info: Info(0),
        };

        // SAFETY: `self.0` is an open file, so `as_raw_fd()` gives a valid fd for the
        // duration of this call. `hdr` is a fully-initialized `SgIoHdr` living on this
        // stack frame. Its `cmdp`/`sbp`/`dxferp` pointers come from `cmd`, `sense`, and
        // `data`, all of which outlive the call (they're not touched again until after
        // it returns), and `cmd_len`/`mx_sb_len`/`dxfer_len` are set from those same
        // buffers' actual lengths, so the kernel can't read or write past them.
        unsafe { sg_io(self.0.as_raw_fd(), &mut hdr) }.map_err(io::Error::from)?;

        // These decode fields the kernel/adapter fill in on every completion, not
        // just failures - worth surfacing even when `info` says the command was
        // fine, since a host/driver-level hiccup doesn't always trip that flag.
        debug!(
            masked_status = ?MaskedStatus::try_from(hdr.masked_status),
            host_status = ?HostStatus::try_from(hdr.host_status),
            driver_status = hdr.driver_status,
            duration_ms = hdr.duration,
            "SG_IO completed"
        );

        // A successful ioctl only means the request was submitted; the command
        // itself may still have failed (see `info` and the sense buffer).
        if hdr.info.check_status() != Info::CHECK {
            return Ok(());
        }

        // `sb_len_wr` is kernel-written output; the driver contract says it's always
        // <= mx_sb_len, but we don't trust that when indexing our caller's buffer with it.
        let sb_len_wr = (hdr.sb_len_wr as usize).min(sense.len());
        // TODO: Remove. Raw sense bytes include the sense-key-specific field
        // pointer (fixed format bytes 15-17) that `SenseData::parse` doesn't
        // expose yet, which is what actually pinpoints the invalid CDB byte.
        debug!(sense_raw = ?&sense[..sb_len_wr], "raw sense buffer");
        let sense = SenseData::parse(&sense[..sb_len_wr]);

        // CHECK CONDITION alone doesn't mean an error occurred; the sense data
        // may describe a normal, expected state (e.g. "not ready, warming
        // up"). Only the caller, which knows how to interpret sense data for
        // the specific device, can tell the two apart, so this stays at
        // debug rather than warn.
        debug!(
            status = format!("0x{:02x}", hdr.status),
            ?sense,
            "SCSI command reported CHECK CONDITION"
        );

        Err(Error::Status {
            status: hdr.status,
            sense,
        })
    }
}
