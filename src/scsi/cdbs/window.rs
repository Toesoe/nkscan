//! GET WINDOW(10) and SET WINDOW(10), SCSI-2 scanner devices, 15.2.2 / 15.2.9.

use crate::scsi::{
    Cdb, Command, CommandData, Error,
    fields::{be_u24, lun_byte},
};

#[derive(Debug, Copy, Clone)]
pub struct GetWindow {
    /// Logical unit number (3 bits)
    lun: u8,
    /// "Single": specifies that a single window descriptor shall be returned for the specified window identifier
    single: bool,
    /// ScanArea identifier
    window_identifier: u8,
    /// Transfer length (24-bits)
    transfer_length: u32,
    /// Control,
    control: u8,
}

impl GetWindow {
    /// `transfer_length` is how many bytes we're willing to receive - the
    /// caller has to size it (header + however many descriptors of however
    /// many bytes are expected back), since neither the descriptor count nor
    /// the vendor-specific tail length is something this generic command can
    /// know ahead of time; that's device-specific.
    pub fn new(
        lun: u8,
        single: bool,
        window_identifier: u8,
        transfer_length: u32,
        control: u8,
    ) -> Self {
        Self {
            lun,
            single,
            window_identifier,
            transfer_length,
            control,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ImageCompositionCode {
    BiLevelBlackAndWhite,
    DitheredHalftoneBlackAndWhite,
    Greyscale,
    BiLevelRgb,
    DitheredHalftoneRgb,
    Rgb,
    Reserved(u8),
}

impl ImageCompositionCode {
    fn to_byte(self) -> u8 {
        match self {
            ImageCompositionCode::BiLevelBlackAndWhite => 0x00,
            ImageCompositionCode::DitheredHalftoneBlackAndWhite => 0x01,
            ImageCompositionCode::Greyscale => 0x02,
            ImageCompositionCode::BiLevelRgb => 0x03,
            ImageCompositionCode::DitheredHalftoneRgb => 0x04,
            ImageCompositionCode::Rgb => 0x05,
            ImageCompositionCode::Reserved(x) => x,
        }
    }

    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => ImageCompositionCode::BiLevelBlackAndWhite,
            0x01 => ImageCompositionCode::DitheredHalftoneBlackAndWhite,
            0x02 => ImageCompositionCode::Greyscale,
            0x03 => ImageCompositionCode::BiLevelRgb,
            0x04 => ImageCompositionCode::DitheredHalftoneRgb,
            0x05 => ImageCompositionCode::Rgb,
            other => ImageCompositionCode::Reserved(other),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PaddingType {
    NoPadding,
    PadWithZeros,
    PadWithOnes,
    Truncate,
    Reserved(u8),
}

impl PaddingType {
    fn to_byte(self) -> u8 {
        match self {
            PaddingType::NoPadding => 0x00,
            PaddingType::PadWithZeros => 0x01,
            PaddingType::PadWithOnes => 0x02,
            PaddingType::Truncate => 0x03,
            PaddingType::Reserved(x) => x,
        }
    }

    /// Padding type only occupies the low 3 bits of its byte (the rest is
    /// shared with the RIF bit and reserved bits), so only 0x00-0x07 are
    /// actually representable - anything above 0x03 is reserved.
    fn from_byte(byte: u8) -> Self {
        match byte & 0x07 {
            0x00 => PaddingType::NoPadding,
            0x01 => PaddingType::PadWithZeros,
            0x02 => PaddingType::PadWithOnes,
            0x03 => PaddingType::Truncate,
            other => PaddingType::Reserved(other),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompressionType {
    NoCompression,
    CcittGroupIii1dimensional,
    CcittGroupIii2dimensional,
    CcittGroupIv2dimensional,
    Reserved(u8),
    Ocr,
    Vendor(u8),
}

impl CompressionType {
    fn to_byte(self) -> u8 {
        match self {
            CompressionType::NoCompression => 0x00,
            CompressionType::CcittGroupIii1dimensional => 0x01,
            CompressionType::CcittGroupIii2dimensional => 0x02,
            CompressionType::CcittGroupIv2dimensional => 0x03,
            CompressionType::Reserved(x) => x,
            CompressionType::Ocr => 0x10,
            CompressionType::Vendor(x) => x,
        }
    }

    /// 04h-0Fh and 11h-7Fh are reserved; 80h-FFh is the vendor-specific range
    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => CompressionType::NoCompression,
            0x01 => CompressionType::CcittGroupIii1dimensional,
            0x02 => CompressionType::CcittGroupIii2dimensional,
            0x03 => CompressionType::CcittGroupIv2dimensional,
            0x10 => CompressionType::Ocr,
            0x80..=0xFF => CompressionType::Vendor(byte),
            other => CompressionType::Reserved(other),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// GET WINDOW 4-byte data header, precedes one or more window descriptors
pub struct GetWindowHeader {
    /// Length in bytes of all the data that follows this field, not including itself
    pub data_length: u16,
    /// Length in bytes of each window descriptor
    pub descriptor_length: u16,
}

#[derive(Debug, Clone)]
/// The 40-byte standard descriptor
pub struct WindowDescriptor {
    /// ScanArea identifier
    pub id: u8,
    pub auto: bool,
    pub x_resolution: u16,
    pub y_resolution: u16,
    pub x_upper_left: u32,
    pub y_upper_left: u32,
    pub width: u32,
    pub length: u32,
    pub brightness: u8,
    pub threshold: u8,
    pub contrast: u8,
    pub composition: ImageCompositionCode,
    pub bits_per_pixel: u8,
    pub halftone_pattern: u16,
    pub rif: bool,
    pub padding: PaddingType,
    pub bit_ordering: u16,
    pub compression: CompressionType,
    pub compression_arg: u8,
    pub vendor: Vec<u8>,
}

impl WindowDescriptor {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![0u8; 40];
        buf[0] = self.id;
        buf[1] = self.auto as u8;
        buf[2..4].copy_from_slice(&self.x_resolution.to_be_bytes());
        buf[4..6].copy_from_slice(&self.y_resolution.to_be_bytes());
        buf[6..10].copy_from_slice(&self.x_upper_left.to_be_bytes());
        buf[10..14].copy_from_slice(&self.y_upper_left.to_be_bytes());
        buf[14..18].copy_from_slice(&self.width.to_be_bytes());
        buf[18..22].copy_from_slice(&self.length.to_be_bytes());
        buf[22] = self.brightness;
        buf[23] = self.threshold;
        buf[24] = self.contrast;
        buf[25] = self.composition.to_byte();
        buf[26] = self.bits_per_pixel;
        buf[27..29].copy_from_slice(&self.halftone_pattern.to_be_bytes());
        buf[29] = ((self.rif as u8) << 7) | self.padding.to_byte();
        buf[30..32].copy_from_slice(&self.bit_ordering.to_be_bytes());
        buf[32] = self.compression.to_byte();
        buf[33] = self.compression_arg;
        buf.extend_from_slice(&self.vendor);
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            id: bytes[0],
            auto: bytes[1] & 1 == 1,
            x_resolution: u16::from_be_bytes([bytes[2], bytes[3]]),
            y_resolution: u16::from_be_bytes([bytes[4], bytes[5]]),
            x_upper_left: u32::from_be_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]),
            y_upper_left: u32::from_be_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]),
            width: u32::from_be_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]),
            length: u32::from_be_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]),
            brightness: bytes[22],
            threshold: bytes[23],
            contrast: bytes[24],
            composition: ImageCompositionCode::from_byte(bytes[25]),
            bits_per_pixel: bytes[26],
            halftone_pattern: u16::from_be_bytes([bytes[27], bytes[28]]),
            rif: bytes[29] & 0b1000_0000 != 0,
            padding: PaddingType::from_byte(bytes[29] & 0b111),
            bit_ordering: u16::from_be_bytes([bytes[30], bytes[31]]),
            compression: CompressionType::from_byte(bytes[32]),
            compression_arg: bytes[33],
            vendor: bytes[40..].to_vec(),
        }
    }
}

