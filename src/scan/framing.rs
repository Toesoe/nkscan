//! Where the frames are, and how this unit expects us to find out
//!
//! Four mechanisms, picked from what the unit and the loaded adapter advertise.

use super::thumbnail;
use crate::{
    error::Error,
    protocol::{
        caps::{Capabilities, address::CoordinateBase, other::DataTypes},
        data::{Boundary, Rect},
    },
};

/// How a scan comes to know where each frame sits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    /// `Frames` already carries every frame. A masked adapter knows its own geometry
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
    /// Pick the mechanism this unit and adapter use
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
            if thumbnail::available(caps) {
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

    /// Whether this is a mechanism we have implemented
    ///
    /// [`Thumbnail`](Self::Thumbnail) takes the pass but not yet the boundary
    /// finding, and [`Perforation`](Self::Perforation) is not written at all
    pub fn ready(self) -> bool {
        matches!(self, Self::Published | Self::Caller)
    }
}

/// The frame table to send before the first pass, 2-11-6
///
/// Both the stage and autofocus resolve against it, so until one is sent
/// nothing steps. `length` is the film format, which nothing advertises, and is
/// ignored where the adapter publishes its own.
pub fn table(caps: &Capabilities, length: u32) -> Result<Boundary, Error> {
    let Some(published) = caps.frames.as_ref() else {
        return Ok(Boundary::default());
    };
    let limit = caps.address.y_axis.boundary;
    let axis_end = caps.address.y_axis.address_range.last;
    let mut frames = Vec::new();

    for (n, image) in published.images.iter().enumerate() {
        // A published length is the adapter's own geometry. Without one, the
        // caller's format is what gets tiled
        let extent = image.length.unwrap_or(length);

        // Past the boundary the stage target comes out behind the home stop,
        // and the mechanism grinds there until a power cycle
        if extent > limit {
            return Err(Error::Unsupported {
                op: "frame table",
                reason: format!(
                    "a frame of {extent} is past the {limit} boundary and would stall the stage"
                ),
            });
        }

        // Where this image's area ends: the next one begins, or the axis does
        let stop = published
            .images
            .get(n + 1)
            .map_or(axis_end, |next| next.top);
        let tops: Vec<u32> = match image.length {
            Some(_) => vec![image.top],
            None => (0..)
                .map(|f| image.top + f * extent)
                .take_while(|top| top + extent <= stop)
                .collect(),
        };

        frames.extend(tops.into_iter().map(|top| Rect {
            top,
            left: image.left,
            bottom: top + extent,
            right: image.left + image.width,
        }));
    }
    Ok(Boundary { frames })
}
