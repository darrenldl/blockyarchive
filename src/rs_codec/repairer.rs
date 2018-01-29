use super::super::reed_solomon_erasure::ReedSolomon;
use super::super::smallvec::SmallVec;
use super::super::sbx_block;
use super::super::sbx_specs::Version;
use super::super::sbx_block::BlockType;
use super::super::sbx_block::Block;
use super::super::sbx_specs::ver_to_block_size;
use super::super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::*;

use super::Error;

pub struct RSRepairer {
    cur_seq_num                  : u64,
    last_block_set_start_seq_num : u64,
    rs_codec_normal              : Option<ReedSolomon>,
    rs_codec_last                : Option<ReedSolomon>,
    dat_num_normal               : usize,
    par_num_normal               : usize,
    dat_num_last                 : usize,
    par_num_last                 : usize,
    total_blocks                 : u64,
    version                      : Version,
    buf_normal                   : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_normal_slice_present     : SmallVec<[bool; 32]>,
    buf_last                     : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_last_slice_present       : SmallVec<[bool; 32]>,
    block_type                   : BlockType,
    ref_block                    : Block,
}

impl RSRepairer {
    pub fn new(version           : Version,
               ref_block         : &Block,
               block_type        : BlockType,
               data_shards       : usize,
               parity_shards     : usize,
               total_data_chunks : u64) -> RSRepairer {
        let last_data_set_size           =
            last_data_set_size(data_shards,
                               total_data_chunks);
        let last_block_set_start_seq_num =
            last_block_set_start_seq_num(data_shards,
                                         parity_shards,
                                         total_data_chunks);
        let last_block_set_seq_num       =
            last_block_set_seq_num(data_shards,
                                   total_data_chunks);
        let last_data_set_parity_count   = calc_parity_shards(data_shards,
                                                              parity_shards,
                                                              last_data_set_size);

        let block_size = ver_to_block_size(version);

        let buf_normal : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; data_shards + parity_shards];

        let buf_last : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; last_data_set_size + last_data_set_parity_count];

        RSRepairer {
            cur_seq_num            : 0,
            last_block_set_start_seq_num,
            rs_codec               :
            if total_data_chunks == 0 { None }
            else { Some(ReedSolomon::new(data_shards,
                                         parity_shards).unwrap()) },
            rs_codec_last          :
            if total_data_chunks == 0 { None }
            else { Some(ReedSolomon::new(last_data_set_size,
                                         last_data_set_parity_count).unwrap()) },
            data_shards,
            parity_shards,
            total_blocks,
            version,
            buf_normal,
            buf_last,
            block_type,
            ref_block              : ref_block.clone(),
        }
    }

    pub fn repair(&mut self) -> Result<(), Error> {
        let rs_codec_normal = match self.rs_codec_normal {
            None        => { return None; },
            Some(ref r) => r,
        };
        let rs_codec_last   = match self.rs_codec_last {
            None        => { return None; },
            Some(ref r) => r
        };

        if self.cur_index < self.last_set_start_seq_num {
            for i in 0..self.data_shards + self.parity_shards {
                self.buf_normal_slice_present[i] =
                    self.ref_block.check_if_buffer_valid(&self.buf_normal[i],
                                                         Some(self.block_type));
            }
            let buf : SmallVec<[&mut [u8]; 32]> =
                convert_2D_slices!(self.buf_normal =>to_mut SmallVec<[&mut [u8]; 32]>,
                                   SmallVec::with_capacity);
            rs_codec_normal.recontruct(&mut buf,
                                       &self.buf_normal_slice_present);
        } else {
            let last_set_parity_count = calc_parity_shards(self.data_shards,
                                                           self.parity_shards,
                                                           self.last_data_set_size);
            for i in 0..self.last_data_set_size + last_set_parity_count {
                self.buf_last_slice_present[i] =
                    self.ref_block.check_if_buffer_valid(&self.buf_last[i],
                                                         Some(self.block_type));
            }
            let buf : SmallVec<[&mut [u8]; 32]> =
                convert_2D_slices!(self.buf_last =>to_mut SmallVec<[&mut [u8]; 32]>,
                                   SmallVec::with_capacity);
            rs_codec_normal.recontruct(&mut buf,
                                       &self.buf_last_slice_present);
        }
    }
}
