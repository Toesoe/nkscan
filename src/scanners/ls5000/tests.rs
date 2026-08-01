//! Sequence tests against a scripted transport
//!
//! The wire formats are pinned in each module's own captured-byte tests; what these cover is
//! the command *order*, which is the part a mock can catch and a byte comparison cannot.

use super::*;
use crate::scanners::nikon::metering::Metering;
use crate::scanners::{
    ScanArea,
    ls5000::geometry::{Dpi, Samples},
};
use crate::scsi::{DataDirection, Error, SenseData, mock::MockTransport};
use boundaries::FrameBoundaries;

/// Samples as their big-endian wire bytes, the way the scanner sends them
fn be_line(samples: &[u16]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_be_bytes()).collect()
}

/// The captured page 0xC1 when that is what was asked for, zeroes otherwise
fn serve_inquiry(cdb: &[u8], data: &mut [u8]) {
    data.fill(0);
    if cdb[1] & 1 == 1 && cdb[2] == 0xC1 {
        let raw = capabilities::fixture::raw_page();
        let n = raw.len().min(data.len());
        data[..n].copy_from_slice(&raw[..n]);
    }
}

/// Small enough that a whole frame is two lines of one pixel
const TINY: ScanArea = ScanArea {
    x_pos: 0,
    y_pos: 0,
    x_size: 4,
    y_size: 8,
};

fn settings(ir: bool, samples: u8) -> ScanSettings {
    ScanSettings {
        resolution: Dpi::_1000.to_dpi(),
        ir,
        samples: Samples::new(samples).unwrap(),
        window: TINY,
        capabilities: capabilities::fixture::capabilities(),
    }
}

/// Answers the capability read that opening starts with
fn mock() -> MockTransport {
    MockTransport::new().with_page(0xC1, capabilities::fixture::raw_page())
}

/// The stream the scanner would send: each line's real samples, zero-padded to `stride`
fn padded(lines: &[Vec<u8>], stride: usize) -> Vec<u8> {
    let mut out = vec![0u8; lines.len() * stride];
    for (i, line) in lines.iter().enumerate() {
        out[i * stride..i * stride + line.len()].copy_from_slice(line);
    }
    out
}

/// Scripts a whole pass, tracking the order the driver drives it in
struct ScanMock {
    image: Vec<u8>,
    cursor: usize,
    /// Caps a single transfer, so a pass takes more than one read
    max_transfer: u32,
    /// Window ids in the order they were armed
    windows_set: Vec<u8>,
    /// Vendor tails of the windows armed, in the same order
    tails: Vec<Vec<u8>>,
    /// Whether the scan parameters were read since the last SCAN attempt
    read_scan_parameters: bool,
    /// How many SCAN attempts to refuse before accepting one
    refuse_scans: usize,
    /// Window ids each accepted SCAN carried
    scans: Vec<Vec<u8>>,
    /// The control byte of the last SCAN, which the firmware is particular about
    scan_control: Option<u8>,
    /// SCAN was issued without the scan parameters having been read first
    scanned_ungated: bool,
    /// Image reads the driver asked for, in bytes
    image_reads: Vec<u32>,
    /// The last autofocus payload seen
    autofocus_payload: Option<Vec<u8>>,
    /// What a channel-selected GET WINDOW reports back as measured
    measured_exposure: Option<[u32; 4]>,
}

impl ScanMock {
    fn new(image: Vec<u8>, max_transfer: u32) -> Self {
        Self {
            image,
            cursor: 0,
            max_transfer,
            windows_set: Vec::new(),
            tails: Vec::new(),
            read_scan_parameters: false,
            refuse_scans: 0,
            scans: Vec::new(),
            scan_control: None,
            scanned_ungated: false,
            image_reads: Vec::new(),
            autofocus_payload: None,
            measured_exposure: None,
        }
    }

    fn refusing(mut self, attempts: usize) -> Self {
        self.refuse_scans = attempts;
        self
    }
}

impl Transport for ScanMock {
    fn max_transfer(&self) -> u32 {
        self.max_transfer
    }

