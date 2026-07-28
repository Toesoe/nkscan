//! Where the frames sit on the loaded film
//!
//! The roll feeder senses this itself and reports it as a transport table on DTC 0x8F: a short
//! header and one eight-byte record per addressable slot. Origins are read rather than
//! computed because the feed does not place frames on an even pitch, and a window the table
//! disagrees with reads into the gap instead of the frame.
//!
//! An adapter with no table falls back on [`FrameBoundaries::evenly_spaced`].

use super::{Ls5000ed, dtc::Dtc};
use crate::scanners::{ScanArea, nikon::capabilities::Capabilities};
use crate::scsi::{self, Transport};

/// Bytes of header before the first record
const TABLE_HEADER: usize = 4;
/// Bytes per record
const RECORD: usize = 8;

/// One sensed frame along the roll
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRecord {
    /// Where the frame starts along the feed, in native dots
    pub origin: u32,
    /// What the feeder calls this slot. Nikon's own records step by eight.
    pub selector: u16,
    /// Per-record flags. Uncharacterized; carried so a caller can see them.
    pub flags: u16,
}

impl FrameRecord {
    /// The window that scans just this frame
    pub fn scan_area(self, capabilities: Capabilities) -> ScanArea {
        super::geometry::whole_frame(self.origin, capabilities)
    }
}

/// The frames the scanner will address, in the order they sit on the film
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBoundaries(pub Vec<FrameRecord>);

impl FrameBoundaries {
    /// Decode a transport table as DTC 0x8F reports it, header included
    ///
    /// The header's third byte is the record count, checked against what arrived: a count
    /// reaching past the buffer would otherwise read whatever followed it as frame positions.
    pub fn parse(payload: &[u8]) -> Result<Self, scsi::Error> {
        let count = usize::from(*payload.get(2).ok_or(scsi::Error::InvalidResponse(
            "transport table shorter than its header",
        ))?);
        let need = TABLE_HEADER + count * RECORD;
        if payload.len() < need {
            return Err(scsi::Error::InvalidResponse(
                "transport table declares more frames than it carries",
            ));
        }
        Ok(Self(
            (0..count)
                .map(|i| {
                    let at = TABLE_HEADER + i * RECORD;
                    let be32 = |o: usize| {
                        u32::from_be_bytes([
                            payload[o],
                            payload[o + 1],
                            payload[o + 2],
                            payload[o + 3],
                        ])
                    };
                    let be16 = |o: usize| u16::from_be_bytes([payload[o], payload[o + 1]]);
                    FrameRecord {
                        origin: be32(at),
                        selector: be16(at + 4),
                        flags: be16(at + 6),
                    }
                })
                .collect(),
        ))
    }

    /// `count` frames one `pitch` apart, for an adapter that reports no table
    ///
    /// The selector is the frame's own index, since nothing is addressing slots here.
    pub fn evenly_spaced(count: u32, pitch: u32) -> Self {
        Self(
            (0..count)
                .map(|i| FrameRecord {
                    origin: i * pitch,
                    selector: i as u16,
                    flags: 0,
                })
                .collect(),
        )
    }

    /// Frames whose window fits inside the travel the adapter reports
    ///
    /// The feeder's table runs past the end of the film on a short roll, and a window starting
    /// there drives the transport into film that is not present.
    pub fn within(&self, capabilities: Capabilities, travel: u32) -> Self {
        Self(
            self.0
                .iter()
                .copied()
                .filter(|record| {
                    record
                        .origin
                        .checked_add(capabilities.boundary_y)
                        .is_some_and(|end| end <= travel)
                })
                .collect(),
        )
    }
}

