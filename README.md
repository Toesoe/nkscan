# nkscan

A cross-platform, performant driver for Nikon film scanners.

## Background

Film photography enthusiasts know that in the world of home-scanning there really is only one name in the game, Nikon.
Nikon made many scanners over the years but the Coolscan 5000 / 9000 are the best of the best for 35mm and medium format, respecivley.
Unfortunately, these pieces of tech are vintage to say the least.
Most people buy a vintage Mac or Windows XP machine to run the official Nikon software or shell out money for the only modern cross-platform program Vuescan.

These options make me sad as a Linux user and a lover of free and open source software.
As such, I'm embarking on an adventure of reverse-engineering NikonScan and reimplementing every feature with full cross-platform support.
We'll start with a simple CLI that kinda emulates SANE and then move to a full GUI.
The intention is just to capture the highest-quality raw images for postprocessing elsewhere and not to reimplement Nikon's color science (yet).

## License

Dual licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed
as above, without any additional terms or conditions.
