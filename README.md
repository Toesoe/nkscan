# nkscan

A cross-platform, performant driver for Nikon film scanners.

## Background

Film photography enthusiasts know that in the world of home-scanning there really is only one name in the game, Nikon.
Nikon made many scanners over the years but the Coolscan 5000 / 9000 are the best of the best for 35mm and medium format, respectively.
Unfortunately these pieces of tech are vintage, to say the least.
Most people buy a vintage Mac or Windows XP machine to run the official Nikon software, or shell out for Vuescan, the only modern cross-platform program.

These options make me sad as a Linux user and a lover of free and open source software.
As such, I'm embarking on an adventure of reverse-engineering NikonScan and reimplementing every feature with full cross-platform support.
We'll start with a simple CLI that kinda emulates SANE and then move to a full GUI.
The intention is just to capture the highest-quality raw images for postprocessing elsewhere and not to reimplement Nikon's color science (yet).

## Usage

Right now, only a library as we figure out the shapes of the data to make it easy to slot in support for other scanners/OSes/backends.
However, a full scanner flow is working for the Coolscan 9000 in the ls9k_cli example on linux.

``` bash
Usage: ls9k_cli [OPTIONS] --frames <FRAMES> <SCANNER>

Arguments:
  <SCANNER>  Linux /dev/sg* for the scanner

Options:
      --frames <FRAMES>            How many frames to expect in the film holder (needed for frame recognition)
      --lock-wb                    Whether to lock the white balance during autoexposure
      --frame <FRAME>              Optional frame number (zero-indexed) to scan, otherwise scan all of them
      --ir                         Save IR alongside the main scan
      --basename <BASENAME>        Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its infrared mask <basename>_<n>_ir.tiff [default: scan]
      --multisample <MULTISAMPLE>  How much multisampling to perform. This increases scan time at the befenit of lower noise. One of 1,2,4,8,16 [default: 1]
      --singleline                 Single-line CCD mode. Slow, but may improve banding noise
  -h, --help                       Print help
  -V, --version                    Print version
```

## License

Dual licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed
as above, without any additional terms or conditions.