    fn execute(
        &mut self,
        cdb: &[u8],
        _direction: DataDirection,
        data: &mut [u8],
        _sense: &mut [u8],
    ) -> Result<(), Error> {
        let busy = |key, asc, ascq| {
            Err(Error::Status {
                status: 0x02,
                sense: Some(SenseData {
                    key,
                    asc,
                    ascq,
                    ili: false,
                    deferred: false,
                }),
            })
        };

        match cdb[0] {
            // TEST UNIT READY / RESERVE / RELEASE / SEND DIAGNOSTIC / MODE SELECT / trigger
            0x00 | 0x16 | 0x17 | 0x1D | 0x15 | 0xC1 => Ok(()),
            // Vendor read: focus
            0xE1 => Ok(()),
            // Vendor write: keep the autofocus payload for inspection
            0xE0 => {
                if cdb[2] == 0xA0 {
                    self.autofocus_payload = Some(data.to_vec());
                }
                Ok(())
            }
            0x12 => {
                serve_inquiry(cdb, data);
                Ok(())
            }
            0x24 => {
                let descriptor = &data[8..];
                self.windows_set.push(descriptor[0]);
                self.tails.push(descriptor[40..50].to_vec());
                Ok(())
            }
            0x1B => {
                if self.refuse_scans > 0 {
                    self.refuse_scans -= 1;
                    self.read_scan_parameters = false;
                    return busy(0x09, 0x80, 0x01);
                }
                if !self.read_scan_parameters {
                    self.scanned_ungated = true;
                }
                self.scans.push(data.to_vec());
                self.scan_control = Some(cdb[5]);
                self.cursor = 0;
                self.windows_set.clear();
                Ok(())
            }
            // An 8-byte header plus one 50-byte descriptor
            0x25 => {
                data.fill(0);
                if data.len() >= 58 {
                    data[6..8].copy_from_slice(&50u16.to_be_bytes());
                    if cdb[1] & 1 == 1 {
                        data[8] = cdb[5];
                        if let Some(measured) = self.measured_exposure {
                            let value = match cdb[5] {
                                1..=3 => measured[cdb[5] as usize - 1],
                                9 => measured[3],
                                _ => 0,
                            };
                            data[54..58].copy_from_slice(&value.to_be_bytes());
                        }
                    }
                }
                Ok(())
            }
            0x28 => match cdb[2] {
                // Framed vendor reads: a 6-byte header carrying the payload length
                0x87 => {
                    self.read_scan_parameters = true;
                    data.fill(0);
                    data[0] = 0x87;
                    data[4..6].copy_from_slice(&3u16.to_be_bytes());
                    Ok(())
                }
                // The image stream, one line a call
                0x00 => {
                    assert_eq!(
                        u16::from_be_bytes([cdb[4], cdb[5]]),
                        0x0001,
                        "image read used the wrong data-type qualifier"
                    );
                    assert_eq!(cdb[9], VENDOR_CONTROL, "image read lost the control byte");
                    let want = u32::from_be_bytes([0, cdb[6], cdb[7], cdb[8]]);
                    self.image_reads.push(want);
                    // Out of film: the scanner ends the transfer rather than short-reading,
                    // since a data phase is always the length the CDB asked for
                    if self.cursor >= self.image.len() {
                        return busy(0x0B, 0x3E, 0x00);
                    }
                    let end = (self.cursor + data.len()).min(self.image.len());
                    let n = end - self.cursor;
                    data[..n].copy_from_slice(&self.image[self.cursor..end]);
                    self.cursor = end;
                    Ok(())
                }
                other => panic!("unexpected data-type code {other:#04x}"),
            },
            other => panic!("unexpected opcode {other:#04x}"),
        }
    }
}

/// The 1x2 frame [`TINY`] asks for: two lines of one RGB pixel, each plane padded to two
/// samples, so R at 0, G at 2, B at 4, over a 512-byte line stride
fn rgb_image() -> Vec<u8> {
    padded(
        &[be_line(&[1, 0, 1, 0, 1]), be_line(&[2, 0, 2, 0, 2])],
        STRIDE,
    )
}

/// What `TINY` computes to at `Dpi::_1000`: three planes of two samples, padded to 512
const STRIDE: usize = 512;

