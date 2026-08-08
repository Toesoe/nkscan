//! The descriptors that scan one frame
//!
//! A pass is a set of descriptors, one per channel, agreeing on everything but
//! the channel identifier and its exposure. Building the set is done here so
//! the rules spanning it are decided once.

use crate::{
    error::Error,
    protocol::{
        caps::{
            Capabilities,
            address::Axis,
            set_window::{ColorComponents, ColorInterleaving, ScanKind, ScanMode},
        },
        data::Rect,
        window::{Channel, Composition, Flags, LENGTH, Window, deepest_depth},
    },
    scan::framing,
};

/// The most times a line can be read, the count being half of 2-10 byte 40
const MAX_SAMPLES: u8 = 16;

/// Lines the three-line readout tiles in
///
/// Stage positions tile against CCD rows in blocks of the line gap, so a window
/// ending part-way through one cannot be unscrambled and the decoder refuses it.
/// The pitch cancels, the gap dividing by it as an extent does, so this is the
/// same number of window units at every resolution
pub(crate) fn block(caps: &Capabilities) -> u32 {
    u32::from(caps.address.line_gap) * u32::from(caps.address.lines)
}

/// `extent` grown to a whole number of blocks
///
/// Up rather than down, since a frame is better over-covered than clipped
pub(crate) fn whole_blocks(caps: &Capabilities, extent: u32) -> u32 {
    match block(caps) {
        0 | 1 => extent,
        block => extent.div_ceil(block) * block,
    }
}

/// The color channels this unit scans
///
/// 2-10-6 has one code for a one-plane output and one for three, so a unit that
/// will not do RGB scans the one default channel
pub(crate) fn color_channels(caps: &Capabilities) -> Vec<Channel> {
    match caps.set_window.components.contains(ColorComponents::RGB) {
        true => vec![Channel::Red, Channel::Green, Channel::Blue],
        false => vec![Channel::Default],
    }
}

/// Blank descriptors for `channels`, carrying what every set agrees on
///
/// Composition counts the color planes in the stream, so infrared does not sway
/// it. What is left is the geometry and the mode, which the caller sets.
pub(crate) fn blank(caps: &Capabilities, channels: &[Channel]) -> Result<Vec<Window>, Error> {
    let bpp = deepest_depth(caps.set_window.depth).ok_or_else(|| Error::Unsupported {
        op: "scan window",
        reason: "this unit advertises no pixel depth".into(),
    })?;
    let composition = match channels.iter().filter(|c| c.is_color()).count() {
        1 => Composition::MultilevelBW,
        _ => Composition::MultilevelRGB,
    };

    Ok(channels
        .iter()
        .map(|channel| {
            let mut w =
                Window::try_from(&[0u8; LENGTH][..]).expect("a zeroed descriptor is long enough");
            w.id = channel.id();
            w.composition = composition;
            w.bpp = bpp;
            // 2-10 byte 45: the default, and what the unit reports back for a 0
            w.ae_value = 255;
            w
        })
        .collect())
}

/// What a pass over a frame should do
///
/// The caller's choices only. Pixel depth, plane count and where the axes stop
/// are read from [`Capabilities`] when the windows are built.
#[derive(Debug, Clone, Copy)]
pub struct Recipe {
    /// Samples per inch along the bar, and the stage stepping to match. Off the
    /// unit's ladder it rounds and reports `01h-37h` rather than refusing
    pub dpi: u16,
    /// Times each line is read for the unit to average, 1 to 16. Repeated reads
    /// of one line, not binning along it
    pub samples: u8,
    /// Which CCD rows the pass reads, 2-10 byte 44
    ///
    /// [`MULTILINE_SIMULTANEOUS`](ColorInterleaving::MULTILINE_SIMULTANEOUS)
    /// takes the three color rows at once, so the stage travels a third as far.
    /// The rows sit a line gap apart, so the host owes re-registration and the
    /// row-response correction. Nikon Scan's "super fine" is
    /// [`LINE_WITHOUT_DISTANCE`](ColorInterleaving::LINE_WITHOUT_DISTANCE), one
    /// row at a time, which owes nothing and takes about three times as long
    pub interleaving: ColorInterleaving,
    /// Add the infrared channel, which measures what is on the film rather than
    /// what is in it. Window 9, which neither spec admits exists
    pub infrared: bool,
}

impl Recipe {
    /// A quick pass to measure by, not to keep
    ///
    /// The coarsest the unit offers, one reading, and no infrared, which has no
    /// exposure to decide from a film's tones. Single-line where it is offered:
    /// a pass that only gets measured has no reason to owe registration
    pub fn metering(caps: &Capabilities) -> Self {
        let offered = caps.set_window.interleaving;
        Self {
            dpi: caps.address.x_axis.dpi_range.start,
            samples: 1,
            interleaving: match offered.contains(ColorInterleaving::LINE_WITHOUT_DISTANCE) {
                true => ColorInterleaving::LINE_WITHOUT_DISTANCE,
                false => ColorInterleaving::MULTILINE_SIMULTANEOUS,
            },
            infrared: false,
        }
    }

