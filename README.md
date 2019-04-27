# blockyarchive

[![Build Status](https://travis-ci.org/darrenldl/blockyarchive.svg?branch=master)](https://travis-ci.org/darrenldl/blockyarchive)
[![Build status](https://ci.appveyor.com/api/projects/status/i4dxpldp4t312gtv?svg=true)](https://ci.appveyor.com/project/darrenldl/blockyarchive)
[![codecov](https://codecov.io/gh/darrenldl/blockyarchive/branch/master/graph/badge.svg)](https://codecov.io/gh/darrenldl/blockyarchive)
[![Coverage Status](https://coveralls.io/repos/github/darrenldl/blockyarchive/badge.svg?branch=master)](https://coveralls.io/github/darrenldl/blockyarchive?branch=master)
[![Crates](https://img.shields.io/crates/v/blkar.svg)](https://crates.io/crates/blkar)
[![dependency status](https://deps.rs/repo/github/darrenldl/blockyarchive/status.svg)](https://deps.rs/repo/github/darrenldl/blockyarchive)
[![Gitter chat](https://badges.gitter.im/blockyarchive/gitter.png)](https://gitter.im/blockyarchive/community)

[Documentation](https://github.com/darrenldl/blockyarchive/wiki)

Blockyarchive/blkar (pronounced "bloc-kar") is a comprehensive utility for creating, rescuing, and general handling of SeqBox archives, with optional forward error correction via Error-correcting SeqBox.

### Demo

[![asciicast](https://asciinema.org/a/240491.svg)](https://asciinema.org/a/240491)

#### What are SeqBox and EC-SeqBox?

SeqBox is a single-file archive format designed by [Marco Pontello](https://github.com/MarcoPon) that facilitates sector level data recovery for when file system metadata is corrupted/missing, while the archive itself still exists as a normal file on file system. Please visit the official [SeqBox](https://github.com/MarcoPon/SeqBox) repo for the original implementation and technical details on this.

Error-correcting SeqBox (or EC-SeqBox for short) is an extended version of SeqBox developed for this project, introducing forward error correction via Reed-Solomon erasure code.

Blockyarchive/blkar was formerly known as rust-SeqBox/rsbx prior to renaming.

#### Features overall

- Data recovery that does not depend on file system metadata (sector level recovery)
    - This allows data recovery even when data is fragmented and out of order
- Supports error correction (via Reed-Solomon erasure code) for EC-SeqBox
- Supports burst (sector) error resistance for EC-SeqBox
    - This is done via an interleaving block arrangement scheme. It is mainly to address the data repair limitation of the simple archive design
    - More complex archive designs such as PAR2 can repair burst errors without any extra arrangement scheme, but they are also vastly more complex than EC-SeqBox
- JSON mode
    - Outputs information in JSON format instead of human readable text, allowing easy integration with scripts

#### Limitations

- Only a single file is supported for encoding as SeqBox and EC-SeqBox are both single-file archive formats
    - However, blkar may still be usable when you have multiple files, as blkar supports taking input from stdin during encoding, and also supports outputting to stdout during decoding
    - This means if you have an archiver that supports bundling and unbundling on the fly with pipes, like tar, you can combine the use of the archiver and blkar into one encoding and decoding step

#### Getting started

**Installation**

`blkar` is available via [AUR](https://aur.archlinux.org/packages/blkar), [GitHub releases](https://github.com/darrenldl/blockyarchive/releases) or `cargo`

```
cargo install blkar
```

**Usage guides & screencasts & other resources**

The [wiki](https://github.com/darrenldl/blockyarchive/wiki) contains comprehensive guides and resources.

#### Comparison to the original SeqBox implementation/design

See [comparison](COMPARISON.md).

#### Goals and status

As blkar is to be used largely as a backup utility, security/robustness of the code will be prioritised over apparent performance.

This project has reached its intended feature completeness, so no active development for new features will occur. However, this project is still actively looked after, i.e. I will respond to PRs, issues, and emails, will consider feature requests, respond to bug reports quickly, and so on.

In other words, this is a completed project with respect to its original scope, but it is not abandoned.

#### Note on Rust to Bash ratio

Just to avoid confusion, blkar is written purely in Rust, Bash is only used to write tests.

#### Got a question?

Feel free to join the [Gitter chat](https://gitter.im/blockyarchive/community) if you've got a question. You can email me directly as well.

#### Changelog and specifications

[Changelog](CHANGELOG.md)

[SBX format](SBX_FORMAT.md) (EC-SeqBox is also specified in this document)

[blkar specs](BLKAR_SPECS.md)

## Contributions

Contributions are welcome. Note that by submitting contributions, you agree to license your work under the same license used by this project as stated in the LICENSE file.

## Acknowledgement

I would like to thank [Marco](https://github.com/MarcoPon) (the official SeqBox author) for discussing and clarifying aspects of his project, and also providing of test data during development of osbx. I would also like to thank him for his feedback on the numbering of the error correction enabled ECSBX versions (versions 17, 18, 19).

I would like to thank [Ming](https://github.com/mdchia/) for his feedback on the documentation, UX design, and several other general aspects of the osbx project, of which most of the designs are carried over to blkar, and also his further feedback on this project as well.

The design of the readable rate in progress report text is copied from [Arch Linux pacman](https://wiki.archlinux.org/index.php/Pacman)'s progress bar design.

The design of block set interleaving arrangement in RS enabled versions is heavily inspired by [Thanassis Tsiodras's design of RockFAT](https://www.thanassis.space/RockFAT.html). The interleaving provides resistance against burst sector errors.

## Donation

**Note** : Donation will **NOT** fuel development of new features. As mentioned above, this project is meant to be stable, well tested and well maintained, but normally I am not actively adding new features to it.

If blockyarchive has been useful to you, and you would like to donate to me for the development effort, you can donate through [here](http://ko-fi.com/darrenldl).

## License

#### Libcrc code

The crcccitt code is translated from the C implementation in [libcrc](https://github.com/lammertb/libcrc) and are under the same MIT License as used by libcrc and as stated in libcrc source code, the license text of the crcccitt.c is copied over to `crc-ccitt/build.rs`, `crc-ccitt/src/lib.rs`, `build.rs` and `src/crc_ccitt.rs` as well.

#### Official SeqBox code

The following files in tests folder copied from official SeqBox are under its license, which is MIT as of time of writing

- tests/SeqBox/*

All remaining files are distributed under the MIT license as stated in the LICENSE file.