fn scan_one(scanner: &mut Ls5000<ScanMock>, settings: &ScanSettings) -> Image {
    scanner.warm_up().unwrap();
    scanner
        .scan_image(settings, calibration::DEFAULT_GAIN)
        .unwrap()
}

#[test]
fn scan_decodes_an_rgb_frame() {
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();
    let frame = scan_one(&mut scanner, &settings(false, 1));
    assert_eq!(frame.rgb.dimensions(), (1, 2));
    assert!(frame.ir.is_none());
    assert_eq!(frame.rgb.get_pixel(0, 0).0, [1u16, 1, 1]);
    assert_eq!(frame.rgb.get_pixel(0, 1).0, [2u16, 2, 2]);
}

#[test]
fn scan_decodes_an_rgbi_frame() {
    // Four planes a line, in the order the driver arms them
    let image = padded(
        &[
            be_line(&[1, 0, 1, 0, 1, 0, 90]),
            be_line(&[2, 0, 2, 0, 2, 0, 91]),
        ],
        STRIDE,
    );
    let mut scanner = Ls5000::new(ScanMock::new(image, STRIDE as u32)).unwrap();
    let frame = scan_one(&mut scanner, &settings(true, 1));
    assert_eq!(frame.rgb.get_pixel(0, 0).0, [1u16, 1, 1]);
    let ir = frame.ir.expect("IR plane captured");
    assert_eq!(ir.dimensions(), (1, 2));
    assert_eq!(ir.get_pixel(0, 0).0, [90u16]);
}

/// SCAN is refused until the scan parameters have been read, so the read has to be inside the
/// retry rather than before it. Arming alone does not clear the gate.
#[test]
fn scan_reads_the_scan_parameters_before_it_is_accepted() {
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32).refusing(3)).unwrap();
    scan_one(&mut scanner, &settings(false, 1));
    assert!(
        !scanner.transport.scanned_ungated,
        "a SCAN went out without the scan parameters having been read"
    );
    assert_eq!(scanner.transport.scans.len(), 1);
}

/// SCAN goes out with a zero control byte, not the vendor bit
///
/// This driver set bit 7 on everything, which is the one thing it did that neither model anyone
/// had run did: both use 0x00 here while setting it on SET WINDOW. An LS-5000 with an SA-21
/// refuses the pass with 0x80 and takes it with 0x00. Pinned because nothing pinned it before,
/// which is how the guess survived to reach hardware.
#[test]
fn scan_carries_no_vendor_bit_in_its_control_byte() {
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();
    scan_one(&mut scanner, &settings(false, 1));
    assert_eq!(scanner.transport.scan_control, Some(0x00));
}

/// A scanner that never opens the gate is an error rather than an infinite retry
#[test]
fn a_scan_that_is_always_refused_gives_up() {
    let mut scanner =
        Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32).refusing(MAX_SCAN_ATTEMPTS + 1))
            .unwrap();
    scanner.warm_up().unwrap();
    assert!(matches!(
        scanner.scan_image(&settings(false, 1), calibration::DEFAULT_GAIN),
        Err(ScanError::Scsi(scsi::Error::InvalidResponse(_)))
    ));
}

/// Infrared leads
#[test]
fn channels_are_armed_and_scanned_infrared_first() {
    let image = padded(
        &[
            be_line(&[1, 0, 1, 0, 1, 0, 90]),
            be_line(&[2, 0, 2, 0, 2, 0, 91]),
        ],
        STRIDE,
    );
    let mut scanner = Ls5000::new(ScanMock::new(image, STRIDE as u32)).unwrap();
    scanner.warm_up().unwrap();
    // Captured before the SCAN clears them
    let armed = {
        let settings = settings(true, 1);
        scanner.arm(&settings, calibration::DEFAULT_GAIN).unwrap();
        scanner.transport.windows_set.clone()
    };
    assert_eq!(armed, [9, 1, 2, 3]);

    scanner.scan(channels(&settings(true, 1))).unwrap();
    assert_eq!(scanner.transport.scans, [vec![9, 1, 2, 3]]);
}

