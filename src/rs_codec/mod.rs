mod encoder;
pub use self::encoder::RSEncoder;

use super::smallvec::SmallVec;

use super::sbx_block::BlockType;
use super::sbx_specs::Version;

use super::sbx_specs::ver_to_block_size;

use std::fmt;

mod repairer;
pub use self::repairer::RSRepairer;

mod tests;

use super::Error;
use super::ErrorKind;

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
                let block_size = ver_to_block_size(self.version) as u32;
                let block_seq_num_start  = self.block_seq_num_start;
                let block_seq_num_end    = block_seq_num_start + self.block_count - 1;
                let file_pos_first_block = block_seq_num_start * block_size;
                let file_pos_last_block  = block_seq_num_end   * block_size;
                writeln!(f, "too few blocks present to repair blocks {} - {} (file pos : {} (0x{:X}) - {} (0x{:X}))\n",
                         block_seq_num_start,
                         block_seq_num_end,
                         file_pos_first_block,
                         file_pos_first_block,
                         file_pos_last_block,
                         file_pos_last_block);
                writeln!(f, "missing/corrupted : ");
                let mut first_num = true;
                for i in 0..self.shard_present.len() {
                    if !self.shard_present[i] {
                        if first_num {
                            writeln!(f, "{}", i);
                            first_num = false;
                        } else {
                            wrteln!(f, ", {}", i);
                        }
                    }
                }
            },
            VerifyFail => {
                let block_size = ver_to_block_size(self.version) as u32;
                let block_seq_num_start  = self.block_seq_num_start;
                let block_seq_num_end    = block_seq_num_start + self.block_count - 1;
                let file_pos_first_block = block_seq_num_start * block_size;
                let file_pos_last_block  = block_seq_num_end   * block_size;
                writeln!(f, "failed to verify blocks {} - {} (file pos : {} (0x{:X}) - {} (0x{:X}))\n",
                         block_seq_num_start,
                         block_seq_num_end,
                         file_pos_first_block,
                         file_pos_first_block,
                         file_pos_last_block,
                         file_pos_last_block);
            }
        }
    }
}
