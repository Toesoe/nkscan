# nkscan

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/activexray/nkscan/ci.yml)


A cross-platform and performant library and command line application for Nikon film scanners

## Support

I only own a Coolscan 9000, but through reverse-engineering of the Nikon binaries and the work of others, we are slowly adding support for more scanners.
Please reach out if you have a scanner we can test! Even just dumps of USB/SCSI payloads of a normal scan would be incredibly useful.

Our goal is to support all the scanners supported by Nikon Scan, which are enumerated here

- ✅ Supported, and run against the hardware
- ⚠️ Untested but theoretically should work
- ❌ Not started

Every scanner Nikon Scan supported is recognized by name, including the ones with no driver:
`nkscan --list` will report an attached 8000, 4000 or IV and say it has no driver rather than
staying silent about it.

### Medium Format Scanners

| Scanner \ Holder | 835M | 835S | 869S  | 869G  | 869GR  | 869M | 816 | 8G1 |
|------------------|:----:|:----:|:-----:|:-----:|:------:|:----:|:---:|:---:|
| 9000             | ❌   | ❌   | ✅    | ⚠️   | ⚠️     | ❌  | ❌  | ❌ |
| 8000             | ❌   | ❌   | ❌    | ❌   | ❌     | ❌  | ❌  | ❌ |

### 35mm Scanners

| Scanner \ Holder | SA-21  | IA-20/21  | MA-20/21   | SA-30  | SF-210/200  |
|------------------|:------:|:---------:|:----------:|:------:|:-----------:|
| 5000             | ✅     | ❌        | ⚠️         | ⚠️     | ❌         |
| 4000             | ❌     | ❌        | ❌         | ❌     | ❌         |
| V                | ✅     | ❌        | ⚠️         | ✅     | ❌         |
| IV               | ❌     | ❌        | ❌         | ❌     | ❌         |

MacOS Firewire support is planned, but just stubbed out for the moment. It should work for USB scanners, like the Coolscan V.

### Caveat Emptor

While the 9000 has the best support right now, there are still some gaps in capability based on what I have.

- The Coolscan 9000 workflow assumes a medium format strip film holder. The frame detection algorithm will not work for others, but you can manually place the frames for the time being. If someone has the 35 holder or others, please reach out and we can add the missing logic.
- The Coolscan V workflow assumes a strip holder, not a full roll holder. I need more dumps of payloads/RE work to understand the typical flow for those (or the other inserts like the bulk slide loader)
- On a roll adapter, leave `--frames` out. Without it the scanner is asked how many it sensed,
  which is what you want on a roll; giving a count overrides that. The SA-30's frame *count*
  works, but the read that reports where those frames actually are is refused, so they are
  placed at an even pitch rather than where the transport found them. See
  `docs/OPEN_QUESTIONS.md` section 19.
- The Coolscan 5000 has now scanned with an SA-21, on Windows. The roll transport, the
  multi-sample readout and the metering path are all still inference, so treat anything past a
  strip scan as unproven.
- Which film holder a medium format body has loaded cannot yet be identified exactly. The
  scanner reports a holder *class* rather than a part number, and a class does not say whether
  it is an FH-869S, an FH-869G or a 35 mm carrier, so capabilities fall back to what every
  holder in the family can do. See `docs/HOLDERS.md`.

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

### What the flags do, per adapter

Most of what a scanner will do depends on what is loaded in it, not on which model it is: the
same body ejects a strip from one adapter and swaps a slide in another. `nkscan` reads the
adapter on open and prints it, and refuses anything it cannot do rather than doing something
else quietly.

`--frames` says how many frames to **place**. `--frame` says which of those to **scan**. They are
easy to confuse and do different things.

| Adapter | Where frames come from | `--frames` | `--pitch` / `--offset` | `--eject` |
|---|---|---|---|---|
| SA-21 (strip) | six fixed apertures | optional, caps how many | shifts film within them | pushes the strip out |
| SA-30 (roll) | the transport senses them | **leave it out**; a count takes only the first that many | shifts film | pushes the film out |
| MA-21 (mount) | one slide, by hand | ignored, there is one | no | nothing to eject |
| SF-210 (feeder) | one slide at a time | ignored | no | returns it and feeds the next |
| IA-20/21 (IX240) | the transport senses them | leave it out | no | rewinds the cartridge |
| FH-869S / FH-869G | found in an overview pass | how many to look for | shifts film | ejects the holder |
| FH-869GR | no overview pass, so placed by hand | how many | no | ejects the holder |
| other FH-8xx holders | found in an overview pass | how many to look for | no | ejects the holder |

Leaving `--frames` out asks the scanner. On an adapter that senses frames that is the right
answer and the whole point of the adapter; on one that senses nothing you have to give either
`--frames` or `--pitch`, or there is nothing to place.

These are the settings that depend on the **model** rather than the adapter:

| Flag | Where it applies |
|---|---|
| `--multisample` | 8000, 9000, 5000, 4000. The V and IV average nothing in hardware. |
| `--singleline` | 8000 and 9000 only, the two with a selectable readout |
| `--lock-wb` | anywhere the host meters. Not the V or the IV, whose firmware meters and gives nothing to hold. |
| `--dpi` | a division of that model's sensor: 4000 on five of them, 2900 on the IV |

Asking for something a scanner does not have is refused by name, so `--singleline` on a V says so
rather than being ignored.

**On a medium format body the exact holder cannot yet be identified.** The scanner reports a
holder *class*, not a part number, so the rows above for FH-869GR in particular are not reachable
today -- capabilities fall back to what every holder in that family can do. See
`docs/HOLDERS.md`.


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

`session.overview()` takes the low-resolution pass Nikon Scan calls a thumbnail, returning the
whole strip in one image alongside the resolution it ran at, which is what maps a point on the
thumbnail back to a position on the film.

`session.capabilities` answers for the model *and the loaded adapter*, since most of what a
scanner will do depends on what is in it: ejecting returns a holder on one adapter and rewinds a
cartridge on another. Anything the scanner will not do raises `nkscan.UnsupportedError`, which
carries `feature` and `reason` so a caller can tell "this scanner cannot" (`not_present`) from
"this library does not yet" (`not_implemented`) without reading the message.

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
- [openICE](https://github.com/a6o/openICE)
- [digital fauxice](https://github.com/rohanpandula/digital-fauxice)
