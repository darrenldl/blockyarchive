# Specification of rust-SeqBox

## Exit code
rsbx returns
- 0 if no errors occured
- 1 if error is detected in user input(i.e. parameters provided)
- 2 if error is detected during operation

## Error handling behaviour in general
- rsbx does **not** remove the generated file(s) even in case of failure
  - This applies to encoding, decoding, rescuing (showing does not generate any files)
  - This is mainly for in case the partial data is useful to the user

## Block handling in general
#### Block validity
Block is valid if
- Header can be parsed
- CRC-CCITT is correct

#### Handling of duplicate metadata in metadata block given the block is valid
- For a given ID, only the first occurance of the metadata will be used
  - e.g. if there are two FNM metadata fields in the metadata block, only the first (in terms of byte order) will be used
- This applies to everywhere where metadata fields need to be accessed

#### Handling of incorrect metadata fields in metadata block given the block is valid
- To avoid propogation of error into core logic, incorrect fields either fail the parsing stage, or are filtered out immediately after the parsing stage. That is, invalid metadata fields are never accessible by other modules.
- This tradeoff means rsbx's error messages regarding metadata fields will be very coarse. For example, if the recorded file name is not a valid UTF-8 string, the core logic code will only see the field as missing, as it is dropped by the `sbx_block` module during parsing, and would not be able to tell whether the field is missing or incorrect, and would not be able to tell the user why the field is incorrect, etc.
- This overall means trading flexibility for security.

## Finding reference block
1. The entire SBX container is scanned using alignment of 128 bytes, 128 is used as it is the largest common divisor of 512(block size for version 1), 128(block size for verion 2), and 4096(block size for version 3)
  - if any block type is allowed
    - the first whatever valid block(i.e. valid metadata or data block) will be used as reference block
  - else
    - if there is any valid metadata block in SBX container, then the first one will be used as reference block
    - else the first valid data block will be used as reference block

## Guessing burst error resistance level
1. Read sequence numbers of first up to **1 + parity shard count + 1000** blocks
- if block is valid, record the sequence number
- else mark the sequence number as missing
- a ref block is required to provide guidance on version and uid accepted
2. Go through level 0 to 1000(inclusive), calculate supposed sequence number at each block position, record number of mismatches for each level
- if sequence number was marked missing, then it is ignored and checked for mismatch
3. return the level with least amount of mismatches

## Encode workflow
1. If metadata is enabled, the following file metadata are gathered from file or retrieved from user input
- file name
- SBX file name
- file size
- file last modification time
- encoding start time
2. If metadata is enabled, then a partial metadata block is written into the output file as filler
  - The written metadata block is valid, but does not contain the actual file hash, a filler pattern of 0x00 is used in place of the hash part of the multihash(the header and length indicator of multihash are still valid)