    /// Whether this unit will read the CCD the way this asks
    ///
    /// Worth checking before anything moves: the windows are not built until
    /// the frames are known, which is a stage move and a pass away
    pub fn supported(&self, caps: &Capabilities) -> Result<(), Error> {
        let offered = caps.set_window.interleaving;
        match self.interleaving.bits().count_ones() == 1 && offered.contains(self.interleaving) {
            true => Ok(()),
            false => Err(Error::Unsupported {
                op: "color interleaving",
                reason: format!(
                    "{:?} is not one reading mode this unit offers, which are {offered:?}",
                    self.interleaving
                ),
            }),
        }
    }

    /// `extent` as a whole number of readout blocks
    ///
    /// A table written by [`framing`] is already block-aligned, so this only
    /// does anything for a caller cropping to a rectangle of its own. Where
    /// there is no room left on the axis it goes down rather than up. One row at
    /// a time has nothing to tile and keeps the extent it was given
    fn blocks(&self, caps: &Capabilities, top: u32, extent: u32) -> u32 {
        if !self
            .interleaving
            .contains(ColorInterleaving::MULTILINE_SIMULTANEOUS)
        {
            return extent;
        }

        let y = &caps.address.y_axis;
        let room = y.address_range.last.saturating_sub(top).min(y.boundary);
        let grown = whole_blocks(caps, extent);
        match grown <= room {
            true => grown,
            false => grown.saturating_sub(block(caps)),
        }
    }