impl<T> Ls5000ed<T>
where
    T: Transport,
{
    /// The transport table the feeder sensed for the loaded roll
    ///
    /// Framed, since a 40-slot roll answers with more than a short one.
    pub fn roll_table(&mut self) -> Result<FrameBoundaries, scsi::Error> {
        let payload = self.read_framed_dtc(Dtc::RollTable, None)?;
        FrameBoundaries::parse(&payload[super::dtc::HEADER_LEN as usize..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Origins a feeder reports for a six-frame strip
    ///
    /// Unevenly spaced on purpose, which is the whole reason the table is read rather than
    /// computed: the gaps here run from 5964 to 6216 dots against a 5959-dot frame.
    const ORIGINS: [u32; 6] = [840, 7056, 13104, 19068, 25172, 31206];

    /// A table in the wire layout, with the selectors stepping by eight the way a feeder's do
    fn table(origins: &[u32]) -> Vec<u8> {
        let mut bytes = vec![0x01, 0x2A, origins.len() as u8, 0x00];
        for (index, &origin) in origins.iter().enumerate() {
            bytes.extend_from_slice(&origin.to_be_bytes());
            bytes.extend_from_slice(&(1 + 8 * index as u16).to_be_bytes());
            bytes.extend_from_slice(&0u16.to_be_bytes());
        }
        bytes
    }

    fn captured() -> Vec<u8> {
        table(&ORIGINS)
    }

    #[test]
    fn parses_a_transport_table() {
        let parsed = FrameBoundaries::parse(&captured()).unwrap();
        assert_eq!(parsed.0.len(), ORIGINS.len());
        assert_eq!(
            parsed.0.iter().map(|r| r.origin).collect::<Vec<_>>(),
            ORIGINS
        );
        assert_eq!(
            parsed.0.iter().map(|r| r.selector).collect::<Vec<_>>(),
            [1, 9, 17, 25, 33, 41]
        );
    }

    /// A full 40-slot roll, which is the largest table the feeder reports
    #[test]
    fn parses_a_whole_rolls_worth_of_records() {
        let origins: Vec<u32> = (0..40).map(|i| 840 + i * 6100).collect();
        let parsed = FrameBoundaries::parse(&table(&origins)).unwrap();
        assert_eq!(parsed.0.len(), 40);
        assert_eq!(parsed.0.last().unwrap().origin, *origins.last().unwrap());
    }

    /// The window takes its origin from the record, so the table and the pass cannot disagree
    /// about where a frame is
    #[test]
    fn a_record_places_the_window_on_its_own_origin() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let parsed = FrameBoundaries::parse(&captured()).unwrap();
        let area = parsed.0[5].scan_area(capabilities);
        assert_eq!(area.y_pos, ORIGINS[5]);
        assert_eq!(area.x_pos, 0);
        assert_eq!((area.x_size, area.y_size), (3946, 5959));
    }

    /// A count reaching past the buffer would otherwise read past the end of the response as
    /// frame positions, and drive the transport to wherever those bytes happened to say
    #[test]
    fn a_table_declaring_more_than_it_carries_is_refused() {
        let mut short = captured();
        short.truncate(TABLE_HEADER + 3 * RECORD);
        assert!(matches!(
            FrameBoundaries::parse(&short),
            Err(scsi::Error::InvalidResponse(_))
        ));
        assert!(matches!(
            FrameBoundaries::parse(&[]),
            Err(scsi::Error::InvalidResponse(_))
        ));
    }

    #[test]
    fn an_empty_table_is_not_an_error() {
        let table = FrameBoundaries::parse(&[0x01, 0x2A, 0x00, 0x00]).unwrap();
        assert!(table.0.is_empty());
    }

    /// A frame whose window would run off the end of the roll is dropped rather than scanned
    #[test]
    fn frames_past_the_reported_travel_are_dropped() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let table = FrameBoundaries::parse(&captured()).unwrap();
        // Room for the first three records' windows and no more
        let travel = ORIGINS[2] + capabilities.boundary_y;
        let kept = table.within(capabilities, travel);
        assert_eq!(kept.0.len(), 3);
        assert_eq!(kept.0.last().unwrap().origin, ORIGINS[2]);
    }

    #[test]
    fn evenly_spaced_places_frames_on_the_pitch() {
        let table = FrameBoundaries::evenly_spaced(3, 5959);
        assert_eq!(
            table.0.iter().map(|r| r.origin).collect::<Vec<_>>(),
            [0, 5959, 11918]
        );
    }
}
