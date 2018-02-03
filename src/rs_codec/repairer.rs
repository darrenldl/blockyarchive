use super::super::reed_solomon_erasure::ReedSolomon;
use super::super::smallvec::SmallVec;
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
        let last_data_set_parity_count   = calc_parity_shards(data_shards,
                                                              parity_shards,
                                                              last_data_set_size);

        let block_size = ver_to_block_size(version);

        let total_blocks = calc_total_blocks(data_shards,
                                             parity_shards,
                                             total_data_chunks);

        let buf_normal : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; data_shards + parity_shards];
        let buf_normal_slice_present : SmallVec<[bool; 32]> =
            smallvec![false; data_shards + parity_shards];

        let buf_last : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; last_data_set_size + last_data_set_parity_count];
        let buf_last_slice_present : SmallVec<[bool; 32]> =
            smallvec![false; last_data_set_size + last_data_set_parity_count];

        RSRepairer {
            cur_seq_num            : 0,
            last_block_set_start_seq_num,
            rs_codec_normal        :
            if total_data_chunks == 0 { None }
            else { Some(ReedSolomon::new(data_shards,
                                         parity_shards).unwrap()) },
            rs_codec_last          :
            if total_data_chunks == 0 { None }
            else { Some(ReedSolomon::new(last_data_set_size,
                                         last_data_set_parity_count).unwrap()) },
            dat_num_normal : data_shards,
            par_num_normal : parity_shards,
            dat_num_last   : last_data_set_size,
            par_num_last   : last_data_set_parity_count,
            total_blocks,
            version,
            buf_normal,
            buf_normal_slice_present,
            buf_last,
            buf_last_slice_present,
            block_type,
            ref_block              : ref_block.clone(),
        }
    }

    pub fn repair(&mut self,
                  data_only : bool) -> Result<(), Error> {
        let rs_codec_normal = match self.rs_codec_normal {
            None        => { return Ok(()); },
            Some(ref r) => r,
        };
        let rs_codec_last   = match self.rs_codec_last {
            None        => { return Ok(()); },
            Some(ref r) => r
        };

        let block_pred = |block : &Block| -> bool {
            block.is_meta()
        };

        if self.cur_seq_num < self.last_block_set_start_seq_num {
            for i in 0..self.dat_num_normal + self.par_num_normal {
                self.buf_normal_slice_present[i] =
                    self.ref_block.check_if_buffer_valid(&self.buf_normal[i]);
            }
            let mut buf : SmallVec<[&mut [u8]; 32]> =
                convert_2D_slices!(self.buf_normal =>to_mut SmallVec<[&mut [u8]; 32]>,
                                   SmallVec::with_capacity);
            let res = if data_only {
                rs_codec_normal.reconstruct_data(&mut buf,
                                                 &self.buf_normal_slice_present)
            } else {
                rs_codec_normal.reconstruct(&mut buf,
                                            &self.buf_normal_slice_present)
            };
            match res {
                Ok(()) => Ok(()),
                Err(_) => Err(to_err(RSError::new(RSErrorKind::RepairFail,
                                                  self.version,
                                                  self.cur_seq_num,
                                                  self.dat_num_normal + self.par_num_normal,
                                                  self.block_type,
                                                  &self.buf_normal_slice_present)))
            }
        } else {
            for i in 0..self.dat_num_last + self.par_num_last {
                self.buf_last_slice_present[i] =
                    self.ref_block.check_if_buffer_valid(&self.buf_last[i]);
            }
            let mut buf : SmallVec<[&mut [u8]; 32]> =
                convert_2D_slices!(self.buf_last =>to_mut SmallVec<[&mut [u8]; 32]>,
                                   SmallVec::with_capacity);
            let res = if data_only {
                rs_codec_last.reconstruct_data(&mut buf,
                                               &self.buf_last_slice_present)
            } else {
                rs_codec_last.reconstruct(&mut buf,
                                          &self.buf_last_slice_present)
            };
            match res {
                Ok(()) => Ok(()),
                Err(_) => Err(to_err(RSError::new(RSErrorKind::RepairFail,
                                                  self.version,
                                                  self.cur_seq_num,
                                                  self.dat_num_last + self.par_num_last,
                                                  self.block_type,
                                                  &self.buf_last_slice_present)))
            }
        }
    }
}
