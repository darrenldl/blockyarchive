# Specification of blockyarchive

The specification is concerned only with actual data operations, UI/UX related matters are ignored.

## Exit code

blkar returns

- 0 if no errors occured
- 1 if error is detected in user input (i.e. parameters provided)
- 2 if error is detected during operation

## Error handling behaviour in general

- blkar does **not** remove the generated file(s) even in case of failure
    - This applies to encoding, decoding, repairing, rescuing, and sorting
        - calculating, checking, showing do not generate any files
    - This is mainly for in case the partial data is useful to the user

## Output

The cli argument parsing library (clap) outputs errors to stderr.

If no errors are discovered by the cli argument parsing library, then

- In non-JSON mode
    - Progress report texts are outputted to stderr
    - All other texts are outputted to stdout, including error messages
- In JSON mode
    - Progress report JSON data is outputted to stderr
        - Each line holds a single JSON object
    - All other JSON data is outputted to stdout
        - The entire output forms a single JSON object

## Block handling in general

#### Basic block validity

Block is valid if

- Header can be parsed
- CRC-CCITT is correct

#### Metadata block validity

Metadata block is valid if

- Basic block validity criteria are satisfied
- Several aspects are relaxed and allowed to not conform to `SBX_FORMAT`
    - Metadata fields are optional
        - They can be missing or unparsable
    - Padding using 0x1A is not mandatory

#### Handling of duplicate metadata in metadata block given the block is valid

- For a given ID, only the first occurance of the metadata will be used
    - e.g. if there are two FNM metadata fields in the metadata block, only the first (in terms of byte order) will be used
- This applies to everywhere where metadata fields need to be accessed

#### Handling of incorrect metadata fields in metadata block given the block is valid

- To avoid propogation of errors into core logic, incorrect fields either fail to be parsed in the parsing stage, or are filtered out immediately after the parsing stage. That is, invalid metadata fields are never accessible by other modules.
- This tradeoff means blkar's error messages regarding metadata fields will be very coarse. For example, if the recorded file name is not a valid UTF-8 string, the core logic code will only see the field as missing, as it is dropped by the `sbx_block` module during parsing, and would not be able to tell whether the field is missing or incorrect, and also would not be able to tell the user why the field was not accessible, etc.
- This overall means trading flexibility and information granularity for better security.

## Finding reference block

1. The entire SBX container is scanned using alignment of 128 bytes, 128 is used as it is the largest common divisor of 512(block size for version 1), 128(block size for verion 2), and 4096(block size for version 3)
     - if any block type is allowed
         - the first whatever valid block (i.e. valid metadata or data block) will be used as reference block
     - else
         - if there is any valid metadata block in SBX container, then the first one will be used as reference block
         - else the first valid data block will be used as reference block

## Guessing burst error resistance level

1. Read sequence numbers of first up to **1 + parity shard count + 1000** blocks
     - if block is valid, record the sequence number
   
     - else mark the sequence number as missing
   
     - a ref block is required to provide guidance on version and uid accepted
2. Go through level 0 to 1000 (inclusive), calculate supposed sequence number at each block position, record number of mismatches for each level
     - if sequence number was marked missing, then it is ignored and checked for mismatch
3. return the level with least amount of mismatches

## Guessing starting block index

1. Read **1024** blocks starting from specified `from` position
   
     - if block is valid && is meta, then mark as missing
   
     - if block is valid && is data, calculate anticipated global block index from the sequence number
       
         - if **global block index < blocks processed**, then mark as missing
       
         - else mark block index as **global block index - blocks processed**
   
     - Overall this collects the starting block indices calculated from the global block indices of sampled blocks

2. Go through collected starting block indices, and count the occurence of each starting block index seen

3. Pick the starting block index with highest count

## Calc workflow

Calc mode only operates at UI/UX level and does not handle any file data, thus it is not documented here.

## Check workflow

1. A reference block is retrieved first and is used for guidance on alignment, version, and uid (see **Finding reference block** procedure specified above)
2. Scan for valid blocks from start of SBX container to decode and output using reference block's block size as alignment
     - if a block is invalid, and error message is shown
   
     - if a block is valid, nothing is done
   
     - By default, completely blank sections are ignored as they usually indicate gaps introduced by the burst error resistance pattern

## Decode workflow

Metadata block is valid if

- Metadata block validity criteria are satisfied (see **Block handling in general** above)
- Version and uid matches reference block (see below)

Data block is valid if and only if

- Basic block validity criteria are satisfied (see **Block handling in general** above)
- Version and uid matches reference block (see below)