3. Load version specific data sized chunk one at a time from input file to encode and output(and if metadata is enabled, Multihash hash state/ctx is updated as well(the actual hash state/ctx used depends on hash type, defaults to SHA256)
  - data size = block size - header size (e.g. version 1 has data size of 512 - 16 = 496)
4. If metadata is enabled, the encoder seeks back to starting position of output file and overwrites the metadata block with one that contains the actual hash

## Decode workflow
Metadata block is valid if
- Basic block validity criteria are satisfied(see **Block handling in general above**)
- Version and uid matches reference block(see below)
- Several aspects are relaxed and allowed to not conform to `SBX_FORMAT`
  - Metadata fields are optional, i.e. do not have to be parsable
  - Padding of 0x1A is not mandatory

Data block is valid if and only if
- Basic block validity criteria are satisfied(see **Block handling in general above**)
- Version and uid matches reference block(see below)

1. A reference block is retrieved first and is used for guidance on alignment, version, and uid(see **Finding reference block** procedure specified above)
2. Scan for valid blocks from start of SBX container to decode and output using reference block's block size as alignment
  - if a block is invalid, nothing is done
  - if a block is valid, and is a metadata block, nothing is done
  - if a block is valid, and is a data parity block, nothing is done
  - if a block is valid, and is a data block, then it will be written to the writepos at output file, where writepos = (sequence number - 1) * block size of reference block in bytes
3. If possible, truncate output file to remove data padding done for the last block during encoding
  - if reference block is a metadata block, and contains file size field, then the output file will be truncated to that file size
  - otherwise nothing is done
4. If possible, report/record if the hash of decoded file matches the recorded hash during encoding
  - if reference block is a metadata block, and contains the hash field, then the output file will be hashed to check against the recorded hash
    - output file will not be deleted even if hash does not match
  - otherwise nothing is done

#### Handling of duplicate metadata/data blocks
- First valid metadata block will be used(if exists)
- For all other data blocks, the last seen valid data block will be used for a given sequence number

#### Handling of corrupted/missing blocks
- Corrupted blocks or missing blocks are not repaired in this mode
- User needs to invoke repair mode to repair the archive

## Rescue workflow
1. Scan for valid blocks from start of the provided file using 128 bytes alignment
- rescue mode rescues all 3 versions of SBX blocks
- if log file is specified, then
  - if the log file exists, then it will be used to initialize the scan's starting position
    - bytes_processed field will be rounded down to closest multiple of 128 automatically
  - the log file will be updated on every ~1.0 second
- each block is appended to OUTDIR/UID, where :
  - OUTDIR = output directory specified
  - UID    = uid of the block in hex(uppercase)
- the original bytes in the file is used, that is, the output block bytes are not generated from scratch by rsbx
2. User is expected to attempt to decode the rescued data in OUTDIR using the rsbx decode command

## Show workflow
1. Scan for metadata blocks from start of provided file using 128 bytes alignment
- if show all flag is supplied, all valid metadata blocks are displayed
- else only the first valid metadata block are displayed
- all displaying of blocks are immediate(no buffering of blocks)

## Repair workflow
1. A reference block is retrieved first and is used for guidance on alignment, version, and uid(see **Finding reference block** procedure specified above)
- a metadata block must be used as reference block in this mode
2. If the version of ref block does not use RS, then exit
3. If `RSD` and `RSP` fields are not found in the ref block, then exit
4. Total block count is then calculated from
- `FSZ` field in ref block if present
- otherwise is estimated the container size
5. Go through all positions where metadata blocks are stored in container
- if the metadata block is valid, nothing is done
- else the metadata block is overwritten by the reference block
6. Go through sequence numbers sequentially until the block count reaches calculated total block count
- For each sequence number, calculate the block position and try to parse
- Each valid block is loaded into the RS codec, and repair process starts for the current block set when the current block set is filled
7. If current blockset contains enough blocks for repair, but repair process failed to start due to the block count reaching the calculated total block count
- This indicates blocks are missing due to truncation
- The the RS codec is invoked once to attempt repair, and write out remaining blocks if repair is successful

## Check workflow
1. A reference block is retrieved first(which is used for guidance on alignment, version, and uid)
  - the entire SBX container is scanned using alignment of 128 bytes, 128 is used as it is the largest common divisor of 512(block size for version 1), 128(block size for verion 2), and 4096(block size for version 3)
  - if no-meta flag is specified
    - the first whatever valid block(i.e. valid metadata or data block) will be used as reference block
  - else
    - if there is any valid metadata block in SBX container, then the first one will be used as reference block
    - else the first valid data block will be used as reference block
  - if the version of reference block is 1, 2, or 3
    - the block can be either `Data` or `Meta`, and all metadata fields are optional
  - else if the version of reference block is 17, 18, or 19
    - the block must be `Meta`, and metadata fields `RSD`, `RSP` must be present
2. Scan for valid blocks from start of SBX container to decode and output using reference block's block size as alignment
  - if a block is invalid, and error message is shown
  - if a block is valid, nothing is done

#### Handling of irreparable blocks
- Output sequence number of the blocks to log

#### Handling of duplicate, out of order blocks, or block sequence number jumps
- Halt repair process

## Sort workflow
1. Check if destination has sufficient space for a complete replica of the current file(may not be sufficient estimate)
2. Read block from input file sequentailly, write to position calculated from sequence number and block size to output file

#### Handling of missing blocks
- Jumps/gaps caused by missing blocks are left to file system to handle(i.e. this may result in sparse file, or file with blank data in the gaps)

#### Handling of corrupted blocks
- Still write to output file

#### Handling of duplicate metadata/data blocks
- Append block to FILENAME.TIME.rSBX.leftover, where FILENAME is the specified archive name(not the name stored in metadata), TIME is string of format "%Y-%M-%D_%h%m" of the start of the sorting process

## To successfully encode a file
- File size must be within threshold
  - For version 1, that means  496 * 2^32 - 1 =  ~1.9375 TiB, where 496 is data size, obtained via 512(block size) - 16(header size)
  - For version 2, that means  112 * 2^32 - 1 =  ~0.4375 TiB, where 112 is data size, obtained via 128(block size) - 16(header size)
  - For version 3, that means 4080 * 2^32 - 1 = ~15.9375 TiB, where 4080 is data size, obtained via 4096(block size) - 16(header size)
- If the file size changes during encoding to beyond the threshold, then the encoding process will be halted

## To successfully decode a SBX container
- At least one valid data block for each position must exist
- If data padding was done for the last block, then at least one valid metadata block must exist and the first block amongst the valid metadata blocks needs to contain a field for the file size in order for truncation of the output file to happen

## To successfully rescue your SBX container
- Get enough valid SBX blocks of your container such that a successful decoding may take place

## To successfully repair your SBX container
- The container has metadata block(or enough metadata parity blocks to reconstruct if corrupted/missing)
- The container blocks are sorted by the sequence number in increasing order
- The container has no duplicate blocks
- The container has enough valid parity blocks to correct all errors

## To successfully sort your SBX container
- There is space to store temporary file of same size at the specified destination
