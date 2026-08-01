//! Coolscan 8000 ED — recognized, not driven
//!
//! A placeholder so that the model has somewhere to live and so what is known about it is
//! written down rather than rediscovered. Nothing here talks to a scanner.
//!
//! The LS-8000 is the LS-9000's predecessor and the other half of
//! [`Family::MediumFormat`](crate::model::Family::MediumFormat): 120/220 film on a removable
//! holder, over FireWire 400, so enumeration finds it by INQUIRY string rather than by USB ids.
//!
//! # What it is believed to need
//!
//! `Protocol::Ls9000`, on the strength of the shared family, the shared interface and the same
//! holder range. Unverified — the LS-9000's command set is the starting point, not a promise.
//! Giving it a driver means confirming that on a wire and then flipping one arm of
//! `Model::protocol`; nothing in [`session`](crate::session) needs to change, since dispatch is
//! on the dialect.
//!
//! # What differs from the LS-9000
//!
//! - **14-bit samples** rather than 16, so the window descriptor's bits-per-pixel byte differs and
//!   the reported `max_bits` should be trusted over any constant.
//! - **No Kodachrome infrared profile.** Both take a plain infrared plane.
//!
//! Everything else in the owner's tables matches: 4000 DPI optical, multi-sample at 1/2/4/8/16,
//! and the single-line versus three-line CCD mode that only these two bodies have.
//!
//! # Holders
//!
//! The full medium-format range — FH-835M, FH-835S, FH-869S, FH-869G, FH-869GR, FH-869M, FH-816
//! and FH-8G1 — named in [`Adapter`](crate::adapter::Adapter). The FH-869GR is the one holder with
//! no overview pass and no batch scanning, and the FH-869S and FH-869G are the two whose frames
//! are not mechanically fixed.
//!
//! The detection scheme is the medium-format one: VPD page 0xC8, a present flag at byte 0 and a
//! class byte at byte 3. Whether the LS-8000 uses the same class numbering as the LS-9000 is
//! exactly the sort of thing that needs a capture rather than an assumption.
