use super::super::reed_solomon_erasure::ReedSolomon;
use super::super::smallvec::SmallVec;
use super::super::sbx_specs::Version;
use super::super::sbx_block::BlockType;
use super::super::sbx_block::Block;
use super::super::sbx_specs::ver_to_block_size;
use super::super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::super::sbx_specs::SBX_RS_METADATA_PARITY_COUNT;
use super::*;
use super::super::sbx_block;

use super::Error;

pub struct RSRepairer {
    active                       : bool,
    cur_seq_num                  : u64,
    start_seq_num                : u64,
    last_block_set_start_seq_num : u64,
    rs_codec_normal              : ReedSolomon,
    rs_codec_last                : ReedSolomon,
    total_blocks                 : u64,
    version                      : Version,
    buf_normal                   : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_normal_par_verify        : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_normal_slice_present     : SmallVec<[bool; 32]>,
    buf_last                     : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_last_par_verify          : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
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

        let buf_normal_par_verify : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; parity_shards];
        let buf_last_par_verify : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; last_data_set_parity_count];

        let start_seq_num = match block_type {
            BlockType::Meta => 0,
            BlockType::Data => 1 + SBX_RS_METADATA_PARITY_COUNT,
        } as u64;

        RSRepairer {
            cur_seq_num            : start_seq_num,
            start_seq_num,
            last_block_set_start_seq_num,
            rs_codec_normal        :
            if total_data_chunks == 0 { None }
            else { Some(ReedSolomon::new(data_shards,
                                         parity_shards).unwrap()) },
            rs_codec_last          :
            if total_data_chunks == 0 { None }
            else { Some(ReedSolomon::new(last_data_set_size,
                                         last_data_set_parity_count).unwrap()) },
            total_blocks,
            version,
            buf_normal,
            buf_normal_par_verify,
            buf_normal_slice_present,
            buf_last,
            buf_last_par_verify,
            buf_last_slice_present,
            block_type,
            ref_block              : ref_block.clone(),
        }
    }

    fn in_normal_block_set(&self) -> bool {
        self.cur_seq_num < self.last_block_set_start_seq_num
    }

    fn pick_asset_for_repair_and_incre_cur_seq_num(&mut self)
                                        ->
        (&ReedSolomon,
         &mut SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
         &mut SmallVec<[bool; 32]>)
    {
        let cur_seq_num = self.cur_seq_num;

        let ret =
            if self.in_normal_block_set() {
                (&self.rs_codec_normal,
                 &mut self.buf_normal,
                 &mut self.buf_normal_slice_present)
            } else {
                (&self.rs_codec_last,
                 &mut self.buf_last,
                 &mut self.buf_last_slice_present)
            };

        self.add_cur_block_set_to_cur_seq_num();

        ret
    }

    fn add_cur_seq_num(&mut self,
                       val : u64) {
        let mut cur_seq_num_from_start  = self.cur_seq_num  - self.start_seq_num;
        let total_blocks_from_start = self.total_blocks - self.start_seq_num;

        cur_seq_num_from_start = cur_seq_num_from_start.wrapping_add(val);
        cur_seq_num_from_start = cur_seq_num_from_start % total_blocks_from_start;

        self.cur_seq_num = cur_seq_num_from_start + self.start_seq_num;
    }

    fn add_cur_block_set_to_cur_seq_num(&mut self) {
        let cur_block_set_size = if self.in_normal_block_set() {
            self.dat_num_normal + self.par_num_normal
        } else {
            self.dat_num_last   + self.par_num_last
        } as u64;

        self.add_cur_seq_num(cur_block_set_size);
    }

    pub fn get_buf(&self) -> &SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> {
        if self.in_normal_block_set() {
            &self.buf_normal
        } else {
            &self.buf_last
        }
    }

    pub fn get_buf_mut(&mut self) -> &mut SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> {
        if self.in_normal_block_set() {
            &mut self.buf_normal
        } else {
            &mut self.buf_last
        }
    }

    pub fn repair(&mut self,
                  data_only : bool) -> Result<(), Error> {
        let 
            if self.in_normal_block_set() {
                let rs_codec = match self.rs_codec_normal {
                    None        => { return Ok(()); },
                    Some(ref r) => r,
                };
                for i in 0..self.dat_num_normal + self.par_num_normal {
                    self.buf_normal_slice_present[i] =
                        self.ref_block.check_if_buffer_valid(&self.buf_normal[i]);
                }
                let mut buf : SmallVec<[&mut [u8]; 32]> = SmallVec::with_capacity(self. dat_num_normal + self.par_num_normal);

                for p in self.buf_normal.iter_mut() {
                    buf.push(sbx_block::slice_data_buf_mut(self.version, p));
                }

                let res = if data_only {
                    rs_codec.reconstruct_data(&mut buf,
                                              &self.buf_normal_slice_present)
                } else {
                    rs_codec.reconstruct(&mut buf,
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
                let rs_codec = match self.rs_codec_last {
                    None        => { return Ok(()); },
                    Some(ref r) => r
                };
                for i in 0..self.dat_num_last + self.par_num_last {
                    self.buf_last_slice_present[i] =
                        self.ref_block.check_if_buffer_valid(&self.buf_last[i]);
                }
                let mut buf : SmallVec<[&mut [u8]; 32]> = SmallVec::with_capacity(self. dat_num_last + self.par_num_last);
                for p in self.buf_last.iter_mut() {
                    buf.push(sbx_block::slice_data_buf_mut(self.version, p));
                }

                let res = if data_only {
                    rs_codec.reconstruct_data(&mut buf,
                                              &self.buf_last_slice_present)
                } else {
                    rs_codec.reconstruct(&mut buf,
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
        };

        res
    }

    pub fn verify(&mut self)
                  -> Result<bool, Error> {
        let res = if self.in_normal_block_set() {
            let rs_codec = match self.rs_codec_normal {
                None        => { return Ok(true); },
                Some(ref r) => r,
            };
            let slices : SmallVec<[&[u8]; 32]> =
                convert_2D_slices!(self.buf_normal =>to SmallVec<[&[u8]; 32]>,
                                   SmallVec::with_capacity);
            let mut buffer : SmallVec<[&mut [u8]; 32]> =
                convert_2D_slices!(self.buf_normal_par_verify =>to_mut SmallVec<[&mut [u8]; 32]>,
                SmallVec::with_capacity);
            match rs_codec.verify_with_buffer(&slices, &mut buffer) {
                Ok(v)  => Ok(v),
                Err(_) => Err(to_err(RSError::new(RSErrorKind::VerifyFail,
                                                  self.version,
                                                  self.cur_seq_num,
                                                  self.dat_num_normal + self.par_num_normal,
                                                  self.block_type,
                                                  &self.buf_normal_slice_present)))
            }
        } else {
            let rs_codec = match self.rs_codec_last {
                None        => { return Ok(true); },
                Some(ref r) => r
            };
            let slices : SmallVec<[&[u8]; 32]> =
                convert_2D_slices!(self.buf_last =>to SmallVec<[&[u8]; 32]>,
                                   SmallVec::with_capacity);
            let mut buffer : SmallVec<[&mut [u8]; 32]> =
                convert_2D_slices!(self.buf_last_par_verify =>to_mut SmallVec<[&mut [u8]; 32]>,
                                   SmallVec::with_capacity);
            match rs_codec.verify_with_buffer(&slices, &mut buffer) {
                Ok(v)  => Ok(v),
                Err(_) => Err(to_err(RSError::new(RSErrorKind::VerifyFail,
                                                  self.version,
                                                  self.cur_seq_num,
                                                  self.dat_num_last + self.par_num_last,
                                                  self.block_type,
                                                  &self.buf_last_slice_present)))
            }
        };

        self.add_cur_block_set_to_cur_seq_num();

        res
    }
}
