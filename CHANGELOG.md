# Changelog

## 7.2.7

- Dependencies update

    - Updated `reed-solomon-erasure` from `~3.1.1` to `~4.0.1`

    - Updated `num_cpus` from `~1.10.1` to `~1.11.1`

    - Updated `rayon` from `~1.1.0` to `~1.2.0`

    - Updated `smallvec` from `~0.6.9` to `~1.0.0`

## 7.2.6

- Updated progress report text

    - Previously, progress report text while running and when finished look as follows
        - ```
          [#############------------]   54%  cur :  56.18M bytes/s  used : 00:00:00  left : 00:00:00
          ```
        - ```
          Time elapsed : 00:00:01  Average rate :  59.69M bytes/s
          ```
    - After the update, the units processed is present in the text as well
        - ```
          [#############------------]   54%   57.54M bytes  cur :  56.18M bytes/s  used : 00:00:00  left : 00:00:00
          ```
        - ```
          Processed : 104.86M bytes  Time elapsed : 00:00:01  Average rate :  59.69M bytes/s
          ```

- Fixed `avgPerSec` in JSON progress report text

- Optimised progress text output

    - Previously, at the start, time left field in text carries a lot of digits as not enough stats is available and so a dummy small value is used for current rate, which causes time left to be a very large number initially.
    
        - This stretches the space used by the text needlessly
        
    - Now current rate and time left are displayed as "N/A" instead of using dummy values

- Updated `quickcheck` from `~0.8.2` to `~0.9.0`

## 7.2.5

- Fixed encode core buffer handling

    - Added missing `buffer.cancel_slot()` for case where seq num overflows

    - Previously, encode mode panics if seq num overflows due to file size change during encoding

## 7.2.4

