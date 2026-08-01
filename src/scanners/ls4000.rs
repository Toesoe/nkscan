//! Coolscan 4000 ED — recognized, not driven
//!
//! A placeholder so that the model has somewhere to live and so what is known about it is
//! written down rather than rediscovered. Nothing here talks to a scanner.
//!
//! The LS-4000 is the LS-5000's predecessor: 35 mm through the same adapters, but over FireWire
//! 400 rather than USB 2.0, so enumeration finds it by INQUIRY string and it carries no USB ids.
//! It is the one model whose family and interface disagree — 35 mm like the USB bodies, FireWire
//! like the medium-format ones — which is why those are separate axes.
//!
//! # What it is believed to need
//!
//! `Protocol::Ls5000`, on the strength of the shared adapter range and the shared 4000 DPI
//! sensor. Unverified. The FireWire link is the reason to be careful: the LS-5000 driver reads
//! the image in 512-aligned 128 KiB chunks sized for a USB pipe, and nothing says a FireWire
//! unit wants the same.
//!
//! Note that the LS-5000 driver itself has never been run against hardware, so this would be a
//! guess resting on a guess.
//!
//! # What differs from the LS-5000
//!
//! - **14-bit samples** rather than 16.
//! - **No Kodachrome infrared profile.**
//!
//! Multi-sample at 1/2/4/8/16 is shared — the LS-4000 is the last 35 mm body to have it, and the
//! LS-50 and LS-40 below it have none.
//!
//! # Adapters
//!
//! The full 35 mm range: SA-21, SA-30, MA-21, SF-210 and the IX240 adapter, detected the same way
//! as on the LS-50 and LS-5000 — by which page codes VPD page 0x00 advertises.
