//! SCSI passthrough on Windows, via the scanner class driver
//!
//! The command block is Microsoft's `SCSISCAN_CMD` from `scsiscan.h`. It maps onto the Linux
//! `sg_io_hdr` in [`super::linux`] closely, with one trap: the transfer is `METHOD_OUT_DIRECT`,
//! so the data buffer is the *output* buffer in both directions and only the flags say which
//! way it goes.

use super::{DataDirection, Error, SenseData, Transport};
use std::{io, os::windows::ffi::OsStrExt, path::Path, ptr, thread::sleep, time::Duration};
use tracing::*;
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_WORKING_SET_QUOTA, HANDLE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    },
    System::IO::DeviceIoControl,
};

/// `FILE_DEVICE_SCANNER << 16 | function 4 << 2 | METHOD_OUT_DIRECT`
const IOCTL_SCSISCAN_CMD: u32 = 0x0019_0012;

/// Direction, as `SCSISCAN_CMD::srb_flags` carries it
const SRB_FLAGS_DATA_IN: u32 = 0x40;
const SRB_FLAGS_DATA_OUT: u32 = 0x80;
const SRB_FLAGS_NO_DATA: u32 = 0x00;

/// The status byte the driver writes back through `srb_status`
const SRB_STATUS_SUCCESS: u8 = 0x01;
const SRB_STATUS_ERROR: u8 = 0x04;
/// Set alongside a base status when the driver has filled the sense buffer
const SRB_STATUS_AUTOSENSE_VALID: u8 = 0x80;
/// The base status lives in the low six bits; the top two are flags
const SRB_STATUS_MASK: u8 = 0x3F;

/// How much sense to ask for
///
/// Not the caller's full buffer. The scanner needs exactly 32: bytes 18-31 carry Nikon state
/// codes past the end of standard fixed-format sense, and asking for the standard 18 silently
/// breaks its init state machine rather than failing outright.
const SENSE_LENGTH: u8 = 32;

/// Direct I/O locks the caller's buffer into the working set, and a large one can exceed the
/// process quota. Nikon Scan rides it out rather than failing, and so do we.
const QUOTA_RETRIES: usize = 200;
const QUOTA_RETRY_DELAY: Duration = Duration::from_millis(50);

/// What a single transfer is chunked at, matching Nikon Scan
///
/// The quota above scales with this, so raising it trades round trips for retries.
const MAX_TRANSFER: u32 = 128 * 1024;

/// Microsoft's `SCSISCAN_CMD`, from `scsiscan.h`
///
/// [`size`](Self::size) is the caller's own `size_of` rather than a literal, so this is the
/// 44 bytes a 32-bit build lays out and the 56 a 64-bit one does. The difference is four bytes
/// of padding after `cdb`, where `repr(C)` aligns the pointers that follow, and the driver
/// expects whichever matches the calling process.
#[repr(C)]
struct ScsiScanCmd {
    reserved1: u32,
    size: u32,
    srb_flags: u32,
    cdb_length: u8,
    sense_length: u8,
    reserved2: u8,
    reserved3: u8,
    transfer_length: u32,
    cdb: [u8; 16],
    srb_status: *mut u8,
    sense_buffer: *mut u8,
}

/// A scanner reachable through `scsiscan.sys`
pub struct ScsiScanDevice {
    handle: HANDLE,
}

// The handle is owned by this struct and every use goes through `&mut self`
unsafe impl Send for ScsiScanDevice {}

impl ScsiScanDevice {
    /// Open a scanner by device path, conventionally `\\.\Scanner0`
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // Synchronous on purpose: `execute` blocks on the command, and an overlapped handle
        // would leave us reading the status byte before the driver had written it
        // SAFETY: `wide` is NUL terminated and outlives the call, and the two null pointers
        // are documented as optional for security attributes and template file.
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null(),
                OPEN_EXISTING,
                0,
                ptr::null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        debug!(?path, "Opened scanner");
        Ok(Self { handle })
    }
}

