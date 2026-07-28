# nkscan

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/activexray/nkscan/ci.yml)


A cross-platform and performant library and command line application for Nikon film scanners

## Support

I only own a Coolscan 9000, but through reverse-engineering of the Nikon binaries and the work of others, we are slowly adding support for more scanners.
Please reach out if you have a scanner we can test! Even just dumps of USB/SCSI payloads of a normal scan would be incredibly useful.

|                            | Linux    | Windows  |
|----------------------------|----------|----------|
| LS-9000 ED (Coolscan 9000) | Tested   | Tested   |
| LS-50 (Coolscan V)         | Tested   | Untested |
| LS-5000 (Coolscan 5000)    | Untested | Untested |

MacOS support is planned, but just stubbed out for the moment.

## Usage

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
      --lock-wb                    Hold the white balance during autoexposure, so the film keeps its cast. Not on the LS-50
      --multisample <MULTISAMPLE>  Multisampling, trading scan time for noise. One of 1,2,4,8,16. LS-9000 only [default: 1]
      --singleline                 Single-line CCD mode. Slow, but may improve banding. LS-9000 only
      --eject                      Send the film back out when everything is done
      --batch                      Scan every strip of the roll, ejecting and pausing to reload in between
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version
```

## Example Workflow

### Raw Negatives for External Inversion

Say I have a LS-9000 and I invert with either [NegPy](https://github.com/marcinz606/NegPy) or [Negative Lab Pro](https://www.negativelabpro.com/).
For this to work well, I want essentially the raw scans out of the scanner, but with equal exposure across all frames so I can perform "roll analysis".

So, I load up the film holder with the first strip (in my case 6x6 negatives, with three frames), and scan in "batch mode".
For this, I tell the program how many frames per strip, include the IR pass, lock the whitebalance during autoexposure of the first frame (which will be applied to every farme), and enable 2x multisampling.

```
nkscan --frames 3 --ir --lock-wb --multisample 2 --batch
```

Personally, I find this much faster than anything I could do in NikonScan or Vuescan.

## License

Dual licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed
as above, without any additional terms or conditions.

## Related Projects and References

- [coolscanpy](https://github.com/rohanpandula/coolscanpy/)
- [Coolscan RE](https://github.com/kevihiiin/Nikon-Coolscan-RE)
- [coolscan-mods](https://github.com/kosma/coolscan-mods)
- [sane-coolscan3](http://sane-project.org/man/sane-coolscan3.5.html)
