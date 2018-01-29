mod encoder;
pub use self::encoder::RSEncoder;

use super::smallvec::SmallVec;

//mod repairer;
//use repairer::RSRepairer;

use super::Error;
use super::ErrorKind;

fn last_data_set_start_index(data_shards       : usize,
                             total_data_blocks : u64) -> u64 {
    total_data_blocks - last_data_set_size(data_shards, total_data_blocks) as u64
}

fn last_data_set_size(data_shards       : usize,
                      total_data_blocks : u64) -> usize {
    let size = total_blocks % data_shards as u64;
    if size == 0 {
        data_shards as usize
    } else {
        size as usize
    }
}

fn last_block_set_start_seq_num(data_shards       : usize,
                                parity_shards     : usize,
                                total_data_blocks : u64) -> u64 {
    let normal_set_count = total_data_blocks / data_shards as u64;

    normal_set_count * (data_shards + parity_shards) as u64
}

fn calc_parity_shards(data_shards   : usize,
                      parity_shards : usize,
                      set_size      : usize) -> usize {
    (set_size * parity_shards + (data_shards - 1)) / data_shards
}

#[derive(Clone)]
pub struct RSError {
    version             : Version,
    block_seq_num_start : u64,
    block_count         : usize,
    block_type          : BlockType,
    shard_present       : SmallVec<[bool; 32]>,
}

fn to_err(e : RSError) -> Error {
    Error::new(ErrorKind::RSError(e))
}

impl RSError {
    pub fn new(version             : Version,
               block_seq_num_start : u64,
               block_count         : usize,
               block_type          : BlockType,
               shard_present       : &[bool]) -> RSError {
        let mut shard_present_vec : SmallVec<[bool; 32]> =
            SmallVec::with_capacity(block_count);
        for s in shard_present.iter() {
            shard_present_vec.push(*s);
        }
        RSError {
            version,
            block_seq_num_start,
            block_count,
            block_type,
            shard_present : shard_present_vec
        }
    }
}
