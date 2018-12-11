# rust-SeqBox

[![Build Status](https://travis-ci.org/darrenldl/rust-SeqBox.svg?branch=master)](https://travis-ci.org/darrenldl/rust-SeqBox)
[![Build status](https://ci.appveyor.com/api/projects/status/ho6v99qysi9l8p6d?svg=true)](https://ci.appveyor.com/project/darrenldl/rust-seqbox)
[![codecov](https://codecov.io/gh/darrenldl/rust-SeqBox/branch/master/graph/badge.svg)](https://codecov.io/gh/darrenldl/rust-SeqBox)
[![Coverage Status](https://coveralls.io/repos/github/darrenldl/rust-SeqBox/badge.svg?branch=master)](https://coveralls.io/github/darrenldl/rust-SeqBox?branch=master)
[![Crates](https://img.shields.io/crates/v/rsbx.svg)](https://crates.io/crates/rsbx)
[![dependency status](https://deps.rs/repo/github/darrenldl/rsbx/status.svg)](https://deps.rs/repo/github/darrenldl/rsbx)

[Documentation](https://github.com/darrenldl/rust-SeqBox/wiki)

Enhanced implementation of SeqBox in Rust

SeqBox is a single-file archive format designed by [Marco Pontello](https://github.com/MarcoPon) that facilitates sector level data recovery for when file system metadata is corrupted/missing, while the archive itself still exists as a normal file on file system.

Please visit the official [SeqBox](https://github.com/MarcoPon/SeqBox) repo for technical details on this.

## Enhancements

This implementation adds forward error correction on top of the SeqBox format by adding support for Reed-Solomon erasure code, and also allows arranging the blocks such that burst sector errors can be tolerated.

rsbx is overall based around [osbx](https://github.com/darrenldl/ocaml-SeqBox), but much more optimized.

## Features overall

- Data recovery that does not depend on file system metadata (sector level recovery)
  - This allows data recovery even when data is fragmented and out of order
- Supports error correction (via Reed-Solomon erasure code)
- Supports burst sector error resistance
- JSON mode
  - Output information in JSON format instead of human readable text

## Goals

As rsbx is to be used largely as a backup utility, security/robustness of the code will be prioritised over apparent performance.

## Getting started

#### Installation

rsbx is available via [GitHub releases](https://github.com/darrenldl/rust-SeqBox/releases) or via `cargo`

```
cargo install rsbx
```

#### Usage guides & screencasts & other resources

The [wiki](https://github.com/darrenldl/rust-SeqBox/wiki) contains comprehensive guides and resources.

## Helpers

Repo [rsbx-helpers](https://github.com/darrenldl/rsbx-helpers) contains helpers for extending the functionality of rsbx.

## Changelog

[Changelog](CHANGELOG.md)

## Specifications

[SBX format](SBX_FORMAT.md)

[rsbx specs](RSBX_SPECS.md)

## Contributions

Contributions are welcome. Note that by submitting contributions, you agree to license your work under the same license used by this project (MIT).

## Acknowledgement

I would like to thank [Marco](https://github.com/MarcoPon)(official SeqBox author) for discussing and clarifying aspects of his project, and also providing of test data during development of osbx. I would also like to thank him for his feedback on the numbering of the error correction enabled SBX versions (version 17, 18, 19).

I would like to thank [Ming](https://github.com/mdchia/) for his feedback on the documentation, UX design, and several other general aspects of the osbx project, of which most of the designs are carried over to rsbx, and also his further feedback on this project as well.

The design of the readable rate in progress report text is copied from [Arch Linux pacman](https://wiki.archlinux.org/index.php/Pacman)'s progress bar design.

The design of block set interleaving arrangement in RS enabled versions is heavily inspired by [Thanassis Tsiodras's design of RockFAT](https://www.thanassis.space/RockFAT.html). The interleaving provides resistance against burst sector errors.

## License

#### Libcrc code

The crcccitt code is translated from the C implementation in [libcrc](https://github.com/lammertb/libcrc) and are under the same MIT License as used by libcrc and as stated in libcrc source code, the license text of the crcccitt.c is copied over to `crcccitt/build.rs`, `crcccitt/src/lib.rs`, `build.rs` and `src/crc_ccitt.rs` as well

The C source code of crcccitt copied directly from libcrc are under the MIT License as used by libcrc, the files are in ```libcrc_crcccitt```

#### Official SeqBox code

The files in tests folder copied from official SeqBox are under its license, which is MIT as of time of writing

- tests/SeqBox/*

All remaining files are distributed under the MIT license as stated in the LICENSE file
