//! Where the frames are, and how this unit expects us to find out
//!
//! Four mechanisms, picked from what the unit and the loaded holder advertise.
//! Deciding once and up front is the point: a job needing work we have not
//! written fails here with a reason, rather than part-way through a scan.

use crate::{
    protocol::caps::{
        Capabilities,
        address::CoordinateBase,
        other::{DataTypes, HostCooperation},
        set_window::ScanKind,
    },
    session::Session,
};

/// How a scan comes to know where each frame sits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    /// `C8h` already carries every frame, lengths and all. A masked holder
    /// knows its own geometry
    Published,
    /// `C8h` carries rectangles with no length. 2-11-6: after a thumbnail of
    /// strip film the host works the boundaries out and sends them with `88h`
    Thumbnail,
    /// No rectangles at all. 135 film seeks by counting perforations: read
    /// `8Eh` and write `8Fh` back
    Perforation,
    /// Neither mechanism is offered, so the caller has to say where to scan
    Caller,
}

impl Framing {
    /// Pick the mechanism this unit and holder use
    pub fn choose(caps: &Capabilities) -> Self {
        let rects = caps
            .address
            .coordinate_base
            .contains(CoordinateBase::FRAME_RECTS);

        if rects {
            // Lengths present means there is nothing left to measure
            if caps.frames.as_ref().is_some_and(|f| f.measured()) {
                return Self::Published;
            }
            if thumbnails(caps) {
                return Self::Thumbnail;
            }
            return Self::Caller;
        }

        // Perforations are the other way a frame gets found, and only the
        // families that read 8Eh can do it
        if caps
            .features
            .data_types
            .contains(DataTypes::PERFORATION_READ)
        {
            return Self::Perforation;
        }
        Self::Caller
    }

    /// The same choice for an open session
    pub fn of(session: &Session) -> Self {
        Self::choose(session.capabilities())
    }

    /// Whether this is a mechanism we have implemented
    ///
    /// [`Thumbnail`](Self::Thumbnail) takes the pass but not yet the boundary
    /// finding, and [`Perforation`](Self::Perforation) is not written at all
    pub fn ready(self) -> bool {
        matches!(self, Self::Published | Self::Caller)
    }
}

/// Whether this unit and holder thumbnail
///
/// Support follows the adapter rather than the model: the LS-5000 offers it on
/// some adapters and leaves the resolution columns blank on the rest, so an
/// advertised kind is not enough on its own
fn thumbnails(caps: &Capabilities) -> bool {
    caps.set_window.kind.contains(ScanKind::THUMBNAIL)
        && caps.address.thumbnail_resolution.start > 0
}

/// Whether the host has to build the thumbnail itself, `E1h` byte 4 bit 0
pub fn host_builds_thumbnail(caps: &Capabilities) -> bool {
    caps.features
        .cooperation
        .contains(HostCooperation::THUMBNAIL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::caps::{
        Page,
        address::Address,
        frames::{Frame, Frames},
        identity::Identity,
        other::Features,
        set_window::SetWindowFunction,
    };

    /// `rects` sets C1h byte 16 bit 6, `thumb` the thumbnail resolution,
    /// `perforation` the E1h bit the 5000 family sets
    fn caps(rects: bool, thumb: bool, perforation: bool, measured: bool) -> Capabilities {
        let mut p = vec![0u8; 91];
        p[1] = Address::PAGE_CODE;
        p[3] = 87;
        p[16] = if rects { 0x42 } else { 0x02 };
        p[18..20].copy_from_slice(&4000u16.to_be_bytes());
        p[20..22].copy_from_slice(&4000u16.to_be_bytes());
        p[22..24].copy_from_slice(&666u16.to_be_bytes());
        if thumb {
            p[70..72].copy_from_slice(&83u16.to_be_bytes());
            p[72..74].copy_from_slice(&83u16.to_be_bytes());
        }
        let address = Address::try_from(&Page::new(Address::PAGE_CODE, p).unwrap()).unwrap();

        let mut d = vec![0u8; 28];
        d[1] = SetWindowFunction::PAGE_CODE;
        d[3] = 24;
        d[4] = if thumb { 0x03 } else { 0x01 };
        let set_window =
            SetWindowFunction::try_from(&Page::new(SetWindowFunction::PAGE_CODE, d).unwrap())
                .unwrap();

        let mut e = vec![0u8; 39];
        e[1] = Features::PAGE_CODE;
        e[3] = 35;
        // Perforation read is byte 9 bit 6
        e[9] = if perforation { 0x40 } else { 0x00 };
        let features = Features::try_from(&Page::new(Features::PAGE_CODE, e).unwrap()).unwrap();

        let mut i = vec![0u8; 36];
        i[4] = 31;

        Capabilities {
            identity: Identity::parse(&i).unwrap(),
            address,
            features,
            set_window,
            ccd: None,
            frames: rects.then(|| Frames {
                images: vec![Frame {
                    left: 518,
                    top: 2236,
                    width: 8964,
                    length: measured.then_some(13176),
                }],
            }),
        }
    }

    /// A masked holder publishes everything and needs no pass
    #[test]
    fn measured_rectangles_are_published_framing() {
        assert_eq!(
            Framing::choose(&caps(true, true, false, true)),
            Framing::Published
        );
    }

    /// The strip holder on an LS-9000: rectangles without lengths
    #[test]
    fn rectangles_without_lengths_need_a_thumbnail() {
        assert_eq!(
            Framing::choose(&caps(true, true, false, false)),
            Framing::Thumbnail
        );
    }

    /// An adapter with no thumbnail leaves it to the caller, even though the
    /// unit publishes rectangles
    #[test]
    fn rectangles_without_a_thumbnail_fall_to_the_caller() {
        assert_eq!(
            Framing::choose(&caps(true, false, false, false)),
            Framing::Caller
        );
    }

    /// 135 film on an LS-5000 counts perforations instead
    #[test]
    fn no_rectangles_means_perforations_where_they_are_offered() {
        assert_eq!(
            Framing::choose(&caps(false, false, true, false)),
            Framing::Perforation
        );
        assert_eq!(
            Framing::choose(&caps(false, false, false, false)),
            Framing::Caller
        );
    }

    /// Only two of the four are written, and saying so is the point
    #[test]
    fn the_unwritten_mechanisms_report_themselves() {
        assert!(Framing::Published.ready());
        assert!(Framing::Caller.ready());
        assert!(!Framing::Thumbnail.ready());
        assert!(!Framing::Perforation.ready());
    }
}