### If output to file

1. A reference block is retrieved first and is used for guidance on alignment, version, and uid (see **Finding reference block** procedure specified above)
2. Scan for valid blocks from start of SBX container to decode and output using reference block's block size as alignment
     - if a block is invalid, nothing is done
     - if a block is valid, and is a metadata block, nothing is done
     - if a block is valid, and is a data parity block, nothing is done
     - if a block is valid, and is a data block, then it will be written to the writepos at output file, where writepos = (sequence number - 1) * block size of reference block in bytes
3. If possible, truncate output file to remove data padding done for the last block during encoding
     - if reference block is a metadata block, and contains file size field, and output is a file, then the output file will be truncated to that file size
     - otherwise nothing is done
     - If possible, report/record if the hash of decoded file matches the recorded hash during encoding
         - if reference block is a metadata block, and contains the hash field, and output is a file, then the output file will be hashed to check against the recorded hash
             - output file will not be deleted even if hash does not match
         - otherwise nothing is done

#### Handling of duplicate metadata/data blocks

- First valid metadata block will be used (if exists)
- For all other data blocks, the last seen valid data block will be used for a given sequence number

#### Handling of corrupted/missing blocks

- Corrupted blocks or missing blocks are not repaired in this mode
- User needs to invoke repair mode to repair the archive

### If output to stdout

##### Read pattern

Read pattern is one of

1. Burst error resistant

2. Sequential with burst error resistance awareness

3. Sequential with no burst error resistance awareness

##### Workflow

- A reference block is retrieved first and is used for guidance on alignment, version, and uid (see **Finding reference block** procedure specified above)

- Determine the read pattern
  
    - if container is RS enabled, then
      
        - if none of `--from`, `--to-exc` and `--to-inc` are specified, then read pattern is `1.`
      
        - else read pattern is `2.`
  
    - else read pattern is `3.`

- If read pattern is `1.`
  
    1. Go through metadata blocks in anticipated positions and try to decode. This is purely for statistics of successfully decoded metadata blocks
  
    2. Scan for valid blocks from the SBX container in the anticipated pattern to decode and output using reference block's block size as alignment
       
         - The anticipated pattern is same as the guessed encoding pattern, which depends on the SBX version, data parity parameters, guessed burst error resistance level
         - blkar halts after going through the last anticipated seq num
         - If a block is valid, and contains the anticipated seq num, then
             - if the block is a metadata block, then nothing is done
           
             - if the block is a data parity block, then nothing is done
           
             - if the block is a data block, then
               
                 - if blkar can determine the block is the last block, the data chunk of the block is truncated so the overall output size matches the original file size, then outputted to stdout
                   
                     - this is only possible when metadata block is used as reference block, and also contains the original file size
               
                 - else the data chunk of the block is outputted to stdout
           
             - else a blank chunk of the same size as a normal data chunk is outputted to stdout

- If read pattern is `2.` or `3.`
  
    1. The starting block index of the blocks to read is guessed first (see **Guessing starting block index** procedure specified above)
  
    2. Using the starting block index as the first block index, the anticipated seq num of each block index is calculated for each block read
       
         - if the block is valid and matches the anticipated seq num, then
           
             - if the block is a metadata block, do nothing
           
             - else if the block is a parity block, do nothing
           
             - else
               
                 - if blkar can determine the block is the last block, the data chunk of the block is truncated so the overall output size matches the original file size, then outputted to stdout
                     - this is only possible when metadata block is used as reference block, and also contains the original file size
       
         - else
           
             - if blkar can determine the block is the last block, a blank chunk is truncated so the overall output size matches the original file size, then outputted to stdout
                 - this is only possible when metadata block is used as reference block, and also contains the original file size
       
         - whichever the case, the chunk is used to update the hashing context if required
           
             - hashing context is only created if the container contains a stored hash and the hash type is supported
           
             - the hashing context is used to calculate the final hash displayed

## Encode workflow

1. If metadata is enabled, the following file metadata are gathered from file or retrieved from user input
     - file name
   
     - SBX file name
   
     - file size
   
     - file last modification time
   
     - encoding start time
2. If metadata is enabled, then a partial metadata block is written into the output file as filler
     - The written metadata block is valid, but does not contain the actual file hash, a filler pattern of 0x00 is used in place of the hash part of the multihash (the header and length indicator of the multihash are still valid)
