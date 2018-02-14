mod encoder;
pub use self::encoder::RSEncoder;

use super::smallvec::SmallVec;

use super::sbx_block::BlockType;
use super::sbx_specs::Version;

use super::sbx_specs::ver_to_block_size;

use std::fmt;

mod repairer;
pub use self::repairer::RSRepairer;

mod indexer;

mod test;

use super::Error;
use super::ErrorKind;

use super::sbx_specs::SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM;

pub fn calc_parity_shards(data_shards   : usize,
                          parity_shards : usize,
                          set_size      : usize) -> usize {
    (set_size * parity_shards + (data_shards - 1)) / data_shards
}

pub mod from_data_block_count {
    use super::*;

    pub fn last_data_set_start_index(data_shards       : usize,
                                     total_data_chunks : u32) -> u32 {
        total_data_chunks - last_data_set_size(data_shards,
                                               total_data_chunks) as u32
    }

    pub fn last_data_set_size(data_shards       : usize,
                              total_data_chunks : u32) -> usize {
        let size = total_data_chunks % data_shards as u32;

        if size == 0 { data_shards as usize }
        else         { size        as usize }
    }

    pub fn last_block_set_start_seq_num(data_shards       : usize,
                                        parity_shards     : usize,
                                        total_data_chunks : u32) -> u32 {
        let last_data_set_size = last_data_set_size(data_shards,
                                                    total_data_chunks) as u32;

        // Cannot just do total_data_chunks / data_shards
        // as the first data set can also be the last data set,
        // in which case normal_set_count would be 0, last_data_set_count would be 1
        let normal_set_count =
            (total_data_chunks - last_data_set_size) / data_shards as u32;

        SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32
            + normal_set_count * (data_shards + parity_shards) as u32
    }

    pub fn last_block_set_size(data_shards       : usize,
                               parity_shards     : usize,
                               total_data_chunks : u32) -> u32 {
        let last_data_set_size = last_data_set_size(data_shards,
                                                    total_data_chunks);

        (last_data_set_size + calc_parity_shards(data_shards,
                                                 parity_shards,
                                                 last_data_set_size)) as u32
    }

    pub fn calc_total_blocks (data_shards       : usize,
                              parity_shards     : usize,
                              total_data_chunks : u32) -> u32 {
        let last_block_set_start_seq_num =
            last_block_set_start_seq_num(data_shards,
                                         parity_shards,
                                         total_data_chunks);
        let last_block_set_size           =
            last_block_set_size(data_shards,
                                parity_shards,
                                total_data_chunks) as u32;

        SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32
            + last_block_set_start_seq_num + last_block_set_size
    }
}

pub mod from_total_block_count {
    pub fn last_block_set_start_seq_num(data_shards   : usize,
                                        parity_shards : usize,
                                        total_blocks  : u32) -> u32 {
        total_blocks - last_block_set_size(data_shards,
                                           parity_shards,
                                           total_blocks) as u32
    }

    pub fn last_block_set_size(data_shards   : usize,
                               parity_shards : usize,
                               total_blocks  : u32) -> usize {
        let size = total_blocks % (data_shards + parity_shards) as u32;
        if size == 0 {
            data_shards + parity_shards
        } else {
            size as usize
        }
    }

    pub fn seq_num_is_parity(data_shards   : usize,
                             parity_shards : usize,
                             total_blocks  : u32) -> bool {
        false
    }
}

#[derive(Clone)]
pub enum RSErrorKind {
    RepairFail,
    VerifyFail,
}

#[derive(Clone)]
pub struct RSError {
    kind                : RSErrorKind,
    version             : Version,
    block_seq_num_start : u32,
    block_count         : u32,
    block_type          : BlockType,
    shard_present       : SmallVec<[bool; 32]>,
}

fn to_err(e : RSError) -> Error {
    Error::new(ErrorKind::RSError(e))
}

impl RSError {
    pub fn new(kind                : RSErrorKind,
               version             : Version,
               block_seq_num_start : u32,
               block_count         : u32,
               block_type          : BlockType,
               shard_present       : Option<&[bool]>) -> RSError {
        let mut shard_present_vec : SmallVec<[bool; 32]> =
            SmallVec::with_capacity(block_count as usize);
        match shard_present {
            None => {},
            Some(shard_present) => {
                for s in shard_present.iter() {
                    shard_present_vec.push(*s);
                }
            }
        }
        RSError {
            kind,
            version,
            block_seq_num_start,
            block_count,
            block_type,
            shard_present : shard_present_vec
        }
    }
}

impl fmt::Display for RSError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use self::RSErrorKind::*;
        match self.kind {
            RepairFail => {
                let mut msg = String::with_capacity(20);
                let block_size = ver_to_block_size(self.version) as u32;
                let block_seq_num_start  = self.block_seq_num_start;
                let block_seq_num_end    = block_seq_num_start + self.block_count - 1;
                let file_pos_first_block = block_seq_num_start * block_size;
                let file_pos_last_block  = block_seq_num_end   * block_size;
                msg.push_str(&format!("too few blocks present to repair blocks {} - {} (file pos : {} (0x{:X}) - {} (0x{:X}))\n",
                                      block_seq_num_start,
                                      block_seq_num_end,
                                      file_pos_first_block,
                                      file_pos_first_block,
                                      file_pos_last_block,
                                      file_pos_last_block,
                ));
                msg.push_str("missing/corrupted : ");
                let mut first_num = true;
                for i in 0..self.shard_present.len() {
                    if !self.shard_present[i] {
                        if first_num {
                            msg.push_str(&format!("{}", i));
                            first_num = false;
                        } else {
                            msg.push_str(&format!(", {}", i));
                        }
                    }
                }
                write!(f, "{}", msg)
            },
            VerifyFail => {
                let mut msg = String::with_capacity(20);
                let block_size = ver_to_block_size(self.version) as u32;
                let block_seq_num_start  = self.block_seq_num_start;
                let block_seq_num_end    = block_seq_num_start + self.block_count - 1;
                let file_pos_first_block = block_seq_num_start * block_size;
                let file_pos_last_block  = block_seq_num_end   * block_size;
                msg.push_str(&format!("failed to verify blocks {} - {} (file pos : {} (0x{:X}) - {} (0x{:X}))\n",
                                      block_seq_num_start,
                                      block_seq_num_end,
                                      file_pos_first_block,
                                      file_pos_first_block,
                                      file_pos_last_block,
                                      file_pos_last_block,
                ));
                write!(f, "{}", msg)
            }
        }
    }
}
