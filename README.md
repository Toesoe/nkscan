# nkscan

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/activexray/nkscan/ci.yml)

A cross-platform and performant driver for Nikon film scanners.

## Usage

For the command-line tool, download a binary release or build from source and run!


### Example

Say I'm batch scanning 6x6 color negatives on my Coolscan 9000 (the only Nikon scanner attached to my computer).
I usually do 2x multisampling at the full native resolution with an IR pass.
Additionally, I'll "lock" the exposure from the first frame so every frame is exposed the same off the scanner so I can perform roll analysis when I invert.
To do this and scan my whole roll (with the program prompting between strips), I'd run

``` bash
nkscan scan --lock-ae --samples 2 --ir --format 66
```

### Options

```
Usage: nkscan scan [OPTIONS] [DEVICE]

Arguments:
  [DEVICE]
          The scanner to connect to. Optional, will default to the first found

Options:
      --basename <BASENAME>
          Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its infrared mask <basename>_<n>_IR.tiff
          
          [default: scan]

      --unlock-wb
          Autoexpose per channel. Better dynamic range, but no longer "calibrated"

      --lock-ae
          Autoexpose the first frame and reuse that exposure across all frames

      --dpi <DPI>
          Resolution. Defaults to scanner maximum

      --samples <SAMPLES>
          Number of samples. Defaults to 1
          
          [default: 1]

      --superfine
          Singleline CCD mode. Only supported on multiline CCD scanners

      --frames <FRAMES>
          Which frame(s) to scan, comma separated. Defaults to all detected. Naming any stops after one holder rather than batching

      --ir
          Include the IR pass

      --no-eject
          Don't eject at the end of the strip

      --format <FORMAT>
          Film format. One of: 135, 16, 645, 66, 67, 68, 69, or a custom frame length in mm. Defaults to what the holder reports (if any)

      --film <FILM>
          Film type, which picks the color profile the scans are tagged with

          Possible values:
          - positive:   Slide film
          - negative:   Color negative
          - kodachrome: Kodachrome, whose dyes need their own profile
          - mono:       Black and white negative
          
          [default: negative]

  -h, --help
          Print help (see a summary with '-h')
```

## Support

Our goal is to support all the scanners supported by Nikon Scan, which are enumerated here by testing status.
This library doesn't have anything scanner or adapter-specific so *theoretically* it should work across devices.

- ✅ Supported, and run against real hardware
- ⚠️ Untested but theoretically should work

### Medium Format Scanners

| Scanner \ Holder | 835M | 835S | 869S  | 869G  | 869GR  | 869M | 816 | 8G1 |
|------------------|:----:|:----:|:-----:|:-----:|:------:|:----:|:---:|:---:|
| 9000             | ⚠️  |  ⚠️   | ✅   | ⚠️   |  ⚠️   |  ⚠️ | ⚠️ |  ⚠️ |
| 8000             | ⚠️  |  ⚠️   | ⚠️   | ⚠️   |  ⚠️   |  ⚠️ | ⚠️ |  ⚠️ |

### 35mm Scanners

| Scanner \ Holder | SA-21  | IA-20/21  | MA-20/21   | SA-30  | SF-210/200  |
|------------------|:------:|:---------:|:----------:|:------:|:-----------:|
| 5000             |   ⚠️   |  ⚠️      |    ⚠️     |  ⚠️    |   ⚠️       |
| 4000             |   ⚠️   |  ⚠️      |    ⚠️     |  ⚠️    |   ⚠️       |
| V                |   ⚠️   |  ⚠️      |    ⚠️     |  ⚠️    |   ⚠️       |
| IV               |   ⚠️   |  ⚠️      |    ⚠️     |  ⚠️    |   ⚠️       |

If you want to use a Firewire scanner on an old Mac that still has OS support for FireWire, let me know and I can scope it out.
It is technically possible, but getting Rust to compile a binary for older MacOS is not something I have experience in.
You could also just like, install Linux on it :)

### USB Scanners

We use [nusb](https://github.com/kevinmehall/nusb), which is a pure-Rust alternative to libusb, but it carries the same invariants.
On Windows, this means you need to associate your device with a WinUSB driver.
The most popular way to do this is with [Zadig](https://zadig.akeo.ie/).

On Linux, make sure you have the appropriate udev rules set up. Nusb has some [help](https://docs.rs/nusb/latest/nusb/#linux) on this.

MacOS *should* just work.

## Design Notes

This library is written from the ground up following the official Nikon spec of the wire protocol for the LS-5000 and LS-9000 ED scanners (located in docs/).
Comparing the two, we find an identical protocol. Some types are absent in one but not the other, some lists capabilities the other doesn't have, but all of the bits and bytes are in the same position across all the data.
This implies we don't need any model or holder specifics, we can just read what the scanner advertises as its capabilities and work from there.
This means (hopefully) we can support every scanner and every holder with a single codebase (although please test and let me know)!

The code is broken down into several layers of independent abstractions
- Transport: Defines what moving SCSI bytes is for the different OSes and physical layer (USB/FireWire)
- Protocol: An implementation of the Nikon spec via serialization and deserialization of bytes as they come off the wire. This module does no IO and is just byte-oriented.
- Session: Combines a trait object of the Transport (type erasure) with the methods from Protocol. This wraps scanner state (like global units) and provides functions that essentially perform the spec's listed actions.
- Scan: Combine the methods from Session to perform high-level scan operations. This asks the scanner what it can do and then orders session operations to do it.

Please note that while LLMs helped with the production of this crate, it was largley written by hand and not vibe-coded.
If you contribute, please adhere to the [contribution guide](CONTRIBUTING.md).

## TODO

- Python bindings

### Algorithms

The scanner occasionally asks the host to perform actions that it does not do internally.
So far this has been:
- Frame detection in certain holders (like the 869S)
- Autoexposure
- CCD anti-banding

We have not reverse-engineered with Nikon scan does and have written our own versions of these algorithms.

## License

Dual licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Except for the ICC profiles in [profiles/](profiles/README.md), which are
derived from Nikon's and are not ours to license.

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
