//! SCSI-over-USB transport for USB-attached Nikon scanners, via `nusb`
//!
//! Nikon's USB Coolscans do NOT use USB Mass Storage (no CBW/CSW wrappers). The
//! framing below is Nikon's own, not a standard: each SCSI command is a fixed
//! exchange over two bulk endpoints.
//!
//! ```text
//!   1. raw CDB              -> bulk-OUT   (its own transfer)
//!   2. 0xD0 phase query     -> bulk-OUT   (separate transfer, never coalesced)
//!   3. phase byte           <- bulk-IN    (0x01 none / 0x02 data-out / 0x03 data-in)
//!   4. data                 <-> per the phase byte
//!   5. 0x06 sense fetch     -> bulk-OUT
//!   6. 8-byte compact sense <- bulk-IN
//! ```
//!
//! Sense is fetched explicitly on every command; there is no autosense.
//!
//! `nusb`'s API is async, but [`Transport`] is synchronous, so this bridges with
//! `nusb`'s own blocking API (`MaybeFuture::wait` / `Endpoint::transfer_blocking`)
//! rather than pulling in an executor.

use super::{DataDirection, Error, SenseData, Transport};
use nusb::{
    Endpoint, Interface, MaybeFuture,
    transfer::{Buffer, Bulk, In, Out},
};
use std::{io, time::Duration};
use tracing::{debug, instrument};

/// Single-byte transport opcodes written to the bulk-OUT pipe
const OP_PHASE_QUERY: u8 = 0xD0;
const OP_SENSE_FETCH: u8 = 0x06;

/// Phase byte values the scanner returns to the phase query
const PHASE_NONE: u8 = 0x01;
const PHASE_DATA_OUT: u8 = 0x02;
const PHASE_DATA_IN: u8 = 0x03;

/// Per-transfer timeout
///
/// Generous on purpose. Several commands drive the mechanism and hold the pipe while they
/// do: a self-test and lamp warm-up run to about twelve seconds from cold, and an autofocus
/// takes ten or so. Timing one of those out aborts the command mid-move rather than stopping
/// the motor, so a timeout short enough to fire on a legitimate operation is worse than no
/// timeout at all.
const TIMEOUT: Duration = Duration::from_secs(180);

/// A SCSI device reachable over USB bulk endpoints
pub struct UsbTransport {
    ep_out: Endpoint<Bulk, Out>,
    ep_in: Endpoint<Bulk, In>,
    in_max_packet: usize,
    // The endpoints own no borrow, but the claimed interface must outlive them
    _interface: Interface,
}

impl UsbTransport {
    /// Open the first device matching `vendor_id:product_id`, claim interface 0,
    /// and grab its bulk endpoints. The ids belong to the scanner that answers to
    /// them, see [`ls50ed`](crate::scanners::ls50ed).
    ///
    /// `claim_interface`, not `detach_and_claim`: these are vendor-class devices
    /// with no kernel driver bound. A unit that turns out to be held by something
    /// else would need a detach plus a udev rule.
    pub fn open(vendor_id: u16, product_id: u16) -> io::Result<Self> {
        let info = nusb::list_devices()
            .wait()?
            .find(|d| d.vendor_id() == vendor_id && d.product_id() == product_id)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("no USB device {vendor_id:04x}:{product_id:04x}"),
                )
            })?;
        let device = info.open().wait()?;
        let interface = device.claim_interface(0).wait()?;
        let ep_out = interface.endpoint::<Bulk, Out>(0x01)?;
        let ep_in = interface.endpoint::<Bulk, In>(0x82)?;
        let in_max_packet = ep_in.max_packet_size().max(1);
        Ok(Self {
            ep_out,
            ep_in,
            in_max_packet,
            _interface: interface,
        })
    }

    fn write_out(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let mut buf = Buffer::new(bytes.len());
        buf.extend_from_slice(bytes);
        self.ep_out
            .transfer_blocking(buf, TIMEOUT)
            .into_result()
            .map_err(transfer_err)?;
        Ok(())
    }

    /// Read into `out` on the bulk-IN pipe, returning how many bytes the scanner
    /// actually sent
    ///
    /// A bulk-IN request must be a whole number of max-size packets; the scanner
    /// ends the transfer early with a short (or zero-length) packet, so the
    /// requested length only caps the read.
    fn read_in(&mut self, out: &mut [u8]) -> Result<usize, Error> {
        let req = out.len().max(1).div_ceil(self.in_max_packet) * self.in_max_packet;
        let buf = self
            .ep_in
            .transfer_blocking(Buffer::new(req), TIMEOUT)
            .into_result()
            .map_err(transfer_err)?;
        let n = buf.len().min(out.len());
        out[..n].copy_from_slice(&buf[..n]);
        Ok(buf.len())
    }
}