/// Armed correctly, but refused before it reaches the wire: the readout is not implemented
#[test]
fn a_multi_sampled_pass_is_refused_rather_than_armed() {
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();
    scanner.warm_up().unwrap();

    assert!(matches!(
        scanner.scan_image(&settings(false, 4), calibration::DEFAULT_GAIN),
        Err(ScanError::Scsi(scsi::Error::Unsupported(_)))
    ));
    assert!(
        scanner.transport.tails.is_empty(),
        "a window was armed for a pass that cannot be read"
    );
}

/// Every window a single-sampled pass arms carries the same tail, and the gain it was given
#[test]
fn every_window_of_a_pass_is_armed_alike() {
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();
    scanner.warm_up().unwrap();
    scanner
        .arm(&settings(false, 1), calibration::DEFAULT_GAIN)
        .unwrap();

    assert_eq!(scanner.transport.tails.len(), 3);
    for tail in &scanner.transport.tails {
        assert_eq!(tail[..6], [0x00, 0x80, 0x01, 0x02, 0x02, 0xFF]);
    }
    let armed: Vec<u32> = scanner
        .transport
        .tails
        .iter()
        .map(|t| u32::from_be_bytes(t[6..10].try_into().unwrap()))
        .collect();
    let seed = calibration::DEFAULT_GAIN;
    assert_eq!(armed, [seed.red, seed.green, seed.blue]);
}

/// Image reads are bulk and 512-aligned, not one line at a time
#[test]
fn image_reads_are_bulk_and_aligned() {
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();
    scan_one(&mut scanner, &settings(false, 1));

    let reads = &scanner.transport.image_reads;
    assert!(!reads.is_empty());
    // The frame here is smaller than a chunk, so every read is the whole remaining pass
    for &want in reads {
        assert!(
            want % READ_ALIGNMENT == 0 || u64::from(want) == settings(false, 1).expected_bytes(),
            "read of {want} is not 512-aligned"
        );
    }
}

/// A chunk is never smaller than a line, or a pass with a line longer than the ceiling would
/// ask for zero bytes and stall
#[test]
fn the_chunk_is_never_smaller_than_a_line() {
    let mut scanner = Ls5000::new(mock()).unwrap();
    let capabilities = capabilities::fixture::capabilities();
    let full = ScanSettings {
        resolution: Dpi::_4000.to_dpi(),
        ir: true,
        samples: Samples::default(),
        window: geometry::whole_frame(0, capabilities),
        capabilities,
    };
    let chunk = scanner.image_chunk(&full);
    assert!(chunk >= full.bytes_per_line() as u32);
}

/// Metering is host-side: the pass is an ordinary scan at the metering resolution, and the
/// gains come off the image rather than out of a register
#[test]
fn metering_scans_and_scales_the_gain_off_the_image() {
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();
    scanner.warm_up().unwrap();

    // Sized so the metering resolution lands on the same 1x2 frame the mock serves
    let frame = ScanArea {
        x_pos: 0,
        y_pos: 0,
        x_size: 28,
        y_size: 28,
    };
    let one_pass = Metering {
        passes: 1,
        ..Metering::default()
    };
    let gain = scanner
        .autoexpose(frame, calibration::DEFAULT_GAIN, one_pass)
        .unwrap();

    // The mock serves a dim frame, so every channel asks for more gain than it started with,
    // and nothing bounds where it lands
    for channel in crate::scanners::nikon::Channel::RGB {
        assert!(
            gain.get(channel) > calibration::DEFAULT_GAIN.get(channel),
            "{channel:?} did not scale up off a dim frame"
        );
    }

    // Every window it armed was a normal single-sampled one at the metering resolution
    assert!(!scanner.transport.tails.is_empty());
    for tail in &scanner.transport.tails {
        assert_eq!(tail[2], 0x01);
        assert_eq!(tail[0], 0x00);
    }
}

#[test]
fn autofocus_targets_the_frame_center() {
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();
    let capabilities = capabilities::fixture::capabilities();
    let whole_frame = ScanSettings {
        window: geometry::whole_frame(0, capabilities),
        ..settings(false, 1)
    };
    scanner.autofocus(whole_frame.center()).unwrap();

    // The full 3946 x 5959 window, centered
    let mut expected = vec![0u8; 9];
    expected[1..5].copy_from_slice(&1973u32.to_be_bytes());
    expected[5..9].copy_from_slice(&2979u32.to_be_bytes());
    assert_eq!(scanner.transport.autofocus_payload, Some(expected));
}

