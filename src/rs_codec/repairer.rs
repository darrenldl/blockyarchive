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

use super::Error;

pub struct RSRepairer {
    index          : usize,
    rs_codec       : ReedSolomon,
    version        : Version,
    buf            : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_par_verify : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_present    : SmallVec<[bool; 32]>,
    ref_block      : Block,
}

#[must_use]
pub enum CodecState {
    Ready,
    NotReady
}

pub struct RSStats<'a> {
    pub successful    : bool,
    pub start_seq_num : u32,
    pub present       : &'a SmallVec<[bool; 32]>
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
    pub fn new(version           : Version,
               ref_block         : &Block,
               data_shards       : usize,
               parity_shards     : usize) -> RSRepairer {
        let block_size = ver_to_block_size(version);

        let buf : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; data_shards + parity_shards];
        let buf_present : SmallVec<[bool; 32]> =
            smallvec![false; data_shards + parity_shards];

        let buf_par_verify : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; parity_shards];

        RSRepairer {
            index          : 0,
            rs_codec       : ReedSolomon::new(data_shards,
                                              parity_shards).unwrap(),
            version,
            buf,
            buf_par_verify,
            buf_present,
            ref_block      : ref_block.clone(),
        }
    }

    pub fn get_block_buffer(&mut self) -> &mut [u8] {
        let index = self.index;

        self.buf_present[index] = true;

        sbx_block::slice_data_buf_mut(self.version, &mut self.buf[index])
    }

    pub fn mark_present(&mut self) -> CodecState {
        let index = self.index;

        add_index!(1 => self);

        self.buf_present[index] = true;

        if index == self.rs_codec.total_shard_count() - 1 {
            CodecState::Ready
        } else {
            CodecState::NotReady
        }
    }

    pub fn mark_missing(&mut self) -> CodecState {
        let index = self.index;

        add_index!(1 => self);

        self.buf_present[index] = false;

        if index == self.rs_codec.total_shard_count() - 1 {
            CodecState::Ready
        } else {
            CodecState::NotReady
        }
    }

    pub fn missing_count(&self) -> usize {
        let mut count = 0;
        for p in self.buf_present.iter() {
            if *p { count += 1; }
        }
        count
    }

    pub fn present_count(&self) -> usize {
        self.rs_codec.total_shard_count() - self.missing_count()
    }

    pub fn repair(&mut self,
                  seq_num : u32) -> RSStats {
        assert_eq!(0, self.index);

        let rs_codec      = &self.rs_codec;

        let mut buf : SmallVec<[&mut [u8]; 32]> =
            SmallVec::with_capacity(rs_codec.total_shard_count());
        for s in self.buf.iter_mut() {
            buf.push(sbx_block::slice_data_buf_mut(self.version, s));
        }

        // reconstruct data portion
        let successful =
            match rs_codec.reconstruct(&mut buf, &self.buf_present) {
                Ok(()) => true,
                Err(_) => false
            };

        let block_set_size = self.rs_codec.total_shard_count() as u32;

        let data_index = seq_num - SBX_FIRST_DATA_SEQ_NUM;

        let block_set_index = data_index / block_set_size;

        let first_data_index_in_cur_set = block_set_index * block_set_size;

        let first_seq_num_in_cur_set = first_data_index_in_cur_set + 1;

        // reconstruct header if successful
        if successful {
            for i in 0..block_set_size {
                if !self.buf_present[i] {
                    self.ref_block.set_seq_num(first_seq_num_in_cur_set + i);
                    self.ref_block.sync_to_buffer(&mut self.buf[i]);
                }
            }
        }

        RSStats {
            successful,
            start_seq_num : first_seq_num_in_cur_set,
            present       : &self.buf_present,
        }
    }
}
