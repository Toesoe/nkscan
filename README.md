# nkscan

A cross-platform, performant driver for Nikon film scanners.

## Usage

Right now, only a library as we figure out the shapes of the data to make it easy to slot in support for other scanners/OSes/backends.
However, a full scanner flow is working for the Coolscan 9000 in the ls9k_cli example on linux, and for the Coolscan V ED (LS-50) in the ls50_cli example.

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

The LS-50 is USB rather than SCSI, so it takes no device path: the example finds the scanner by
its USB ids. Frames are placed by the adapter's reported pitch plus your own per-frame `--offset`
correction, since there is no overview pass to detect them from.

``` bash
Usage: ls50_cli [OPTIONS]

Options:
      --basename <BASENAME>  Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its infrared mask <basename>_<n>_ir.tiff [default: scan]
      --dpi <DPI>            Resolution in DPI. One of the firmware's divisions of the 4000-DPI sensor [default: 4000]
      --frames <FRAMES>      Frames on the loaded strip: 1 for a single frame, 0 to take the adapter's count [default: 1]
      --frame <FRAME>        Which of those frames to actually scan, zero-indexed, comma separated. All by default
      --ir                   Capture the infrared plane for dust removal
      --ae                   Run the autoexposure pre-pass
      --af                   Run firmware autofocus at the frame center
      --focus <FOCUS>        Fixed focus setpoint. Ignored with --af; without either the motor parks at 0
      --offset <OFFSET>      Where each frame starts along the feed axis, in mm, comma separated, last value repeating. One per frame, since the feed does not place them evenly [default: 0]
      --pitch <PITCH>        Override the frame pitch, in mm. Omitted, the adapter's reported pitch is used, which is what advances the film. Zero holds every window in one place
      --eject                Eject the film once the batch is done
  -h, --help                 Print help (see more with '--help')
  -V, --version              Print version
```


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
