## Technical Specification
The following specification is copied directly from the official specification with extensions.

Byte order: Big Endian
## For versions : 1, 2, 3
### Common blocks header:

| pos | to pos | size | desc                                |
|---- | ---    | ---- | ----------------------------------- |
|  0  |      2 |   3  | Recoverable Block signature = 'SBx' |
|  3  |      3 |   1  | Version byte |
|  4  |      5 |   2  | CRC-16-CCITT of the rest of the block (Version is used as starting value) |
|  6  |     11 |   6  | file UID                            |
| 12  |     15 |   4  | Block sequence number               |

### Block 0

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | n        | var  | encoded metadata |
|  n+1| blockend | var  | padding (0x1a)   |

### Blocks > 0 & < last:

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | data             |

### Blocks == last:

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | n        | var  | data             |
| n+1 | blockend | var  | padding (0x1a)   |

### Versions:

| ver | blocksize | note    |
|---- | --------- | ------- |
|  1  | 512       | default |
|  2  | 128       |         |
|  3  | 4096      |         |

### Metadata encoding:

| Bytes | Field | 
| ----- | ----- |
|    3  | ID    |
|    1  | Len   |
|    n  | Data  |

#### IDs

| ID | Desc |
| --- | --- |
| FNM | filename (utf-8) |
| SNM | sbx filename (utf-8) |
| FSZ | filesize (8 bytes) |
| FDT | date & time (8 bytes, seconds since epoch) |
| SDT | sbx date & time (8 bytes) |
| HSH | crypto hash (using [Multihash](http://multiformats.io) protocol) |
| PID | parent UID (*not used at the moment*)|

Supported crypto hashes since 1.0.0 are
  - SHA1
  - SHA256
  - SHA512
  - BLAKE2B\_512

## For versions : 11, 12, 13
Overall similar to above specs.

Assumes configuration is **N** data shards and **M** parity shards.

### Common blocks header:

| pos | to pos | size | desc                                |
|---- | ---    | ---- | ----------------------------------- |
|  0  |      2 |   3  | Recoverable Block signature = 'SBx' |
|  3  |      3 |   1  | Version byte |
|  4  |      5 |   2  | CRC-16-CCITT of the rest of the block (Version is used as starting value) |
|  6  |     11 |   6  | file UID                            |
| 12  |     15 |   4  | Block sequence number               |

### Block 0

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | n        | var  | encoded metadata |
|  n+1| blockend | var  | padding (0x1a)   |

### Block 1-2

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | parity           |

RS arrangement : block 0 (data shard) block 1 (parity shard) block 2 (parity shard).

Above gives 200% redundancy for the metadata block.

### Blocks >= 3 & < 3 + K * (N + M), where K is an integer >= 1:

For **N** continuous blocks

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | data             |

For **M** continuous blocks

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | parity           |

RS arrangement : N blocks (N data shards) M blocks (M parity shards).

### Last set of blocks

For **X** continuous blocks, where **X** is the remaining number of data blocks

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | data             |

For **ceil(X * M / N)** continuous blocks

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | parity           |

RS arrangement : X blocks (X data shards) Y blocks (Y parity shards), where Y = ceil(X * M / N).

### Versions:

| ver | blocksize | note    |
|---- | --------- | ------- |
| 11  | 512       |         |
| 12  | 128       |         |
| 13  | 4096      |         |

### Metadata encoding:

| Bytes | Field | 
| ----- | ----- |
|    3  | ID    |
|    1  | Len   |
|    n  | Data  |

#### IDs

| ID | Desc |
| --- | --- |
| FNM | filename (utf-8) |
| SNM | sbx filename (utf-8) |
| FSZ | filesize (8 bytes) |
| FDT | date & time (8 bytes, seconds since epoch) |
| SDT | sbx date & time (8 bytes) |
| HSH | crypto hash (using [Multihash](http://multiformats.io) protocol) |
| PID | parent UID (*not used at the moment*)|
| RSD | Reed-Solomon data shards part of ratio (ratio = RSD : RSP) |
| RSP | Reed-Solomon parity shards part of ratio (ratio = RSD : RSP) |

Supported forward error correction algorithms since 1.0.0 are
  - Reed-Solomon erasure code - probably the only one for versions 11, 12, 13