impl Transport for UsbTransport {
    /// Bulk transfers have no protocol ceiling, so this is a chunk size rather than a limit
    ///
    /// Stated only so a caller that chunks by it does not inherit the trait's default, which
    /// is the reserved buffer of a Linux sg device and means nothing here.
    fn max_transfer(&self) -> u32 {
        128 * 1024
    }

    #[instrument(skip_all, fields(cdb = ?cdb, ?direction, data_len = data.len()))]
    fn execute(
        &mut self,
        cdb: &[u8],
        direction: DataDirection,
        data: &mut [u8],
        sense: &mut [u8],
    ) -> Result<(), Error> {
        // Steps 1-2: CDB and phase query as two separate bulk-OUT transfers.
        // The scanner treats a coalesced blob as one oversized CDB.
        self.write_out(cdb)?;
        self.write_out(&[OP_PHASE_QUERY])?;

        // Step 3: one phase byte back
        let mut phase = [0u8; 1];
        if self.read_in(&mut phase)? == 0 {
            return Err(Error::InvalidResponse("empty phase response"));
        }
        let phase = phase[0];

        // Step 4: act on the phase the scanner reports, not the one we asked for.
        // On an error it may report PHASE_NONE for a data command and put the
        // reason in sense, so `direction` stays advisory, logged on a mismatch.
        let expected = match direction {
            DataDirection::None => PHASE_NONE,
            DataDirection::Read => PHASE_DATA_IN,
            DataDirection::Write => PHASE_DATA_OUT,
        };
        if phase != expected {
            debug!(phase, expected, "phase byte differs from command direction");
        }
        match phase {
            PHASE_DATA_IN => {
                let got = self.read_in(data)?;
                // Ordinary for a command whose allocation length overshoots the reply, which
                // is most of them. On an image read it means a lost line instead, but
                // `execute` has no residual channel to say which this was, and either way the
                // caller's buffer keeps the zeroed tail it arrived with.
                if got < data.len() {
                    debug!(
                        got,
                        want = data.len(),
                        "device sent less than was asked for"
                    );
                }
            }
            PHASE_DATA_OUT => {
                self.write_out(data)?;
            }
            PHASE_NONE => {}
            _ => return Err(Error::InvalidResponse("unexpected phase byte")),
        }

        // Steps 5-6: always fetch sense. What comes back is an 8-byte compact
        // buffer, not either standard SCSI layout, so parse it here.
        self.write_out(&[OP_SENSE_FETCH])?;
        let mut raw = [0u8; 8];
        let n = self.read_in(&mut raw)?;
        let raw = &raw[..n.min(raw.len())];
        let copy = raw.len().min(sense.len());
        sense[..copy].copy_from_slice(&raw[..copy]);

        match parse_compact_sense(raw) {
            None => Ok(()),
            Some(sense) => Err(Error::Status {
                status: 0x02,
                sense: Some(sense),
            }),
        }
    }
}

/// Map a `nusb` transfer error into this crate's SCSI error
fn transfer_err(e: nusb::transfer::TransferError) -> Error {
    Error::Transport(io::Error::other(e))
}

/// Parse the compact 8-byte sense buffer the sense fetch answers with
///
/// Layout, as captured off an LS-50:
///   `[0]` header (`0x00` = GOOD, non-zero = condition present),
///   `[1]` sense key (low nibble), `[2]` ASC, `[3]` ASCQ, `[4]` FRU.
/// Returns `None` when the scanner reports GOOD.
///
/// FRU (byte 4) is dropped: [`SenseData`] has nowhere to put it, and the
/// sub-states it distinguishes (motor busy, calibrating, generally becoming
/// ready) all collapse to the same readiness status here.
fn parse_compact_sense(raw: &[u8]) -> Option<SenseData> {
    if raw.first().copied().unwrap_or(0) == 0 || raw.len() < 4 {
        return None;
    }
    Some(SenseData {
        key: raw[1] & 0x0f,
        asc: raw[2],
        ascq: raw[3],
        // Compact vendor sense carries neither ILI nor deferred bits
        ili: false,
        deferred: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_sense_is_none() {
        assert_eq!(parse_compact_sense(&[0, 0, 0, 0, 0, 0, 0, 0]), None);
    }

    #[test]
    fn parses_field_positions() {
        // header, key, asc, ascq, fru: power-on reset 06/29/00
        let sd = parse_compact_sense(&[0x02, 0x06, 0x29, 0x00, 0x00, 0, 0, 0]).unwrap();
        assert_eq!((sd.key, sd.asc, sd.ascq), (0x06, 0x29, 0x00));
    }

    #[test]
    fn masks_sense_key_to_low_nibble() {
        let sd = parse_compact_sense(&[0x02, 0xF6, 0x29, 0x00]).unwrap();
        assert_eq!(sd.key, 0x06);
    }

    #[test]
    fn too_short_after_header_is_none() {
        assert_eq!(parse_compact_sense(&[0x02, 0x06]), None);
    }
}
