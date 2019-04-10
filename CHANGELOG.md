# Changelog

## 5.0.0

- Error-correcting versions of SeqBox are now called Error-correcting SeqBox or EC-SeqBox for short, and use the file extension `.ecsbx`
  - This is done for easier differentiation between the extended versions and the original versions
  - Fundamentally this does not change how blkar functions, as blkar does not take file extensions into account for all modes interacting with SBX containers

## 4.0.0

- Changed "Uid" to "UID" in encode help messages for consistency
- Changed default archiving options
  - Changed from using SBX version 1 to using SBX version 17 with data parity ratio of 10:2 and burst error resistance level of 10 by default
- Slight change in wording in calc mode error correction parameters interpretation
  - Replaced the term "any" with "each" when referring to block set or super block set
- Updated sort mode to ignore failure to sort completely blank blocks by default
  - Added `--report-blank` flag to toggle this behaviour

## 3.0.0

- Changed decode mode to use only file portion of stored file name in SBX container

  - In previous versions, if the stored file name contains a path, then the entire path would be used, which can lead to unexpected output locations

- Added `--multi-pass` and `--multi-pass-no-skip` flag to the following modes

  - decode
  - sort mode
  - This disables truncation, and allows updating the file gradually across multiple runs

- Upgraded stats tracking in sort mode

  - Now it also tracks number of blocks in same order and in different order

- Added `--dry-run` flag to sort mode

  - Combined with the improved stats tracking allows checking if the container is sorted or not

- Repalced `--to` with `--to-inc` and `--to-exc`

  - This affects `rescue` and `show` mode

- Added `--from`, `--to-inc` and `--to-exc` to the following modes

  - check
  - decode
  - encode
  - sort

- Added `--ref-from`, `--ref-to-inc` and `--ref-to-exc` to the following modes

  - check
  - decode
  - sort

- Added `--force-misalign` to the following modes

  - check
  - decode
  - sort

- Updated reference block scanning code to respect `--force-misalign`

- Updated burst error resistance level guessing to respect `--force-misalign`

- Updated decode mode stats output

  - This results in potentially incompatible JSON output

- Fixed `misc_utils::calc_required_len_and_seek_to_from_byte_range`

  - Sometimes off by one error occured

- Ran `rustfmt` for code formatting

- Ran `cargo fix --edition` to update to Rust 2018 edition

- Fixed potential integer overflow issues in decode and repair mode

  - Previously, when output is stdout, block index and seq num counter in decode mode may be incremented even if already at max
  - Previously, seq num counter in repair mode may be incremented even if already at max

- Updated burst error resistance level guessing code to respect `--from` and `--force-misalign` options

- Fixed potential incorrect behaviour when processed block is incomplete

  - Rectified by adding `#[must_use]` to `Reader::ReadResult`, forcing all code paths to check read result instead of possibly ignoring it

## 2.2.0

- Added `--only-pick-uid` option to show mode

- Changed "uid" to "UID" in output text for consistency

## 2.1.0

- Added `--burst` option to decode mode, used when output is stdout and container version is RS enabled

- Updated help messages in decode, encode mode to note that `./-` can be used when the file of interest is named `-`

## 2.0.1

- First release under name `blkar`

- Warning message fix for stdout output in decode mode

## 2.0.0

- Dependencies upate

  - Updated `rand` from `0.4` to `0.5.4`

