//! The self-describing vendor data-structure read
//!
//! Every vendor structure on the models that use this comes back behind a fixed six-byte
//! header whose last two bytes are the payload length, so a caller reads twice: once for the
//! header, once for the whole thing. The alternative is guessing an allocation length, and a
//! structure that grew past the guess would be silently truncated rather than short.
//!
//! Which codes and qualifiers exist is per model, so only the framing lives here.

use crate::scsi::{
    self, Transport, TransportExt,
    cdbs::{DataTypeCode, Read},
};

/// The framing header every read below is prefixed with
pub const HEADER_LEN: u32 = 6;

/// Read one whole framed vendor structure, header included
///
/// `probe` is how much to ask for on the first read. It only has to reach the length field at
/// bytes 4..6; anything longer is a caller's choice and costs nothing but the transfer.
///
/// The returned buffer still carries the header, since callers pin captured bytes against it.
pub fn read_framed<T: Transport + ?Sized>(
    transport: &mut T,
    dtc: DataTypeCode,
    dtq: u16,
    probe: u32,
    control: u8,
) -> Result<Vec<u8>, scsi::Error> {
    let header = transport.send(&Read::new(0, dtc, dtq, probe, control))?;
    let length = header
        .get(4..6)
        .map(|l| u16::from_be_bytes([l[0], l[1]]))
        .ok_or(scsi::Error::InvalidResponse(
            "vendor DTC read shorter than its 6-byte header",
        ))?;
    transport.send(&Read::new(
        0,
        dtc,
        dtq,
        HEADER_LEN + u32::from(length),
        control,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scsi::{DataDirection, Error, mock::MockTransport};

    /// Answers the length in the header, then serves that many bytes
    struct Framed {
        payload: Vec<u8>,
        asked: Vec<u32>,
    }

    impl Transport for Framed {
        fn execute(
            &mut self,
            cdb: &[u8],
            _direction: DataDirection,
            data: &mut [u8],
            _sense: &mut [u8],
        ) -> Result<(), Error> {
            let want = u32::from_be_bytes([0, cdb[6], cdb[7], cdb[8]]);
            self.asked.push(want);
            data.fill(0);
            data[0] = cdb[2];
            data[4..6].copy_from_slice(&(self.payload.len() as u16).to_be_bytes());
            let body = HEADER_LEN as usize;
            let n = self.payload.len().min(data.len().saturating_sub(body));
            data[body..body + n].copy_from_slice(&self.payload[..n]);
            Ok(())
        }
    }

    /// The captured LS-5000 exchange: probe 6 bytes, learn 0x1b, re-read 33
    #[test]
    fn probes_the_header_then_reads_the_whole_structure() {
        let mut transport = Framed {
            payload: vec![0xAB; 27],
            asked: Vec::new(),
        };
        let out = read_framed(
            &mut transport,
            DataTypeCode::Vendor(0x87),
            0x0000,
            HEADER_LEN,
            0x80,
        )
        .unwrap();

        assert_eq!(transport.asked, [6, 33]);
        assert_eq!(out.len(), 33);
        assert_eq!(&out[6..], &[0xAB; 27]);
    }

    /// A probe longer than the header is legal and must not change what comes back
    #[test]
    fn a_generous_probe_reads_the_same_structure() {
        let mut transport = Framed {
            payload: vec![0x11; 4],
            asked: Vec::new(),
        };
        let out = read_framed(&mut transport, DataTypeCode::Vendor(0x8C), 0x0103, 10, 0x80).unwrap();
        assert_eq!(transport.asked, [10, 10]);
        assert_eq!(out.len(), 10);
    }

    /// A device answering with less than the header has nothing to take a length from, and
    /// treating a truncated reply as a zero-length structure would report success
    #[test]
    fn a_response_too_short_to_carry_a_length_is_an_error() {
        let mut transport = MockTransport::new();
        assert!(matches!(
            read_framed(
                &mut transport,
                DataTypeCode::Vendor(0x87),
                0x0000,
                4,
                0x80
            ),
            Err(scsi::Error::InvalidResponse(_))
        ));
    }
}
