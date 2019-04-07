# blockyarchive

[![Build Status](https://travis-ci.org/darrenldl/blockyarchive.svg?branch=master)](https://travis-ci.org/darrenldl/blockyarchive)
[![Build status](https://ci.appveyor.com/api/projects/status/i4dxpldp4t312gtv?svg=true)](https://ci.appveyor.com/project/darrenldl/blockyarchive)
[![codecov](https://codecov.io/gh/darrenldl/blockyarchive/branch/master/graph/badge.svg)](https://codecov.io/gh/darrenldl/blockyarchive)
[![Coverage Status](https://coveralls.io/repos/github/darrenldl/blockyarchive/badge.svg?branch=master)](https://coveralls.io/github/darrenldl/blockyarchive?branch=master)
[![Crates](https://img.shields.io/crates/v/blkar.svg)](https://crates.io/crates/blkar)
[![dependency status](https://deps.rs/repo/github/darrenldl/blockyarchive/status.svg)](https://deps.rs/repo/github/darrenldl/blockyarchive)
[![Gitter chat](https://badges.gitter.im/blockyarchive/gitter.png)](https://gitter.im/blockyarchive/community)

[Documentation](https://github.com/darrenldl/blockyarchive/wiki)

Blockyarchive/blkar (pronounced "bloc-kar") is a comprehensive utility for creating, rescuing, and general handling of SeqBox archives, with optional forward error correction

SeqBox is a single-file archive format designed by [Marco Pontello](https://github.com/MarcoPon) that facilitates sector level data recovery for when file system metadata is corrupted/missing, while the archive itself still exists as a normal file on file system

Please visit the official [SeqBox](https://github.com/MarcoPon/SeqBox) repo for the original implementation and technical details on this

Blockyarchive/blkar was formerly known as rust-SeqBox/rsbx prior to renaming

## Comparison to the original SeqBox implementation/design

The original SeqBox implementation and format do not support repairing of data, only sector level recoverability

Blockyarchive allows repairs to be made by adding forward error correction (Reed-Solomon erasure code) to extended versions of SeqBox format, and also allows arranging the blocks in a burst error resistant pattern

Blockyarchive is also more robust compared to the original SeqBox implementation, as it does not assume the SBX container to be well formed, and makes as few assumptions about the SBX container as possible

blkar is overall based around [osbx](https://github.com/darrenldl/ocaml-SeqBox), but much more optimized

## Features overall

- Data recovery that does not depend on file system metadata (sector level recovery)
  - This allows data recovery even when data is fragmented and out of order
- Supports error correction (via Reed-Solomon erasure code)
- Supports burst (sector) error resistance
- JSON mode
  - Outputs information in JSON format instead of human readable text, easy integration with scripts

## Limitations

- Only a single file is supported as SeqBox is a single-file archive format
  - However, blkar may still be usable when you have multiple files as you can pipe stdout of other archivers that support bundling multiple files, like tar, into blkar's stdin during encoding, and blkar can also pipe data out to stdout during decoding

## Goals

As blkar is to be used largely as a backup utility, security/robustness of the code will be prioritised over apparent performance

## Status

This project has reached its intended feature completeness, so no active development for new features will occur. However, this project is still actively looked after, i.e. I will respond to PRs, issues, and emails, will consider feature requests, respond to bug reports quickly, and so on.

In other words, this is a completed project with respect to its original scope, but it is not abandoned

## Getting started

#### Installation

`blkar` is available via [GitHub releases](https://github.com/darrenldl/blockyarchive/releases) or via `cargo`

```
cargo install blkar
```

#### Usage guides & screencasts & other resources

The [wiki](https://github.com/darrenldl/blockyarchive/wiki) contains comprehensive guides and resources

## Note on Rust to Bash ratio

Just to avoid confusion, blkar is written purely in Rust, Bash is only used to write tests

## Got a question?

Feel free to join the [Gitter chat](https://gitter.im/blockyarchive/community) if you've got a question. You can email me directly as well.

## Changelog

[Changelog](CHANGELOG.md)

## Specifications

[SBX format](SBX_FORMAT.md)

[blkar specs](BLKAR_SPECS.md)

## Contributions

Contributions are welcome. Note that by submitting contributions, you agree to license your work under the same license used by this project (MIT).

## Acknowledgement

I would like to thank [Marco](https://github.com/MarcoPon) (the official SeqBox author) for discussing and clarifying aspects of his project, and also providing of test data during development of osbx. I would also like to thank him for his feedback on the numbering of the error correction enabled SBX versions (versions 17, 18, 19).

I would like to thank [Ming](https://github.com/mdchia/) for his feedback on the documentation, UX design, and several other general aspects of the osbx project, of which most of the designs are carried over to blkar, and also his further feedback on this project as well

The design of the readable rate in progress report text is copied from [Arch Linux pacman](https://wiki.archlinux.org/index.php/Pacman)'s progress bar design

The design of block set interleaving arrangement in RS enabled versions is heavily inspired by [Thanassis Tsiodras's design of RockFAT](https://www.thanassis.space/RockFAT.html). The interleaving provides resistance against burst sector errors.

## Donation

**Note** : Donation will **NOT** fuel development of new features. As mentioned above, this project is meant to be stable, well tested and well maintained, but normally I am not actively adding new features to it.

If blockyarchive has been useful to you, and you would like to donate to me for the development effort, you can donate through [here](http://ko-fi.com/darrenldl).

## License

#### Libcrc code

The crcccitt code is translated from the C implementation in [libcrc](https://github.com/lammertb/libcrc) and are under the same MIT License as used by libcrc and as stated in libcrc source code, the license text of the crcccitt.c is copied over to `crc-ccitt/build.rs`, `crc-ccitt/src/lib.rs`, `build.rs` and `src/crc_ccitt.rs` as well

#### Official SeqBox code

The following files in tests folder copied from official SeqBox are under its license, which is MIT as of time of writing

- tests/SeqBox/*

All remaining files are distributed under the MIT license as stated in the LICENSE file
