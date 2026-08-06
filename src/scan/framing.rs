//! Where the frames are, and how this unit expects us to find out
//!
//! Four mechanisms, picked from what the unit and the loaded holder advertise.

use crate::{
    error::Error,
    protocol::{
        caps::{
            Capabilities,
            address::CoordinateBase,
            other::{DataTypes, HostCooperation},
            set_window::ScanKind,
        },
        data::{Boundary, Rect},
    },
    session::Session,
};

/// How a scan comes to know where each frame sits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    /// `Frames` already carries every frame. A masked holder knows its own geometry
    Published,
    /// `Frames` carries rectangles with no length.
    /// 2-11-6: after a thumbnail of strip film the host works the boundaries out and sends them as `DataType::Boundary`
    Thumbnail,
    /// No rectangles at all. 135 film seeks by counting perforations: read `DataType::Perforation` and write `DataType::Boundary2` back
    Perforation,
    /// Neither mechanism is offered, so the caller has to say where to scan
    Caller,
}

impl Framing {
    /// Pick the mechanism this unit and holder use for Framing
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

        // Perforations are the other way a frame gets found, and only the families that read DataType::Perforation can do it
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

/// The frame table to send before the first pass
///
/// 2-11-6: until the host says where the frames are, the unit answers
/// `DataType::Boundary` with one rectangle over the whole sensor. A frame-kind
/// SET WINDOW against that table drives the stage to its home stop and back
/// rather than stepping to the frame.
///
/// A holder that publishes its own lengths needs nothing from the caller. A
/// strip publishes an opening and no length, so `length` is the film format,
/// tiled from the front edge until the opening runs out.
pub fn table(caps: &Capabilities, length: u32) -> Result<Boundary, Error> {
    let Some(published) = caps.frames.as_ref() else {
        return Ok(Boundary::default());
    };
    let limit = caps.address.y_axis.boundary;
    let mut frames = Vec::new();
    for (n, opening) in published.images.iter().enumerate() {
        let extent = opening.length.unwrap_or(length);
        if extent > limit {
            return Err(Error::Unsupported {
                op: "frame table",
                reason: format!(
                    "a frame of {extent} is past the {limit} boundary and would stall the stage"
                ),
            });
        }
        let right = opening.left + opening.width;
        // Where this opening stops: the next one begins, or the axis ends
        let stop = published
            .images
            .get(n + 1)
            .map_or(caps.address.y_axis.address_range.last, |next| next.top);
        let rect = |top| Rect {
            top,
            left: opening.left,
            bottom: top + length,
            right,
        };
        match opening.length {
            Some(measured) => frames.push(Rect {
                bottom: opening.top + measured,
                ..rect(opening.top)
            }),
            None => frames.extend(
                (0..)
                    .map(|f| opening.top + f * length)
                    .take_while(|top| top + length <= stop)
                    .map(rect),
            ),
        }
    }
    Ok(Boundary { frames })
}

/// Whether this unit and holder can capture a thumbnail
fn thumbnails(caps: &Capabilities) -> bool {
    caps.set_window.kind.contains(ScanKind::THUMBNAIL)
        && caps.address.thumbnail_resolution.start > 0
}

/// Whether the host has to build the thumbnail itself, `Features` byte 4 bit 0
pub fn host_builds_thumbnail(caps: &Capabilities) -> bool {
    caps.features
        .cooperation
        .contains(HostCooperation::THUMBNAIL)
}