- Switched from `ring` to `sha-1` and `sha2` crates

  - See issue [#86](https://github.com/darrenldl/rust-SeqBox/issues/86)

- Doc fix

  - Added space before parantheses in code comments, documentation and help messages

- Added stdin input option for encode mode

- Added stdout output option for decode mode

- Fixed data padding bytes calculation in encode mode

- Renamed project from `rust-SeqBox/rsbx` to `blockyarchive/blkar`

## 1.1.2

- Dependencies update
  - Updated `reed-solomon-erasure` from `^3.0` to `^3.1`

## 1.1.1

- Added fuzzing suite
  - No code changes from this as no bugs were found
- Dependency update
  - Updated `nom` from `^3.2` to `^4.0`
- Fixed incorrect use of nom combinators
  - Previously was using `alt!` while `alt_complete!` should have been used
  - This affects the following parsers
    - `multihash` (multihash parser for metadata of hash)
    - `sbx_block::metadata` (metadata parser for metadata blocks)
    - `sbx_block::header::ver_p` (version parser for version byte in header)

## 1.1.0

- Added --json flag to all modes
  - If specified, all information will be outputted in JSON format instead of human readable text
  - This includes progress report text, which is outputted to stderr
- Progress report text changed to use stderr

## 1.0.8

- Updated file size retrieval procedure to handle block devices properly
  - Previously modes would not interact with block devices correctly since metadata of block devices gives file size of 0
  - Currently modes retrieve file size via seeking to the end of file, this gives the block device size correctly

## 1.0.7

- Polished repair stats text

## 1.0.6

- Improved calc mode dialog about interpretation of error correction parameters

## 1.0.5

- No code changes
- Added binary releases via GitHub releases

## 1.0.4

- Help messages polish
- Added text in help messages about rsbx's limitations on burst error resistance level
- Massively improved code coverage
  - Added a lot of internal tests
- Bug fixes in following internal functions
  - Note that the main binary may restrict parameters provided to these internal functions, so not all bugs are visible or reproducible from user perspective
  - Fixed `calc_required_len_and_seek_to_from_byte_range_inc`, issue #56
    - rescue core and show core uses this function to calculate seek to positions and number of bytes to read
  - Fixed `make_path`, issue #57
    - All modes that outputs files use this function to calculate final output path
  - Fixed `rs_coder::encoder` incorrect index counting logic
    - This is used by encode mode, thought to be fixed in `1.0.0`
    - Does **NOT** actually lead to incorrect SBX container generation
    - This means containers generated by rsbx version `>= 1.0.0` are still correct
- Fixed meta blocks written stats reporting in encode mode, issue #59

## 1.0.3

- General output text polishing
- Fixed container size calculation for when --no-meta flag is supplied
- Fixed encode mode for when --no-meta flag is supplied
  - Previously rsbx would leave a blank spot at where the metadata block would otherwise sit instead of skipping the metadata block properly
  - SBX containers created with --no-meta flag enabled prior to this fix are still valid and can be decoded by rsbx successfully
- Fixed reference block retrieval procedure related code
  - Previously for decode, sort, and check mode, rsbx would interpret --no-meta flag incorrecty for reference block preference, namely any block type is allowed when the flag is absent, and metadata block is preferred when the flag is present, while it should be the other way around
- Fixed a crash that occurs when sort mode is used with a SBX container of RS enabled version, and using data block as reference block

## 1.0.2

- Fixed wording of error correction parameters interpretation strings in calc mode
- Fixed container size calculation for when file size is 0
  - Previously for RS enabled SBX versions, rsbx would fail to take burst gaps between metadata blocks into account

## 1.0.1

- Added displaying of metadata block repairs in repair mode when verbose flag is supplied

## 1.0.0

- Added fields to stats display in encode mode
  - uid
  - file size
  - container size
- Added fields to stats display in decode mode
  - uid
  - file size
  - container size
- General output text polishing
- Fixed repair mode code to handle block sets with blocks missing due to truncation properly
- Fixed encode mode code to avoid writing extraneous RS block set
  - Previously if data read finishes right at the end of a block set, the RS codec would write out an extra RS block set with data blocks being just padding
- Added --dry-run flag to repair mode
- Added displaying of position in file of blocks requiring repair in repair mode

## 0.9.3

- Various UI/UX improvements in subcommands
  - Added --info-only flag to encode mode to show info about encoding
  - Added file and container sizes to encode mode stats
- Added calc mode to show detailed info about encoding configuration

## 0.9.2 (forgot to publish, whoops)

- Made decode mode output file path determination more robust
  - Only the file part of the SNM field is used rather than the entire path when computing the final output path
- Added `--info-only` flag to encode mode
  - Using the flag shows various calculation results and statistical information

## 0.9.1

- Fixed encode mode output file determination logic
  - Prior to this version, encode mode would append the entire input path to the output path if output path is a directory, instead of just appending only the file name part

## 0.9.0

- Base version