impl Drop for ScsiScanDevice {
    fn drop(&mut self) {
        // SAFETY: `handle` came from a successful `CreateFileW` and nothing else closes it
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

impl Transport for ScsiScanDevice {
    fn max_transfer(&self) -> u32 {
        MAX_TRANSFER
    }

    #[instrument(skip_all, fields(cdb = ?cdb, ?direction, data_len = data.len()))]
    fn execute(
        &mut self,
        cdb: &[u8],
        direction: DataDirection,
        data: &mut [u8],
        sense: &mut [u8],
    ) -> Result<(), Error> {
        if cdb.len() > 16 {
            return Err(Error::Unsupported(
                "SCSISCAN_CMD carries at most a 16-byte CDB",
            ));
        }

        let mut srb_status = 0u8;
        let sense_length = SENSE_LENGTH.min(u8::try_from(sense.len()).unwrap_or(u8::MAX));

        let mut padded = [0u8; 16];
        padded[..cdb.len()].copy_from_slice(cdb);

        let mut cmd = ScsiScanCmd {
            reserved1: 0,
            size: size_of::<ScsiScanCmd>() as u32,
            srb_flags: match direction {
                DataDirection::None => SRB_FLAGS_NO_DATA,
                DataDirection::Read => SRB_FLAGS_DATA_IN,
                DataDirection::Write => SRB_FLAGS_DATA_OUT,
            },
            cdb_length: cdb.len() as u8,
            sense_length,
            reserved2: 0,
            reserved3: 0,
            transfer_length: data.len() as u32,
            cdb: padded,
            srb_status: &mut srb_status,
            sense_buffer: sense.as_mut_ptr(),
        };

        // There is no timeout field here, unlike `sg_io_hdr`. The driver's own default governs,
        // which is as well: a frame SET WINDOW walks the stage, and aborting one mid-move
        // grinds the mechanism until a power cycle rather than stopping the motor.
        let mut returned = 0u32;
        let mut attempt = 0;
        loop {
            // SAFETY: `handle` is open for the life of `self`. `cmd` is fully initialized on
            // this stack frame, and its `srb_status`/`sense_buffer` pointers borrow locals that
            // outlive the call. The data buffer is passed with its own length, so the driver
            // cannot map past it. `METHOD_OUT_DIRECT` means that buffer is the *output*
            // parameter whichever way the data actually flows.
            let ok = unsafe {
                DeviceIoControl(
                    self.handle,
                    IOCTL_SCSISCAN_CMD,
                    (&raw mut cmd).cast(),
                    size_of::<ScsiScanCmd>() as u32,
                    data.as_mut_ptr().cast(),
                    data.len() as u32,
                    &mut returned,
                    ptr::null_mut(),
                )
            };

            if ok != 0 {
                break;
            }

            // Direct I/O could not lock the buffer. Transient, and waiting is what clears it.
            let e = io::Error::last_os_error();
            if e.raw_os_error() != Some(ERROR_WORKING_SET_QUOTA as i32) || attempt >= QUOTA_RETRIES
            {
                return Err(e.into());
            }
            attempt += 1;
            debug!(attempt, "Working set quota exceeded, retrying");
            sleep(QUOTA_RETRY_DELAY);
        }

        // Written on every completion, success included, so worth surfacing either way
        debug!(
            srb_status = format!("0x{srb_status:02x}"),
            returned, attempt, "SCSISCAN_CMD completed"
        );

        let base = srb_status & SRB_STATUS_MASK;
        if base == SRB_STATUS_SUCCESS {
            return Ok(());
        }

        // A bus or driver level fault, as distinct from the target answering with a status
        if base != SRB_STATUS_ERROR {
            return Err(Error::HostAdapter {
                status: u16::from(srb_status),
            });
        }

        let sense = if srb_status & SRB_STATUS_AUTOSENSE_VALID != 0 {
            let len = usize::from(sense_length).min(sense.len());
            debug!(sense_raw = ?&sense[..len], "raw sense buffer");
            SenseData::parse(&sense[..len])
        } else {
            None
        };

        // CHECK CONDITION is not necessarily an error: the sense may describe a normal state,
        // and only the caller can tell. There is no status byte in this struct, and it is the
        // only status that produces autosense, so it is what we report.
        debug!(?sense, "SCSI command reported CHECK CONDITION");
        Err(Error::Status {
            status: 0x02,
            sense,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The driver reads this by offset, so a reordered or repacked field corrupts every
    /// command rather than failing loudly
    #[test]
    fn the_command_block_matches_the_driver_layout() {
        let cmd = ScsiScanCmd {
            reserved1: 0,
            size: 0,
            srb_flags: 0,
            cdb_length: 0,
            sense_length: 0,
            reserved2: 0,
            reserved3: 0,
            transfer_length: 0,
            cdb: [0; 16],
            srb_status: ptr::null_mut(),
            sense_buffer: ptr::null_mut(),
        };
        let base = &cmd as *const _ as usize;
        let offset = |field: *const u8| field as usize - base;

        assert_eq!(offset((&raw const cmd.reserved1).cast()), 0x00);
        assert_eq!(offset((&raw const cmd.size).cast()), 0x04);
        assert_eq!(offset((&raw const cmd.srb_flags).cast()), 0x08);
        assert_eq!(offset(&raw const cmd.cdb_length), 0x0C);
        assert_eq!(offset(&raw const cmd.sense_length), 0x0D);
        assert_eq!(offset((&raw const cmd.transfer_length).cast()), 0x10);
        assert_eq!(offset((&raw const cmd.cdb).cast()), 0x14);

        // 44 bytes with 32-bit pointers, which is what the captures show, and 56 with 64-bit
        // ones once `repr(C)` pads `cdb` out to their alignment
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(offset((&raw const cmd.srb_status).cast()), 0x24);
            assert_eq!(offset((&raw const cmd.sense_buffer).cast()), 0x28);
            assert_eq!(size_of::<ScsiScanCmd>(), 44);
        }
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(offset((&raw const cmd.srb_status).cast()), 0x28);
            assert_eq!(offset((&raw const cmd.sense_buffer).cast()), 0x30);
            assert_eq!(size_of::<ScsiScanCmd>(), 56);
        }
    }
}
