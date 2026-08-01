//! Scanner-specific VENDOR CDBs
//!
//! The encoding is shared, in
//! [`nikon::vendor_read_write`](crate::scanners::nikon::vendor_read_write). What is this
//! model's is the focus read length, which the two USB bodies disagree about.

pub use crate::scanners::nikon::vendor_read_write;

/// The firmware rejects a transfer length above 13 bytes here
///
/// See docs/OPEN_QUESTIONS.md section 18: the two drivers disagree and neither has been checked
/// against the other's value.
pub const FOCUS_READ_LEN: u32 = 13;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scsi::Command;

    /// The one value this module binds, on the wire where the firmware reads it
    #[test]
    fn the_focus_read_asks_for_this_models_length() {
        let cdb = vendor_read_write::VendorRead::focus(FOCUS_READ_LEN).cdb().0;
        assert_eq!(cdb[8], 13);
    }
}
