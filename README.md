# nkscan

A cross-platform and performant library and command line application for Nikon film scanners.

## Usage

The `nkscan` binary drives both supported scanners, the Coolscan 9000 ED (SCSI) and the
Coolscan V ED / LS-50 (USB). It is behind a feature flag, so a library consumer does not pull
in the CLI stack:

``` bash
cargo install --path . --features cli
```

``` bash
Usage: nkscan [OPTIONS]

Options:
      --device <DEVICE>            Device path, skipping the search. `/dev/sg*` on Linux, `\\.\Scanner0` on Windows
      --frames <FRAMES>            Frames on the loaded strip
      --frame <FRAME>              Which of those to actually scan, zero-indexed, comma separated. All by default
      --dpi <DPI>                  Resolution in DPI. One of the firmware's divisions of the sensor's native pitch
      --gain <R,G,B[,IR]>          Fixed per-channel analog gain as `red,green,blue[,ir]`, which turns autoexposure off
      --focus <FOCUS>              Focus: `auto` to let the scanner find it on each frame, or a fixed setpoint [default: auto]
      --ir                         Capture the infrared plane for dust removal
      --basename <BASENAME>        Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its infrared mask <basename>_<n>_ir.tiff [default: scan]
      --offset <OFFSET>            Where each frame starts along the feed, in mm, comma separated, last value repeating
      --pitch <PITCH>              Frame pitch in mm, overriding what the scanner would use
      --lock-wb                    Hold the white balance during autoexposure, so the film keeps its cast. LS-9000 only
      --multisample <MULTISAMPLE>  Multisampling, trading scan time for noise. One of 1,2,4,8,16. LS-9000 only [default: 1]
      --singleline                 Single-line CCD mode. Slow, but may improve banding. LS-9000 only
      --eject                      Send the film back out when everything is done
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version
```

Without `--device` it looks for a scanner itself: USB devices by their ids, SCSI ones by
INQUIRY across the platform's device paths. One found is used, none or several is an error
naming `--device`. Model-specific flags are refused by name when the attached scanner has no
such knob, rather than being accepted and ignored.

Frames are found by an 83-DPI overview pass on the 9000. Giving `--pitch` or `--offset` places
them arithmetically instead, on either scanner, which is the way round a strip the search
misreads. The LS-50 always places them arithmetically, since it has no overview pass.

Output is 16-bit linear TIFF with an embedded linear-gamma ICC profile. The scanner's ADC is
linear and nothing applies a transfer curve, so an untagged file looks far too dark.

On Windows the scanner is reached through the scanner class driver, and opening that device
path needs an elevated prompt. Without one it fails to open at all rather than reporting a
denial.

### Scanning a roll under one exposure

Autoexposure meters every frame on its own, which is what you want for a single frame and not
what you want across a roll: two strips of the same film come back matched to their own
contents rather than to each other. For roll analysis, meter once and reuse the result.

``` bash
# Meter one representative frame and note the gains it logs
nkscan --frames 3 --frame 0

# ... INFO nkscan: Metered frame=0 gain=ChannelExposures { red: 679831, green: 487244, ... }

# Scan everything else at exactly that exposure
nkscan --frames 3 --gain 679831,487244,400117
```

`--gain` turns autoexposure off entirely. Autofocus still runs per frame, since film does not
sit flat and focus is not a property of the roll the way gain is. The infrared value is
optional, and the LS-50 takes three values rather than four because it drives infrared off a
zeroed gain.

## TODO

- Exercise the driver for the 9000 to make sure the shapes of data match what the scanner expects
- Run through many test scans to make sure the result looks ok
- Fix frame detection. It's not great right now, I might just make a little popup GUI for selecting each frame.
- Create PyO3 wrapper to connect up to NegPy
- Add support for other Nikon scanners
- Expand backend support

## License

Dual licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed
as above, without any additional terms or conditions.
