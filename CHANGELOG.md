# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0]

A complete and total rewrite.
Nothing of the 0.2.0 API survives, so the entry below it describes a crate that no longer exists rather than anything you can still call.
The driver now reads what a unit advertises and works from that, instead of carrying a table of what each model and holder is supposed to be able to do.

### Added

- Four layers with the boundaries actually enforced: `transport` moves SCSI
  bytes, `protocol` is types and parsing with no IO, `session` holds an open
  unit and the state that outlives one command, `scan` decides what a pass
  should do.
- Windows support, through the `scsiscan.sys` class driver, alongside the Linux
  sg path and the nusb USB path that covers all three platforms.
- Frame detection from a whole-strip thumbnail, for the holders that publish
  rectangles with no lengths. The film format is the caller's to supply
  (`--format`); nothing on the wire carries it.
- Host-side metering, for the units that run no AE pass of their own. Scales
  per-channel exposure to the ADC ceiling and takes another pass only when a
  channel came back clipped.
- CCD row-response correction from `DataType::CcdData`, which is the banding a
  three-line pass has and a single-line one does not.
- Autofocus per frame, and the focus position read back so a focus is
  repeatable without focusing again.
- Nikon's own scanner profiles, one per model per film type, embedded and
  converted to take the linear samples a pass produces. See
  `profiles/README.md`: they are not covered by this crate's license.
- `nkscan scan` batches a holder at a time and prompts for the next, writing
  16-bit TIFF with the infrared mask in a file of its own.
- Release binaries for macOS, both Apple Silicon and Intel, alongside the Linux
  and Windows ones. The macOS ones are unsigned, so Gatekeeper quarantines a
  downloaded one until `xattr -d com.apple.quarantine` clears it.
- CI cross-checks every target `rust-toolchain.toml` names. Only the host was
  ever compiled before, so `src/transport/windows.rs` first met a compiler when
  a release tag reached the Windows job and failed there, and the macOS side had
  gone the same way.
- A release asserts its own portability before it uploads anything: statically
  linked on Linux, no VC++ runtime imports on Windows, and no dylib outside the
  system paths on macOS.

### Changed

- Releases are built by GitHub Actions on each platform's own runner, and CI
  runs cargo directly, so neither goes through Nix. The flake stays for devshells
  and to provide binaries as a flake.
- The Linux and Windows binaries are static. musl links the whole libc in and
  `+crt-static` folds in the MSVC runtime, so neither has a system library to
  match on the machine that runs it.

- Sense data is read as what to do next rather than as an error, so polling,
  unit attentions and the vendor cooperation handshake are absorbed by the
  retry loop instead of surfacing to callers.
- A pass is decoded as it streams, into a buffer the caller owns, rather than
  after a scan-sized read.

### Fixed

- The published Linux binary had its ELF interpreter baked to a `/nix/store`
  glibc, so it ran on a Nix machine and nowhere else, failing with "No such file
  or directory" for a file that was plainly there.

### Removed

- The Python bindings, temporarily. There is no `#[pymodule]` on this branch;
  the `python` feature and `pyproject.toml` are kept for when they come back,
  and the jobs that would build and publish a wheel are commented out in both
  workflows.

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
