# nkscan

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/activexray/nkscan/ci.yml)


A cross-platform and performant library and command line application for Nikon film scanners

## Support

I only own a Coolscan 9000, but through reverse-engineering of the Nikon binaries and the work of others, we are slowly adding support for more scanners.
Please reach out if you have a scanner we can test! Even just dumps of USB/SCSI payloads of a normal scan would be incredibly useful.

Our goal is to support all the scanners supported by Nikon Scan, which are enumerated here

### Medium Format Scanners

|                  | Status        |
|------------------|---------------|
| Coolscan 9000    | Supported     |
| Coolscan 8000    | Not started   |

### 35mm Scanners

|               | Status      |
|---------------|-------------|
| Coolscan 5000 | Untested    |
| Coolscan 4000 | Not started |
| Coolscan V    | Supported   |
| Coolscan IV   | Not started |

MacOS Firewire support is planned, but just stubbed out for the moment. It should work for USB scanners, like the Coolscan V.

### Caveat Emptor

While the 9000 has the best support right now, there are still some gaps in capability based on what I have.

- The Coolscan 9000 workflow assumes a medium format strip film holder. The frame detection algorithm will not work for others, but you can manually place the frames for the time being. If someone has the 35 holder or others, please reach out and we can add the missing logic.
- The Coolscan V workflow assumes a strip holder, not a full roll holder. I need more dumps of payloads/RE work to understand the typical flow for those (or the other inserts like the bulk slide loader)
- The Coolscan 5000 path is totally untested. Someone should try it ;)

If you would like to contribute support for new scanners, please take a look at scsi_proxy/README.md

## Usage

```
Usage: nkscan [OPTIONS]

Options:
      --device <DEVICE>            Which scanner to use, as `--list` reports it. Only needed when more than one is attached
      --list                       List the scanners attached and exit, without touching any of them
      --frames <FRAMES>            Frames on the loaded strip
      --frame <FRAME>              Which frames to actually scan, zero-indexed and comma separated. All of them by default
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

## Python

We also ship the driver as a Python extension, which simplifies use in from Python-based scanning/inversion programs.
The image data comes back as zero-copy numpy arrays in linear 16-bit ADC counts.

```
pip install nkscan
```

```python
import nkscan

device = nkscan.list_devices()[0]
print(device.id, device.model, device.capabilities.dpi)

with nkscan.Session(device.id) as session:
    # Three 6x6 frames at a 56 mm pitch, holding one exposure across all of them
    session.prepare(frames=3, pitch_mm=56.0, gain=(283048, 202864, 166589))

    for frame in range(3):
        result = session.scan(frame, dpi=2000, ir=True,
                              progress=lambda read, total: print(f"\r{read}/{total}", end=""))
        result.rgb          # (height, width, 3) uint16
        result.ir           # (height, width) uint16, or None
    session.eject()
```

The scan may take minutes, so a progress callback can drive a UI and returning `False` from it stops the scan.
Failures worth retrying share a `nkscan.TransientError` base, so one `except` covers a link glitch or a short read without swallowing a real problem like `nkscan.MediaError`.

Building it from source needs [maturin](https://www.maturin.rs/); the dev shell in `flake.nix` has it, along with a Python carrying numpy.

```
maturin develop
```

The type stub is generated rather than written, so it cannot fall out of step with the bindings:

```
cargo run --features python --bin stub_gen
```

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
