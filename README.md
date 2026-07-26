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
Usage: ls9k_cli [OPTIONS] <SCANNER> <COMMAND>

Commands:
  scan       Scan every frame from the loaded strip at 4000 DPI
  calibrate  Measure the backlight with an EMPTY holder loaded, and write the neutral gains

Arguments:
  <SCANNER>  Linux /dev/sg* for the scanner

Options:
      --wb-file <WB_FILE>  Where the bare-light white balance lives, as `calibrate` writes it and `scan` reads it [default: nkscan-wb.txt]
  -h, --help               Print help
  -V, --version            Print version
```

``` bash
Usage: ls9k_cli <SCANNER> scan [OPTIONS] --frames <FRAMES>

Options:
      --frames <FRAMES>            How many frames to expect in the film holder (needed for frame recognition)
      --lock-wb                    Whether to lock the white balance during autoexposure
      --frame <FRAME>              Optional frame number (zero-indexed) to scan, otherwise scan all of them
      --ir                         Save IR alongside the main scan
      --basename <BASENAME>        Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its infrared mask <basename>_<n>_ir.tiff [default: scan]
      --multisample <MULTISAMPLE>  How much multisampling to perform. This increases scan time at the befenit of lower noise. One of 1,2,4,8,16 [default: 1]
      --singleline                 Single-line CCD mode. Slow, but may improve banding noise
  -h, --help                       Print help
```

### White balance

Equal analog gain on every channel does not scan neutral: the LEDs and the CCD are not equally strong across the three bands, and red needs about 1.7x what blue does to match it.
One scanner's measurement is built in as the default, and it agrees with the gains a Nikon Scan capture staged to within 0.7%, so it is the hardware rather than any particular unit.

To measure your own, run `calibrate` once with the holder loaded and **empty**.
It meters the bare backlight and writes the gains that even the three channels out, and `scan` picks them up as the starting point for every frame's autoexposure.
Bare light is the brightest thing the scanner can ever see, so those gains also double as a ceiling: no film clips at them, and every metering pass starts from somewhere informative.

This matters most under `--lock-wb`, which scales all three channels by a single factor, so it preserves the film's own cast (what you want for slides, or for a negative you intend to invert yourself) and cannot correct the scanner's.
Without a calibration, that lock keeps the scanner's imbalance too.

It is a white point and nothing more.
The three LED bands are narrow, so there is no color temperature to hit, and real colorimetry needs a 3x3 from an IT8 target rather than three gains.

## TODO

- Exercise the driver for the 9000 to make sure the shapes of data match what the scanner expects
- Run through many test scans to make sure the result looks ok
- Fix frame detection. It's not great right now, I might just make a little popup GUI for selecting each frame.
- Create PyO3 wrapper to connect up to NegPy
- Add support for other Nikon scanners

## License

Dual licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed
as above, without any additional terms or conditions.