impl Command for GetWindow {
    type Response = Vec<WindowDescriptor>;
    type Cdb = Cdb<10>;

    fn cdb(&self) -> Self::Cdb {
        let [length_hi, length_mid, length_lo] = be_u24(self.transfer_length);
        Cdb([
            0x25, // opcode
            lun_byte(self.lun) | (self.single as u8),
            0x00, // reserved
            0x00, // reserved
            0x00, // reserved
            self.window_identifier,
            length_hi,
            length_mid,
            length_lo,
            self.control,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Read(self.transfer_length as usize)
    }

    fn parse_response(&self, data: &[u8]) -> Result<Self::Response, Error> {
        if data.len() < 8 {
            return Err(Error::InvalidResponse(
                "GET WINDOW response shorter than the 8-byte header",
            ));
        }

        let header = GetWindowHeader {
            data_length: u16::from_be_bytes([data[0], data[1]]),
            descriptor_length: u16::from_be_bytes([data[6], data[7]]),
        };

        let descriptor_len = header.descriptor_length as usize;
        if descriptor_len < 40 {
            return Err(Error::InvalidResponse(
                "window descriptor shorter than the standardized 40 bytes",
            ));
        }

        // `header.data_length` is the device's advertised total across every
        // window it has defined, not the size of `data`. Per spec it's
        // "not adjusted to reflect truncation," so it can't be used to size
        // anything here. Only decode as many whole descriptors as actually
        // fit in what we received; a short trailing remainder (data.len()
        // not an exact multiple of descriptor_len) is silently dropped
        // rather than indexed into.
        let descriptors: Vec<_> = data[8..]
            .chunks_exact(descriptor_len)
            .map(WindowDescriptor::from_bytes)
            .collect();

        Ok(descriptors)
    }
}

#[cfg(test)]
mod get_window_tests {
    use super::*;

    #[test]
    fn cdb_matches_real_capture() {
        // LS-9000ED per-channel query: `25 01 00 00 00 01 00 00 3A 80`,
        // allocation length 0x3A = 8-byte header + one 50-byte descriptor.
        let get_window = GetWindow::new(0, true, 1, 58, 0x80);
        assert_eq!(
            get_window.cdb().0,
            [0x25, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x3A, 0x80]
        );
    }

    #[test]
    fn cdb_encodes_allocation_length_as_big_endian_u24() {
        let get_window = GetWindow::new(0, false, 0, 0x01_2345, 0x00);
        let cdb = get_window.cdb().0;
        assert_eq!([cdb[6], cdb[7], cdb[8]], [0x01, 0x23, 0x45]);
    }

    #[test]
    fn cdb_encodes_lun_and_single_bit() {
        let cdb = GetWindow::new(3, true, 9, 0, 0).cdb().0;
        assert_eq!(cdb[1], (3 << 5) | 1);
        assert_eq!(cdb[5], 9);
        assert_eq!(GetWindow::new(0, false, 0, 0, 0).cdb().0[1], 0x00);
    }
}

/// SET WINDOW(10), SCSI-2 scanner devices, 15.2.9
pub struct SetWindow {
    /// Logical unit number (3 bits)
    lun: u8,
    /// Framed DATA OUT payload: 8-byte header + each descriptor's encoded bytes back to back
    parameters: Vec<u8>,
    /// Control byte
    control: u8,
}

impl SetWindow {
    pub fn new(lun: u8, descriptors: &[WindowDescriptor], control: u8) -> Self {
        let descriptor_len = descriptors.first().map_or(0, |d| d.to_bytes().len());
        let mut parameters = vec![0u8; 8];
        parameters[6..8].copy_from_slice(&(descriptor_len as u16).to_be_bytes());
        for descriptor in descriptors {
            parameters.extend_from_slice(&descriptor.to_bytes());
        }
        Self {
            lun,
            parameters,
            control,
        }
    }
}

impl Command for SetWindow {
    type Response = ();
    type Cdb = Cdb<10>;

    fn cdb(&self) -> Self::Cdb {
        let [length_hi, length_mid, length_lo] = be_u24(self.parameters.len() as u32);
        Cdb([
            0x24, // opcode
            lun_byte(self.lun),
            0x00, // reserved
            0x00, // reserved
            0x00, // reserved
            0x00, // reserved
            length_hi,
            length_mid,
            length_lo,
            self.control,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Write(&self.parameters)
    }

    fn parse_response(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod set_window_tests {
    use super::*;

    fn descriptor(id: u8, vendor: Vec<u8>) -> WindowDescriptor {
        WindowDescriptor {
            id,
            auto: false,
            x_resolution: 4000,
            y_resolution: 4000,
            x_upper_left: 0,
            y_upper_left: 0,
            width: 8964,
            length: 13176,
            brightness: 0,
            threshold: 0,
            contrast: 0,
            composition: ImageCompositionCode::Rgb,
            bits_per_pixel: 16,
            halftone_pattern: 0,
            rif: false,
            padding: PaddingType::NoPadding,
            bit_ordering: 0,
            compression: CompressionType::NoCompression,
            compression_arg: 0,
            vendor,
        }
    }

    #[test]
    fn cdb_encodes_opcode_and_lun() {
        let set_window = SetWindow::new(3, &[], 0);
        let cdb = set_window.cdb().0;
        assert_eq!(cdb[0], 0x24);
        assert_eq!(cdb[1], 3 << 5);
    }

    #[test]
    fn cdb_encodes_control_byte_verbatim() {
        let set_window = SetWindow::new(0, &[], 0x80);
        assert_eq!(set_window.cdb().0[9], 0x80);
    }

    #[test]
    fn header_descriptor_length_matches_encoded_descriptor() {
        let set_window = SetWindow::new(0, &[descriptor(1, vec![0xAA, 0xBB])], 0);
        let CommandData::Write(payload) = set_window.data() else {
            panic!("expected Write");
        };
        // header[6:8] = descriptor length; descriptor is 40 standard + 2 vendor
        assert_eq!(u16::from_be_bytes([payload[6], payload[7]]), 42);
        assert_eq!(payload.len(), 8 + 42);
    }

    #[test]
    fn cdb_transfer_length_matches_full_framed_payload() {
        let set_window = SetWindow::new(0, &[descriptor(1, vec![0xAA, 0xBB])], 0);
        let cdb = set_window.cdb().0;
        let transfer_length = u32::from_be_bytes([0, cdb[6], cdb[7], cdb[8]]);
        assert_eq!(transfer_length as usize, 8 + 42);
    }

    #[test]
    fn empty_descriptors_is_zero_length_and_not_an_error() {
        let set_window = SetWindow::new(0, &[], 0);
        assert!(matches!(set_window.data(), CommandData::Write(p) if p.len() == 8));
    }

    #[test]
    fn descriptor_to_bytes_round_trips_through_from_bytes() {
        let original = descriptor(1, vec![0xAA, 0xBB]);
        let bytes = original.to_bytes();
        assert_eq!(bytes.len(), 42);
        let round_tripped = WindowDescriptor::from_bytes(&bytes);
        assert_eq!(round_tripped.id, original.id);
        assert_eq!(round_tripped.x_resolution, original.x_resolution);
        assert_eq!(round_tripped.width, original.width);
        assert_eq!(round_tripped.composition, original.composition);
        assert_eq!(round_tripped.vendor, original.vendor);
    }

    /// Byte 29 packs RIF into bit 7 and padding type into bits 0-2, so the two
    /// have to survive being decoded out of the same byte independently.
    #[test]
    fn rif_and_padding_share_byte_29() {
        for padding in [
            PaddingType::NoPadding,
            PaddingType::PadWithZeros,
            PaddingType::PadWithOnes,
            PaddingType::Truncate,
        ] {
            for rif in [false, true] {
                let mut original = descriptor(1, vec![]);
                original.rif = rif;
                original.padding = padding;

                let bytes = original.to_bytes();
                assert_eq!(bytes[29], ((rif as u8) << 7) | padding.to_byte());

                let round_tripped = WindowDescriptor::from_bytes(&bytes);
                assert_eq!(round_tripped.rif, rif, "rif lost for {padding:?}");
                assert_eq!(round_tripped.padding, padding, "padding lost, rif={rif}");
            }
        }
    }
}
