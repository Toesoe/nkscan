//! What the Nikon Coolscans have in common
//!
//! Nikon-specific but not model-specific: the vendor CDB framing, the capability page, the
//! frame table. Anything whose *encoding* differs between models stays in that model's module
//! even when it shares a name here, because the wire formats disagree in ways that would be
//! silent if merged.

pub mod adapter;
pub mod cdbs;
pub mod decode;
pub mod dtc;
pub mod limits;
pub mod limits_usb;
pub mod metering;
pub mod status_usb;
pub mod usb;
pub mod vendor_read_write;

/// A color the scanner's lamp emits, which is also the window it is scanned through
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Channel {
    /// The composite window, which carries geometry but no exposure of its own
    ///
    /// Not every model has one, and none of them scan through it.
    All,
    Red,
    Green,
    Blue,
    Ir,
}

impl Channel {
    /// The three visible channels, in the order they are staged
    pub const RGB: [Channel; 3] = [Channel::Red, Channel::Green, Channel::Blue];
    /// The visible channels plus infrared, as a dust-removal pass needs
    pub const RGBI: [Channel; 4] = [Channel::Red, Channel::Green, Channel::Blue, Channel::Ir];

    pub fn to_id(self) -> u8 {
        match self {
            Channel::All => 0,
            Channel::Red => 1,
            Channel::Green => 2,
            Channel::Blue => 3,
            Channel::Ir => 9,
        }
    }

    /// The window identifier as it comes back off the scanner
    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => Channel::All,
            1 => Channel::Red,
            2 => Channel::Green,
            3 => Channel::Blue,
            9 => Channel::Ir,
            _ => return None,
        })
    }
}

/// Per-channel analog gain, as a window descriptor's tail carries it
///
/// Linear in the value and free in time: it amplifies rather than integrating longer. Persists
/// in the scanner across sessions, so a readback is whatever was last written, and metering that
/// starts from one compounds run over run.
///
/// A model with no separate infrared gain leaves that field zero, which is what its infrared
/// window carries anyway. The defaults are per model, so they live with the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelExposures {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub ir: u32,
}

/// Comma separated, which is the form a caller passes gains back in
impl std::fmt::Display for ChannelExposures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            red,
            green,
            blue,
            ir,
        } = self;
        write!(f, "{red},{green},{blue},{ir}")
    }
}

impl ChannelExposures {
    /// The same gain on every channel, asserting no white balance at all
    pub fn flat(exposure: u32) -> Self {
        Self {
            red: exposure,
            green: exposure,
            blue: exposure,
            ir: exposure,
        }
    }

    /// The gain staged for one channel
    ///
    /// The composite window has none of its own, so it reports the red one the scanners lead
    /// with.
    pub fn get(&self, channel: Channel) -> u32 {
        match channel {
            Channel::Red | Channel::All => self.red,
            Channel::Green => self.green,
            Channel::Blue => self.blue,
            Channel::Ir => self.ir,
        }
    }

    pub fn set(&mut self, channel: Channel, exposure: u32) {
        match channel {
            Channel::Red | Channel::All => self.red = exposure,
            Channel::Green => self.green = exposure,
            Channel::Blue => self.blue = exposure,
            Channel::Ir => self.ir = exposure,
        }
    }
}

/// A millimeter figure in device dots
///
/// `dots_per_inch` is the measurement unit the driver set at open, which is the same number the
/// mode page divides the inch by. It is not the scanner's optical resolution and not the same on
/// every model, so it is a parameter rather than a constant. Negative input floors at zero: a
/// window cannot start before the film does.
pub fn native_dots(millimeters: f32, dots_per_inch: u32) -> u32 {
    const MM_PER_INCH: f32 = 25.4;
    (millimeters * dots_per_inch as f32 / MM_PER_INCH)
        .round()
        .max(0.0) as u32
}

/// A length-prefixed, NUL-terminated ASCII name out of a VPD page
///
/// The count covers the terminator, so an 11-character name arrives as 12. Which page carries
/// one is per model: the LS-50's adapter names itself on 0x46, the LS-5000's on 0x01. The shape
/// is the same either way, and pages 0x60/0x61 carry parameter names in it too.
pub fn page_name(data: &[u8]) -> Option<String> {
    let len = usize::from(*data.first()?);
    let text: String = data
        .get(1..1 + len)?
        .iter()
        .take_while(|&&byte| byte != 0)
        .map(|&byte| char::from(byte))
        .collect();
    (!text.is_empty() && text.is_ascii()).then_some(text)
}

/// The per-channel analog gain out of a window descriptor's vendor tail
///
/// The last four bytes of the ten, big-endian. `None` if the tail is short, which is not the
/// same as a gain of zero: zero is a value a caller would go on to arm a black pass with.
pub fn exposure_from_vendor(vendor: &[u8]) -> Option<u32> {
    vendor
        .get(6..10)
        .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The identifiers are the wire format, and the same on every model here
    #[test]
    fn identifiers_round_trip() {
        for channel in [
            Channel::All,
            Channel::Red,
            Channel::Green,
            Channel::Blue,
            Channel::Ir,
        ] {
            assert_eq!(Channel::from_id(channel.to_id()), Some(channel));
        }
        assert_eq!(Channel::RGBI.map(Channel::to_id), [1, 2, 3, 9]);
        assert_eq!(Channel::from_id(4), None);
    }

    #[test]
    fn every_channel_round_trips_through_its_gain() {
        let mut gains = ChannelExposures::flat(0);
        for (index, channel) in Channel::RGBI.into_iter().enumerate() {
            gains.set(channel, index as u32 + 1);
        }
        assert_eq!((gains.red, gains.green, gains.blue, gains.ir), (1, 2, 3, 4));
        for (index, channel) in Channel::RGBI.into_iter().enumerate() {
            assert_eq!(gains.get(channel), index as u32 + 1);
        }
    }
}
