use super::super::reed_solomon_erasure::ReedSolomon;
use super::super::smallvec::SmallVec;
use super::super::sbx_specs::Version;
use super::super::sbx_block::BlockType;
use super::super::sbx_block::Block;
use super::super::sbx_specs::ver_to_block_size;
use super::super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::super::sbx_specs::SBX_FIRST_DATA_SEQ_NUM;
use super::*;
use super::super::sbx_block;

use std::fmt;

use super::Error;

pub struct RSRepairer {
    index          : usize,
    rs_codec       : ReedSolomon,
    data_par_burst : Option<(usize, usize, usize)>,
    version        : Version,
    buf            : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_present    : SmallVec<[bool; 32]>,
    ref_block      : Block,
}

pub struct RSRepairStats<'a> {
    pub successful    : bool,
    pub start_seq_num : u32,
    pub present       : &'a SmallVec<[bool; 32]>,
    pub missing_count : usize,
    pub present_count : usize,
}

impl<'a> fmt::Display for RSRepairStats<'a> {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        if self.missing_count > 0 {
            if self.successful {
                write!(f, "Repair successful for ")?;
            } else {
                write!(f, "Repair failed     for ")?;
            }

            write!(f, "block set [{} - {}], block no. : ",
                   self.start_seq_num,
                   self.start_seq_num + self.present.len() as u32 - 1)?;

            let mut first_num = true;
            for i in 0..self.present.len() {
                if !self.present[i] {
                    if !first_num {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", self.start_seq_num + i as u32)?;

                    first_num = false;
                }
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

macro_rules! add_index {
    (
        $self:ident, $val:expr
    ) => {{
        $self.index =
            ($self.index + $val) % $self.rs_codec.total_shard_count();
    }};
    (
        1 => $self:ident
    ) => {{
        add_index!($self, 1);
    }}
}

impl RSRepairer {
    pub fn new(version       : Version,
               ref_block     : &Block,
               data_shards   : usize,
               parity_shards : usize,
               burst         : usize) -> RSRepairer {
        let block_size = ver_to_block_size(version);

        let buf : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; data_shards + parity_shards];
        let buf_present : SmallVec<[bool; 32]> =
            smallvec![false; data_shards + parity_shards];

        RSRepairer {
            index          : 0,
            rs_codec       : ReedSolomon::new(data_shards,
                                           parity_shards).unwrap(),
            data_par_burst : Some((data_shards, parity_shards, burst)),
            version,
            buf,
            buf_present,
            ref_block      : ref_block.clone(),
        }
    }

    pub fn get_block_buffer(&mut self) -> &mut [u8] {
        let index = self.index;

        self.buf_present[index] = true;

        sbx_block::slice_buf_mut(self.version, &mut self.buf[index])
    }

    pub fn mark_present(&mut self) -> RSCodecState {
        let index = self.index;

        add_index!(1 => self);

        self.buf_present[index] = true;

        if index == self.rs_codec.total_shard_count() - 1 {
            RSCodecState::Ready
        } else {
            RSCodecState::NotReady
        }
    }

    pub fn mark_missing(&mut self) -> RSCodecState {
        let index = self.index;

        add_index!(1 => self);

        self.buf_present[index] = false;

        if index == self.rs_codec.total_shard_count() - 1 {
            RSCodecState::Ready
        } else {
            RSCodecState::NotReady
        }
    }

    pub fn missing_count(&self) -> usize {
        self.rs_codec.total_shard_count() - self.present_count()
    }

    pub fn present_count(&self) -> usize {
        let mut count = 0;
        for p in self.buf_present.iter() {
            if *p { count += 1; }
        }
        count
    }

    pub fn repair_with_block_sync(&mut self,
                                  seq_num : u32)
                                  ->
        (RSRepairStats,
         SmallVec<[(u64, &[u8]); 32]>)
    {
        assert_eq!(0, self.index);

        let mut repaired_blocks =
            SmallVec::with_capacity(self.rs_codec.parity_shard_count());

        let rs_codec      = &self.rs_codec;

        let successful;
        {
            let mut buf : SmallVec<[&mut [u8]; 32]> =
                SmallVec::with_capacity(rs_codec.total_shard_count());
            for s in self.buf.iter_mut() {
                buf.push(sbx_block::slice_data_buf_mut(self.version, s));
            }

            // reconstruct data portion
            successful =
                match rs_codec.reconstruct(&mut buf, &self.buf_present) {
                    Ok(()) => true,
                    Err(_) => false
                };
        }

        let block_set_size = self.rs_codec.total_shard_count() as u32;

        let data_index = seq_num - SBX_FIRST_DATA_SEQ_NUM;

        let block_set_index = data_index / block_set_size;

        let first_data_index_in_cur_set = block_set_index * block_set_size;

        let first_seq_num_in_cur_set = first_data_index_in_cur_set + 1;

        // reconstruct header if successful
        if successful {
            for i in 0..block_set_size as usize {
                if !self.buf_present[i] {
                    self.ref_block.set_seq_num(first_seq_num_in_cur_set + i as u32);
                    self.ref_block.sync_to_buffer(None,
                                                  &mut self.buf[i]).unwrap();
                }
            }
            for i in 0..block_set_size as usize {
                let cur_seq_num = first_seq_num_in_cur_set + i as u32;
                if !self.buf_present[i] {
                    let pos = sbx_block::calc_data_block_write_pos(self.version,
                                                                   cur_seq_num,
                                                                   self.data_par_burst);
                    repaired_blocks.push((pos, sbx_block::slice_buf(self.version,
                                                                    &self.buf[i])));
                }
            }
        }

        (RSRepairStats {
            successful,
            start_seq_num : first_seq_num_in_cur_set,
            present       : &self.buf_present,
            missing_count : self.missing_count(),
            present_count : self.present_count(), },
         repaired_blocks)
    }
}
