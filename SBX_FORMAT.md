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
| FSZ | filesize (8 bytes - BE uint64) |
| FDT | date & time (8 bytes - BE int64, seconds since epoch) |
| SDT | sbx date & time (8 bytes - BE int64) |
| HSH | crypto hash (using [Multihash](http://multiformats.io) protocol) |
| PID | parent UID (*not used at the moment*)|

Supported crypto hashes since 1.0.0 are
  - SHA1
  - SHA256
  - SHA512
  - BLAKE2B\_512

Metadata block (block 0) can be disabled

## For versions : 17(0x11), 18(0x12), 19(0x13)
Overall similar to above specs.

Block categories : `Meta`, `Data`, `Parity`

**`Meta` and `Data`** are mutually exclusive, and **`Meta` and `Parity`** are mutually exclusive. A block can be both `Data` and `Parity`.

Assumes configuration is **M** data shards and **N** parity shards.

### Note
The following only describes the sequence number arrangement, not the actual block arrangement.

See section "Block set interleaving scheme" below for details on actual block arrangement.

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

Block 0 is `Meta` only.

### Blocks >= 1 & < 1 + K * (M + N), where K is an integer >= 1:

For **M** continuous blocks

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | data             |

For **N** continuous blocks

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | parity           |

RS arrangement : M blocks (M data shards) N blocks (N parity shards).

The M blocks are `Data` only.

The N blocks are both `Data` and `Parity`.

### Last set of blocks

For **X** continuous blocks, where **X** is the remaining number of data blocks

#### Blocks in **first X - 1**:

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | data             |

#### Last block

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | n        | var  | data             |
| n+1 | blockend | var  | padding (0x1a)   |

For **M - X** continuous blocks, where **M** is the specified data shards count.

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | padding (0x1a)   |

For **N** continuous blocks

| pos | to pos   | size | desc             |
|---- | -------- | ---- | ---------------- |
| 16  | blockend | var  | parity           |

RS arrangement : M blocks (X data shards + (M - X) padding blocks) N blocks.

The M blocks are `Data` only.

The N blocks are both `Data` and `Parity`.

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
| FSZ | filesize (8 bytes - BE uint64) |
| FDT | date & time (8 bytes - BE int64, seconds since epoch) |
| SDT | sbx date & time (8 bytes - BE int64) |
| HSH | crypto hash (using [Multihash](http://multiformats.io) protocol) |
| PID | parent UID (*not used at the moment*)|
| RSD | Reed-Solomon data shards part of ratio (ratio = RSD : RSP) (1 byte - uint8) |
| RSP | Reed-Solomon parity shards part of ratio (ratio = RSD : RSP) (1 byte - uint8) |

Supported forward error correction algorithms since 1.0.0 are
  - Reed-Solomon erasure code - probably the only one for versions 17, 18, 19

Metadata and the parity blocks (blocks 0 - 3) are mandatory

### Block set interleaving scheme
This block set interleaving is heavily inspired by [Thanassis Tsiodras's design of RockFAT](https://www.thanassis.space/RockFAT.html).

The major difference between the two schemes is that RockFAT's one is byte based interleaving, rsbx's one is SBX block based interleaving.

The other difference is that rsbx allows customizing level of resistance against burst sector errors.

A burst error is defined as consecutive SBX block erasures.

Burst error resistance is defined as the maximum number of consective SBX block erasures tolerable for any instance of burst error.

The maximum number of such errors tolerable is same as the parity shard count.

Assuming arrangement of **M** data shards, **N** parity shards, **B** burst error resistance.

Then the SBX container can tolerate up to **N** burst errors, and each individual error may be up to **B** SBX blocks.

#### Diagrams

General case

Sequential arrangement, **M** data shards, **N** parity shards, **B** burst error resistance

| 00 | 01 | 02 | 03 | 04 | ...

Interleaving arrangement

Let base block set size = **M + N**

First **1 + N** block sets have size = 1 + base block set size, the rest have size = base block set size

First **1 + N** block sets :

| 0  | 1 | 2 | 3 | ... | B - 1 |
|---|---|---|---|---|---|
| 00 | 01 | 01 + (M + N) | 01 + 2 * (M + N) | ... | 01 + (B - 1) * (M + N) |
| 00 | 02 | 02 + (M + N) | 02 + 2 * (M + N) | ... | 02 + (B - 1) * (M + N) |
| ...|...|...|...|...|...|
| 00 | 1 + N | (1 + N) + (M + N) | (1 + N) + 2 * (M + N) | ... | (1 + N) + (B - 1) * (M + N) |

Rest of the block sets :

Let **K > 1 + N** :

| 0 | 1 | 2 | 3 | ... | B - 2 |
|---|---|---|---|---|---|
| K | K + (M + N) | K + 2 * (M + N) | ... | K + (B - 1) * (M + N) |
