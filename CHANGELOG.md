# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-08-01

### Added

- Adapter and model vocabulary for all six Coolscans; every scanner Nikon Scan
  supported is now recognized by name (`nkscan --list` reports an attached
  8000, 4000 or IV as undriven rather than staying silent about it).
- `Session::sensed_frames()`, so a host can place frames from what the
  transport reports instead of assuming a pitch; `prepare()` takes
  `offsets_mm` as a sequence, superseding `offset_mm`.
- The overview pass, previously computed and thrown away, is now exposed.
- Support for the SA-30 holder on the LS-50.

### Changed

- Frame placement is picked from what the scanner can actually do rather than
  from absent arguments.
- Refusals are machine-readable: `UnsupportedError` now carries `feature`,
  `reason`, `allowed` and `asked` alongside the message, so callers branch on
  `err.reason == "not_implemented"` instead of on the wording.
- Capabilities are computed from the model and the loaded adapter instead of
  being declared per driver.
- The five options nothing had ever checked are now gated and enforced.
- `nikon::capabilities::Capabilities` was renamed to `nikon::limits::DeviceLimits`.
- The pixel-depth width list was dropped; the bit depth comes off the device
  rather than from a table.

### Fixed

- The LS-5000 sends SCAN with a zero control byte and is no longer "totally
  untested" in the README (only a strip scan is proven; the roll transport,
  multi-sample readout and metering are still inference).
- Page 0xC8 byte 4 is no longer described as an aperture count.

### Removed

- `ResolvedPlacement`, a byte-for-byte copy of `Placement`.
- Three abstractions that were a bool in a costume.
- The pixel-depth width list (this library never downsamples).

## [0.1.0] - 2026-07-30

Initial release.