#[test]
fn a_truncated_pass_is_an_error() {
    // Two lines declared, one delivered
    let mut scanner = Ls5000::new(ScanMock::new(
        padded(&[be_line(&[1, 0, 1, 0, 1])], STRIDE),
        STRIDE as u32,
    ))
    .unwrap();
    scanner.warm_up().unwrap();
    assert!(matches!(
        scanner.scan_image(&settings(false, 1), calibration::DEFAULT_GAIN),
        Err(ScanError::Scsi(scsi::Error::InvalidResponse(_)))
    ));
}

/// A zero-length line makes the read loop exit before it starts, so the SCAN would be left
/// pending with nothing to drain it and an empty image would come back as a success
#[test]
fn a_window_narrower_than_a_pixel_is_refused_before_arming() {
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();
    let settings = ScanSettings {
        window: ScanArea { x_size: 0, ..TINY },
        ..settings(false, 1)
    };
    assert!(matches!(
        scanner.scan_image(&settings, calibration::DEFAULT_GAIN),
        Err(ScanError::Scsi(scsi::Error::Unsupported(_)))
    ));
}

/// The boundary itself is legal here, and one dot past it is not
#[test]
fn the_window_bound_is_inclusive_of_the_reported_boundary() {
    let capabilities = capabilities::fixture::capabilities();
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();

    fn window_at(capabilities: crate::scanners::nikon::limits::DeviceLimits) -> WindowDescriptor {
        WindowParams {
            samples: Samples::default(),
            exposure: 0,
        }
        .descriptor(
            &ScanSettings {
                window: geometry::whole_frame(0, capabilities),
                ..settings(false, 1)
            },
            Channel::Red,
        )
    }

    let at_boundary = window_at(capabilities);
    assert_eq!(at_boundary.length, capabilities.boundary_y);
    let mut past = window_at(capabilities);
    past.length = capabilities.boundary_y + 1;

    assert!(scanner.set_window(Channel::Red, at_boundary).is_ok());
    assert!(matches!(
        scanner.set_window(Channel::Red, past),
        Err(scsi::Error::Unsupported(_))
    ));
}

/// Every frame takes its window off the table the feeder reported
#[test]
fn a_roll_yields_a_frame_per_record() {
    let capabilities = capabilities::fixture::capabilities();
    let table = FrameBoundaries::evenly_spaced(3, capabilities.frame_pitch, &[]);
    let mut scanner = Ls5000::new(ScanMock::new(rgb_image(), STRIDE as u32)).unwrap();
    scanner.warm_up().unwrap();

    for record in &table.0 {
        let settings = ScanSettings {
            window: ScanArea {
                y_pos: record.scan_area(capabilities).y_pos,
                ..TINY
            },
            ..settings(false, 1)
        };
        let frame = scanner
            .scan_image(&settings, calibration::DEFAULT_GAIN)
            .unwrap();
        assert_eq!(frame.rgb.dimensions(), (1, 2));
    }
}

/// Assembled from the mode page rather than held as a blob, so the wire bytes are pinned
#[test]
fn opening_pins_the_units_with_the_captured_parameter_list() {
    let scanner = Ls5000::new(mock()).unwrap();
    assert_eq!(
        scanner
            .transport
            .data_out(0x15)
            .expect("MODE SELECT was sent"),
        [
            0x00, 0x00, 0x00, 0x08, // header, block descriptor length 8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // block descriptor
            0x03, 0x06, 0x00, 0x00, 0x0F, 0xA0, 0x00, 0x00, // page 0x03, 4000 DPI
        ]
    );
}

#[test]
fn opening_takes_its_geometry_from_the_device() {
    let scanner = Ls5000::new(mock()).unwrap();
    assert_eq!(
        scanner.capabilities(),
        capabilities::fixture::capabilities()
    );
}

/// Page 0x00 advertises 0xC1 on every unit seen, so silence means something is wrong and
/// hardcoded constants would hide it behind plausible geometry
#[test]
fn opening_without_a_capability_page_fails() {
    assert!(matches!(
        Ls5000::new(MockTransport::new()),
        Err(scsi::Error::InvalidResponse(_))
    ));
}