3. Load version specific data sized chunk one at a time from input file to encode and output (and if metadata is enabled, Multihash hash state/ctx is updated as well - the actual hash state/ctx used depends on hash type, defaults to SHA256)
     - data size = block size - header size (e.g. version 1 has data size of 512 - 16 = 496)
     - if the seq num exceeds the maximum, the encoding procedure is terminated
     - If RS is enabled, then the RS codec is updated as needed
4. If metadata is enabled, the encoder seeks back to starting position of output file and overwrites the metadata block with one that contains the actual hash

##### Notes

- The work flow is the same whether input is file or stdin, as the reader used abstracts away the input type, and since the input is read purely sequentially, there was no need for different handling

## Repair workflow

Metadata block is valid if

- Metadata block validity criteria are satisfied (see **Block handling in general** above)
- Version and uid matches reference block (see below)

Data block is valid if and only if

- Basic block validity criteria are satisfied (see **Block handling in general** above)
- Version and uid matches reference block (see below)
1. A reference block is retrieved first and is used for guidance on alignment, version, and uid (see **Finding reference block** procedure specified above)
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
   
     - Only blocks which were missing/damaged then successfully repaired are written back, all other blocks are not touched
       
         - This means if a block cannot be repaired, then it is not touched

#### Handling of irreparable blocks

- Output sequence number of the blocks to log

## Rescue workflow

1. Scan for valid blocks from start of the provided file using 128 bytes alignment
     - rescue mode rescues all 3 versions of SBX blocks
   
     - if log file is specified, then
       
         - if the log file exists, then it will be used to initialize the scan's starting position
             - bytes_processed field will be rounded down to closest multiple of 128 automatically
   
     - the log file will be updated on every ~1.0 second
   
     - each block is appended to OUTDIR/UID, where :
       
         - OUTDIR = output directory specified
         - UID = uid of the block in hex (uppercase)
   
     - the original bytes in the file is used, that is, the output block bytes are not generated from scratch by blkar
2. User is expected to attempt to decode the rescued data in OUTDIR using the blkar decode command

## Show workflow

1. Scan for metadata blocks from start of provided file using 128 bytes alignment
     - if show all flag is supplied, all valid metadata blocks are displayed
   
     - else only the first valid metadata block are displayed
   
     - all displaying of blocks are immediate (no buffering of blocks)

## Sort workflow

Metadata block is valid if

- Metadata block validity criteria are satisfied (see **Block handling in general** above)
- Version and uid matches reference block (see below)

Data block is valid if and only if

- Basic block validity criteria are satisfied (see **Block handling in general** above)
- Version and uid matches reference block (see below)
1. A reference block is retrieved first and is used for guidance on alignment, version, and uid (see **Finding reference block** procedure specified above)
2. Read block from input file sequentailly, and write to position calculated from sequence number, block size and burst error resistance level to output file
     - The burst error resistance level by default is guessed using the **Guessing burst error resistance level** procedure specified above
   
     - The first metadata block is used for all metadata blocks in output container
   
     - The last valid data block is used for each sequence number

#### Handling of missing blocks

- Jumps/gaps caused by missing blocks are left to file system to handle (i.e. this may result in sparse file, or file with blank data in the gaps)

## Update workflow

Metadata block is valid if

- Metadata block validity criteria are satisfied (see **Block handling in general** above)

- Version and uid matches reference block (see below)
1. A reference block is retrieved first and is used for guidance on alignment, version, and uid (see **Finding reference block** procedure specified above)

2. Read metadata block from input file using the calculated positions
   
     - The burst error resistance level by default is guessed using the **Guessing burst error resistance level** procedure specified above
     - Metadata update/addition and removal is considered individually for each metadata block rather than overwriting other medatablocks using the first metadata block
     - For metadata update/addition
         - If the metadata field already exists, then it is replaced and stays in the same position (i.e. if the field is the $i$th field, then it remains as the $i$th field
         - If metadata field does not exist, then it is added as the last field
     - For metdata removal
         - If the metadata field exists, then it is removed and the remaining fields shift up in their positions
         - If the metadata field does not exist, then nothing is changed
     - Metadata update/addition process is done before removal process takes place
     - Field processing order in both update/addition and removal process
         - FNM
         - SNM

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

- Get enough valid SBX blocks of your container such that a successful decoding or repair may take place

## To successfully repair your SBX container

- The container has metadata block (or enough metadata parity blocks to reconstruct if corrupted/missing)
- The blocks' sequence numbers are in consistent order
- The container has enough valid parity blocks to correct all errors

## To successfully sort your SBX container

- There is space to store temporary file of same size at the specified destination
