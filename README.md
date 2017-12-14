# rust-SeqBox
Enhanced implementation of SeqBox in Rust

SeqBox is a single-file archive format designed by [Marco Pontello](https://github.com/MarcoPon) that facilitates sector level data recovery when file system metadata is corrupted/missing, and data is fragmented and not in order, while the archive itself still exists as a normal file on file system.

Please visit the official [SeqBox](https://github.com/MarcoPon/SeqBox) repo for technical details on this.

## Enhancements
This implementation adds forward error correction on top of the SeqBox format by adding support for Reed-Solomon erasure code, and also aims for higher performance(via more optimized code and utilising concurrency capability of Rust), but otherwise is based around [osbx](https://github.com/darrenldl/ocaml-SeqBox).

## Goals
As rsbx is to be used largely as a backup utility, security/robustness of the code will be prioritised over apparent performance.

And also since I don't have a lot of time outside of holidays, modularity and ease of maintenance will be of high priority during development as well so I can keep maintaining rsbx in future.

## Compared to osbx
rsbx will be using the same test suite for the core functionalities, and should share largely the same command line interface, with differences due to different designs of the libraries.

rsbx is expected to have higher performance than osbx.

rsbx has forward error correction, osbx does not.

## TODO
- Things to port from osbx
  - ```sbx_block.ml```, using [nom](https://github.com/Geal/nom) parser combinator library
  - Command line interface code, using [clap](https://github.com/kbknapp/clap-rs)
  - ```crcccitt.ml```
  - ...
- Diagrams for the actor model used

## Done
- ```crcccitt``` crate
- ```mpmc_rb``` crate (multi-producer multi-consumer ring buffer)

## Acknowledgement
I would like to thank [Marco](https://github.com/MarcoPon) (official SeqBox author) for discussing and clarifying aspects of his project, and also providing of test data during development of osbx.

I would like to thank [Ming](https://github.com/mdchia/) for his feedback on the documentation, UX design, and several other general aspects of the osbx project, of which most of the designs are carried over to rsbx.

## License

#### Libcrc code
The crcccitt code is translated from the C implementation in [libcrc](https://github.com/lammertb/libcrc) and are under the same MIT License as used by libcrc and as stated in libcrc source code, the license text of the crcccitt.c is copied over to ```crcccitt/build.rs``` and ```crcccitt/src/lib.rs``` as well

The C source code of crcccitt copied directly from libcrc are under the MIT License as used by libcrc, the files are in ```libcrc_crcccitt```

#### BackBlaze Java Reed-Solomon implementation
The tables and main functions of ```reed_solomon/build.rs``` and ```reed_solomon/src/*``` are translated from (BackBlaze Java Implementation)[https://github.com/Backblaze/JavaReedSolomon], and are under the same MIT License as used by the BackBlaze project

The source code copied directly from BackBlaze's project repo are under the MIT License as used by the project, the files are in ```BackBlaze_JavaReedSolomon```

#### Klaus Post Go Reed-Solomon implementation
The tables and test functions of ```reed_solomon/src/*``` are translated from (Klaus Post's Go Implementation)[https://github.com/klauspost/reedsolomon], and are under the same MIT License as used by Klaus Post's project

The source code copied directly from Klaus Post's project repo are under the MIT License as used by the project, the files are in ```KlausPost_reedsolomon```

#### Official SeqBox code
The files in tests folder copied from official SeqBox are under its license, which is MIT as of time of writing
  - tests/SeqBox/*

All remaining files are distributed under the 3-Clause BSD license as stated in the LICENSE file