    /// The descriptors that scan `frame`, one per channel
    ///
    /// The frame's own edges are the window, so the stage steps to it and
    /// stays. Pass a rectangle inside a frame to scan a crop of it.
    pub fn windows(&self, caps: &Capabilities, frame: Rect) -> Result<Vec<Window>, Error> {
        self.supported(caps)?;
        if !(1..=MAX_SAMPLES).contains(&self.samples) {
            return Err(Error::Unsupported {
                op: "scan window",
                reason: format!(
                    "{} readings of a line is outside 1 to {MAX_SAMPLES}",
                    self.samples
                ),
            });
        }

        // The captures lead with infrared
        let mut channels = color_channels(caps);
        if self.infrared {
            channels.insert(0, Channel::Infrared);
        }

        let (x, y) = (&caps.address.x_axis, &caps.address.y_axis);
        let clamp =
            |v: u32, axis: &Axis| v.clamp(axis.address_range.start, axis.address_range.last);
        let origin = (clamp(frame.left, x), clamp(frame.top, y));

        // Measured from where the window actually starts, so growing the extent
        // cannot push the far edge off the axis
        let extent = self.blocks(caps, origin.1, frame.bottom.saturating_sub(origin.1));
        framing::reachable(caps, extent)?;
        let size = (frame.right.saturating_sub(origin.0).min(x.boundary), extent);

        // Byte 43 is normal quality in every captured scan. High speed turns up
        // only in the 666 dpi preview, and the LS-5000 does not offer it at all
        let mut mode = ScanMode::NORMAL_QUALITY;
        if self.samples > 1 {
            mode |= ScanMode::MULTI_READING;
        }

        let mut windows = blank(caps, &channels)?;
        for w in &mut windows {
            // A scan is square. Only a metering pass halves Y, in the prescan builder
            w.resolution = (self.dpi, self.dpi);
            w.origin = origin;
            w.size = size;
            w.scanning_kind = ScanKind::IMAGE;
            w.scanning_mode = mode;
            w.color_interleaving = self.interleaving;
            // Byte 41 bit 7 averages along the scan line, across the bar. Every
            // capture but the preview sets it, as every capture sets bit 0
            w.flags = Flags::AVERAGING | Flags::POSITIVE;
            // Byte 40 carries one less than the reading count. Its low nibble
            // leaves the color ordering to the unit
            w.multiple_reading = self.samples - 1;
        }
        Ok(windows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::caps::{
        Page, address::Address, identity::Identity, other::Features, set_window::SetWindowFunction,
    };

    /// An LS-9000 cut down to the fields a window builder reads
    fn caps() -> Capabilities {
        let mut p = vec![0u8; 91];
        p[1] = Address::PAGE_CODE;
        p[3] = 87;
        for axis in [18, 40] {
            p[axis..axis + 2].copy_from_slice(&4000u16.to_be_bytes());
            p[axis + 2..axis + 4].copy_from_slice(&4000u16.to_be_bytes());
            p[axis + 4..axis + 6].copy_from_slice(&666u16.to_be_bytes());
        }
        p[24..28].copy_from_slice(&20000u32.to_be_bytes());
        p[85] = 12;
        p[86] = 3;
        p[36..40].copy_from_slice(&10000u32.to_be_bytes());
        p[46..50].copy_from_slice(&20000u32.to_be_bytes());
        p[58..62].copy_from_slice(&20000u32.to_be_bytes());
        let address = Address::try_from(&Page::new(Address::PAGE_CODE, p).unwrap()).unwrap();

        let mut d = vec![0u8; 28];
        d[1] = SetWindowFunction::PAGE_CODE;
        d[3] = 24;
        d[6] = (ColorInterleaving::MULTILINE_SIMULTANEOUS
            | ColorInterleaving::LINE_WITHOUT_DISTANCE)
            .bits();
        d[7] = ColorComponents::RGB.bits();
        d[10] = crate::protocol::caps::set_window::BitDepth::BIT_16.bits();
        let set_window =
            SetWindowFunction::try_from(&Page::new(SetWindowFunction::PAGE_CODE, d).unwrap())
                .unwrap();

        let mut e = vec![0u8; 39];
        e[1] = Features::PAGE_CODE;
        e[3] = 35;
        let features = Features::try_from(&Page::new(Features::PAGE_CODE, e).unwrap()).unwrap();

        let mut i = vec![0u8; 36];
        i[4] = 31;

        Capabilities {
            identity: Identity::parse(&i).unwrap(),
            address,
            features,
            set_window,
            ccd: None,
            frames: None,
        }
    }

    fn frame() -> Rect {
        Rect {
            top: 2236,
            left: 518,
            bottom: 2236 + 8964,
            right: 518 + 8964,
        }
    }

    fn recipe() -> Recipe {
        Recipe {
            dpi: 4000,
            samples: 1,
            interleaving: ColorInterleaving::MULTILINE_SIMULTANEOUS,
            infrared: false,
        }
    }

    /// Infrared leads the set the way the captures send it, and is not a plane
    #[test]
    fn infrared_joins_the_set_without_joining_the_planes() {
        let windows = Recipe {
            infrared: true,
            ..recipe()
        }
        .windows(&caps(), frame())
        .expect("windows");

        assert_eq!(
            windows.iter().map(|w| w.id).collect::<Vec<_>>(),
            vec![9, 1, 2, 3]
        );
        assert!(
            windows
                .iter()
                .all(|w| w.composition == Composition::MultilevelRGB)
        );
    }

    /// Byte 40's high nibble is one less than the count, and byte 43 says so too
    #[test]
    fn multisampling_is_stated_in_both_bytes() {
        let four = Recipe {
            samples: 4,
            ..recipe()
        }
        .windows(&caps(), frame())
        .expect("windows");
        assert_eq!(four[0].multiple_reading, 3);
        assert!(four[0].scanning_mode.contains(ScanMode::MULTI_READING));

        let one = recipe().windows(&caps(), frame()).expect("windows");
        assert_eq!(one[0].multiple_reading, 0);
        assert!(!one[0].scanning_mode.contains(ScanMode::MULTI_READING));
    }

    /// A reading mode the unit has not got is refused before anything moves
    #[test]
    fn an_interleaving_this_unit_has_not_got_is_refused() {
        let mut caps = caps();
        caps.set_window.interleaving = ColorInterleaving::LINE_WITHOUT_DISTANCE;
        assert!(recipe().supported(&caps).is_err());
        assert!(recipe().windows(&caps, frame()).is_err());
    }

    /// 56mm of 6x6 is 8819 dots at 4000 dpi, which the three-line readout
    /// cannot tile: it takes blocks of the line gap times the CCD rows
    #[test]
    fn a_multi_line_window_is_whole_blocks_of_the_readout() {
        let frame = Rect {
            top: 2236,
            left: 518,
            bottom: 2236 + 8819,
            right: 518 + 8964,
        };
        let windows = recipe().windows(&caps(), frame).expect("windows");
        assert_eq!(windows[0].size.1, 8820);
        assert_eq!(windows[0].size.1 % 36, 0);

        // One row at a time has nothing to tile
        let single = Recipe {
            interleaving: ColorInterleaving::LINE_WITHOUT_DISTANCE,
            ..recipe()
        }
        .windows(&caps(), frame)
        .expect("windows");
        assert_eq!(single[0].size.1, 8819);
    }

    /// Growing the extent cannot push the far edge past the axis
    #[test]
    fn a_frame_against_the_end_of_the_axis_shrinks_instead() {
        let caps = caps();
        let end = caps.address.y_axis.address_range.last;
        let frame = Rect {
            top: end - 8819,
            left: 518,
            bottom: end,
            right: 518 + 8964,
        };
        let windows = recipe().windows(&caps, frame).expect("windows");
        assert_eq!(windows[0].size.1, 8784);
        assert!(windows[0].origin.1 + windows[0].size.1 <= end);
    }

    /// The window is the frame, so the stage steps to it and stays
    #[test]
    fn the_window_is_the_frame() {
        let windows = recipe().windows(&caps(), frame()).expect("windows");
        assert_eq!(windows[0].origin, (518, 2236));
        assert_eq!(windows[0].size, (8964, 8964));
    }
}