- Fixed input file size validation, encode mode seq num tracking and checking
  
    - Bug does not impact archive integrity
      
        - Blkar has another lower layer of seq num overflow check for cases where input file size changes when running, and would previously cause blkar to panic. This fix makes it terminate gracefully by fixing the upper layer to catch the condition properly.
      
        - In other words, if blkar did not crash on you when creating the archive, then the archive was still valid
  
    - See [PR #256](https://github.com/darrenldl/blockyarchive/pull/256) for details

## 7.2.3

- Updated encode mode input file size calculation to use the data bytes encoded directly instead of file calculation
  
    - This mostly affects encoding and decoding of files 0 in length
  
    - This does, however, also make the recording of FSZ field more accurate in the event that the input file size changes during the encoding process, in which case the FSZ field will not match the actual number of bytes encoded

- Added multithreading and operation pipelining to rescue and sort core
  
    - Rescue mode is 4x the performance (jumping from ~50MB/s to ~200MB/s) and sort mode is 2.75x the performance (jumping from ~80MB/s to ~220MB/s) on my laptop
  
    - See [issue #243](https://github.com/darrenldl/blockyarchive/issues/243) for details

- Fixed input file name handling and recorded file name handling
  
    - Bug does not impact archive integrity
  
    - See issues [#249](https://github.com/darrenldl/blockyarchive/issues/249) and [#250](https://github.com/darrenldl/blockyarchive/issues/250) for details

## 7.2.2

- Updated encode core to terminate if an incomplete data chunk is read as well
    - Previously encode core only terminates when required number of byte count is reached or when reader returns 0 bytes
  
    - In practice, this should not cause any difference especially for file reader, and mainly serves as a better enforcement of termination determination logic by making less assumption about the input reader
  
    - For readers with jitters, however, the previous behaviour may have caused some inconsistencies. There was a single test case where encode mode with stdin input was used, blkar recorded more data than the original file has. I could not pinpoint the error in the code base after a lot of code review, so it seems to be an issue with the pipe rather than blkar itself. Following is what I suspect to have happened in that particular test case.
      
        - The decoded file contains more data than the original file, but there was no mismatch in the data up to the length of the original file. So data was recorded correctly, but with extra bytes at the end, indicating incorrect input data byte count.
      
        - If stdin does have very rare occasional jitter, then the encountered failure makes sense. It could be the case that the last chunk of data is split across, say, two read results due to the jitter, leading to two data chunks both smaller than 496 bytes (or data size of another SBX version) in length. And since in the previous logic, reading fewer than 496 bytes does not cause termination, this leads to two data blocks both having padding, while the SBX format design only assumes the last data block to have padding, if any at all. And if the padding pattern (0x1A) matches the last few bytes of the original file, then there would not be any mismatch up to the length of the original file.
      
        - Since encode core hashing code takes length of each individual reads into account, the hash result displayed during encoding would still be still correct, even though the decoded file would contain a mismatch and consequently mismatching hash.
  
    - Overall this should be a very rare occurance, as the reader code in blkar already has a retry 5 times logic for reading from file or stdin, so these types of jitters should not be visible. But the code logic is patched to reduce assumptions made about the reader anyway.

## 7.2.1

- Fixed `sbx_container_content::hash`
  
    - Previously, it may produce incorrect hash due to incorrect termination determination logic of core loop
      
        - Specifically the core loop may end while reading a parity block, causing the final block set to be missed out entirely for hashing
        - A simple way to reproduce the issue is as follows
        - ```
          $ touch dummy
          $ truncate -s 4961
          $ blkar encode dummy
          $ blkar check --hash dummy.ecsbx
          ```
        - The hash check will fail due to the computed hash being different to the recorded hash
  
    - This only affects the following modes and usage
      
        - check mode `--hash`, `--hash-only`
      
        - update mode `--hash`
      
        - All other modes are not affected by this
  
    - 7.2.0 is yanked nonetheless to avoid widespread use of update mode `--hash` option which may result in incorrect hash being stored

- Fixed Reed-Solomon erasure encoding issue
  
    - Previously for 7.1.1 and 7.2.0, the encoding is done over the entire SBX block rather than just the data section of the SBX block
  
    - Due to how Reed-Solomon erasure coding works, the final result is still the same, and archives generated by blkar 7.1.1 and 7.2.0 should still be valid
      
        - The tests suite confirms this as well
  
    - 7.1.1 is yanked nonetheless to avoid other potential issues

- Added multithreading and operation pipelining to `sbx_container_content::hash`
  
    - This speeds up the hashing step of the following modes and usage
        - check mode `--hash`, `--hash-only`
      
        - update mode `--hash`
      
        - See [issue #222](https://github.com/darrenldl/blockyarchive/issues/222) for details

- Added multithreading and operation pipelining to decode core
  
    - This speeds up decoding in all scenarios
    - See [issue #222](https://github.com/darrenldl/blockyarchive/issues/222) for details

- Dependencies update
  
    - Updated `rand` from `~0.6.1` to `~0.7.0`
    - Updated `nom` from `~4.2.3` to `~5.0.0`

## 7.2.0 (yanked)

- Fixed hash type validation in commandline arguments processing
  
    - Some unsupported hash functions are accepted but not actually usable in core code, and cause crash when hash context is being created
  
    - These unsupported hash functions are not listed in help messages, so normally not triggered

- Added --hash to update mode
  
    - This options allows rehashing stored data in the SBX container with a possibly different hash function

- Check mode UX improvement
  
    - Previously if hashing fails, then blkar errors out without showing the block check stats. This wastes a lot of time if the container is large, as the block check stats could be useful in diagnosis.
    - Now blkar displays the error during stats reporting instead of erroring out and exiting immediately. This means block check stats are visible even when hashing error occurs.

## 7.1.1 (yanked)

- Updated file error messages casing

- Added multithreading and operation pipelining to encode core
  
    - For SBX encoding
      
        - Performance of encode mode is roughly 75% faster
  
    - For ECSBX encoding
      
        - Performance of encode mode now scales roughly linearly to number of CPU cores for version 17, 18

- Fixed progress reporting code synchronisation issue
  
    - On some occasions, the summary of progress, specifically the time elapsed and average rate, may not be correctly calculated

- Fixed check and decode mode reference block checking
  
    - Previously it may accept a reference block which does not contain `RSD` or `RSP` field even though it is required for version 17, 18, 19

- Fixed help message on behaviour of --guess-burst-from option
  
    - It was stated it defaults to start of file, but it should state it defaults to --from option value

- Fixed help message on behaviour of --ref-from option
  
    - It was stated it defaults to start of file, but it should state it defaults to --from option value mod SBX scan block size (128)

- Added missing --burst option to check mode

- Fixed input file size checking for encode mode
  
    - Previously for SBX version 17, 18, 19, blkar fails to take data and parity shard ratio into account

## 7.1.0

- Dependencies update
    - Updated use of `blake2_c` to `blake2`
- Updated encode help message to mention BLAKE2b-256 as a supported hash function explicitly
- Added support for following hash functions
    - BLAKE2s-128
    - BLAKE2s-256

## 7.0.0

- Updated `--pv` help message to state the default in JSON mode

- Added `update` mode for updating metadata

- Updated output text of following modes to use null instead of "N/A" for missing fields in JSON mode for consistency
  
    - Encode and show mode
  
    - Bumped major version as this may break backward compatibility

- Updated JSON code to output all numbers without quotes for consistency
  
    - Previously, version numbers were quoted
  
    - Previously, there may have been inconsistencies in general as well
  
    - Bumped major version as this may break backward compatibility

- Switched to using tilde requirements for dependencies
  
    - This is to ensure build stability for users who install blkar via crates.io, as `Cargo.lock` is not currently published along with the package on crates.io

- Updated progress reporting code
  
    - Encode stdin mode now reports current rate and time used during encoding, and shows normal progress stats at the end
    - The following modes now use "bytes" as units for progress reporting instead of "chunks" or "blocks"
        - Check
        - Decode
        - Encode
        - Repair
        - Sort

- Fixed crashing bug in repair mode (issue [#191](https://github.com/darrenldl/blockyarchive/issues/191)) in PR [#192](https://github.com/darrenldl/blockyarchive/pull/192)
  
    - In repair mode, if the version number of any data block is using a version number which size exceeds the one associated with version number of reference block, then blkar panics and crashes
        - For example, for a EC-SeqBox archive of version 17, if any of the data block is changed to version 19 in its header, then blkar will crash, as version 17 is of block size 512 bytes, while version 19 is of block size 4096 bytes
    - The reason is that the buffer size in the RS codec is fixed at the beginning based on the version number of reference block (the buffer is used for all later decoding of blocks), and even though a predicate was already provided in repair mode for tackling this, predicate checking in `sbx_block::sync_from_buffer` at the time only occurs after header parsing and reading from buffer as the predicate is a block predicate
        - To be more specific, the predicate used in repair mode ensures the block to share same version and UID with referece block, but the checking occurs after reading from the buffer with length based on block size of version number. The reading from buffer triggers a panic if buffer is not of sufficient length.
    - This is fixed by adding a `header_pred` parameter for `sbx_block::sync_from_buffer`, and changing all predicates supplied to `sbx_block::sync_from_buffer` from block predicates to header predicates when possible
        - In repair mode case, this means now the block is filtered before reading from buffer takes place, if the block is not of the correct version, thus mitigating the issue

- Changed encoding defaults
  
    - From `sbx-version=17, rs-data=10, rs-parity=2, burst=10`
  
    - To `sbx-version=17, rs-data=10, rs-parity=2, burst=12`
  
    - This means now by default the archive can survive burst sector error of size 3 on modern disks where the sector size is 4096 bytes
  
    - Bumped major version as this may break backward compatibility

- Added options for hashing stored data in check mode
  
    - This can be triggered via `--hash` or `--hash-only`
  
    - Both are incompatible with range options, as opposed to decode mode where hashing is still done with range options
      
        - This is to reduce complexity, especially since ranged hashing isn't very useful in general

- Time elapsed fields display update for decode mode
  
    - Now decoding time and hashing time are displayed separately

## 6.0.1

- Minor fixes for rescue and decode mode help messages
- Minor fix for calc mode output text

## 6.0.0

- Updated `calc` mode to use the same defaults as `encode` mode
  
    - Bumped major version as this may break backward compatibility

- Fixed `check` and `sort` mode progress tracking when dealing with blank blocks
  
    - Previously, blank blocks do not count toward progress made unless `--report-blank` is supplied

## 5.0.0

- Error-correcting versions of SeqBox are now called Error-correcting SeqBox or EC-SeqBox for short, and use the file extension `.ecsbx`
    - This is done for easier differentiation between the extended versions and the original versions
    - Fundamentally this does not change how blkar functions, as blkar does not take file extensions into account for all modes interacting with SBX containers
    - Bumped major version as this may break backward compatibility
- `Cargo.lock` update via `cargo update`

## 4.0.0

- Changed "Uid" to "UID" in encode help messages for consistency
- Changed default archiving options
    - Changed from using SBX version 1 to using SBX version 17 with data parity ratio of 10:2 and burst error resistance level of 10 by default
    - Bumped major version as this may break backward compatibility
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
