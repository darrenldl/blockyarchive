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

## Encode workflow
1. If metadata is enabled, the following file metadata are gathered from file or retrieved from user input : file name, SBX file name, file size, file last modification time, encoding start time
2. If metadata is enabled, then a partial metadata block is written into the output file as filler
  - The written metadata block is valid, but does not contain the actual file hash, a filler pattern of 0x00 is used in place of the hash part of the multihash(the header and length indicator of multihash are still valid)
3. Load version specific data sized chunk one at a time from input file to encode and output(and if metadata is enabled, Multihash hash state/ctx is updated as well(the actual hash state/ctx used depends on hash type, defaults to SHA256)
  - data size = block size - header size (e.g. version 1 has data size of 512 - 16 = 496)
4. If metadata is enabled, the encoder seeks back to starting position of output file and overwrites the metadata block with one that contains the actual hash

## Decode workflow
Metadata block is valid if and only if
- Header can be parsed
- All metadata fields(duplicate or not) can be parsed successfully
  - Duplicate refers to metadata fields with the same ID
- All remaining space is filled with 0x1A pattern
- Version(specifically alignment/block size) matches reference block(see below)
- CRC-CCITT is correct

Data block is valid if and only if
- Header can be parsed
- Version and uid matches reference block(see below)
- CRC-CCITT is correct

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
  - if a block is invalid, nothing is done
  - if a block is valid, and is a metadata block, nothing is done
  - if a block is valid, and is a metadata parity block, nothing is done
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

#### Handling of duplicate metadata in metadata block given the block is valid
- For a given ID, only the first occurance of the metadata will be used
  e.g. if there are two FNM metadata fields in the metadata block, only the first (in terms of byte order) will be used

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
- each block is appended to OUTDIR/uid, where :
  - OUTDIR = output directory specified
  - uid    = uid of the block in hex
- the original bytes in the file is used, that is, the output block bytes are not generated from scratch by oSBX
2. User is expected to attempt to decode the rescued data in OUTDIR using the oSBX decode command

## Show workflow
1. Scan for metadata blocks from start of provided file using 128 bytes alignment
  - if block scanned has sequence number 0, then
    - if the block is a valid metadatablock, it will be collected
    - up to some specified maximum number of blocks are collected(defaults to 1)
  - else
    - nothing is done
2. Metadata of collected list of metadata blocks are displayed

## Repair workflow
1. Load metadata block and the 3 parity blocks, repair any of the 4 blocks if necessary
2. Load up to M + N blocks sequentially, where M is the number of data shards and N is the number of parity shards
3. Check CRC of all blocks and record invalid blocks
4. Reconstruct the invalid blocks if possible

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
