use super::super::reed_solomon_erasure::ReedSolomon;
use super::super::smallvec::SmallVec;
use super::super::sbx_specs::Version;
use super::super::sbx_block::BlockType;
use super::super::sbx_block::Block;
use super::super::sbx_specs::ver_to_block_size;
use super::super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::super::sbx_specs::SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM;
use super::*;
use super::super::sbx_block;

use super::Error;

pub struct RSRepairer {
    cur_seq_num    : u32,
    start_seq_num  : u32,
    rs_codec       : ReedSolomon,
    version        : Version,
    buf            : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_par_verify : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_present    : SmallVec<[bool; 32]>,
    block_type     : BlockType,
    ref_block      : Block,
}

macro_rules! add_cur_seq_num {
    (
        $self:ident, $val:expr
    ) => {{
        let mut cur_seq_num_from_start = $self.cur_seq_num  - $self.start_seq_num;

        cur_seq_num_from_start = cur_seq_num_from_start.wrapping_add($val);

        $self.cur_seq_num = cur_seq_num_from_start + $self.start_seq_num;
    }};
    (
        cur_block_set => $self:ident
    ) => {{
        let cur_block_set_size = $self.rs_codec.total_shard_count() as u32;

        add_cur_seq_num!($self, cur_block_set_size);
    }}
}

impl RSRepairer {
    pub fn new(version           : Version,
               ref_block         : &Block,
               block_type        : BlockType,
               data_shards       : usize,
               parity_shards     : usize) -> RSRepairer {
        let block_size = ver_to_block_size(version);

        let buf : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; data_shards + parity_shards];
        let buf_present : SmallVec<[bool; 32]> =
            smallvec![false; data_shards + parity_shards];

        let buf_par_verify : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; parity_shards];

        let start_seq_num = match block_type {
            BlockType::Meta => 0,
            BlockType::Data => SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM,
        } as u32;

        RSRepairer {
            cur_seq_num       : start_seq_num,
            start_seq_num,
            rs_codec          : ReedSolomon::new(data_shards,
                                                 parity_shards).unwrap(),
            version,
            buf,
            buf_par_verify,
            buf_present,
            block_type,
            ref_block         : ref_block.clone(),
        }
    }

    pub fn get_buf(&self) -> &SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> {
        &self.buf
    }

    pub fn get_buf_mut(&mut self) -> &mut SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> {
        &mut self.buf
    }

    pub fn repair(&mut self,
                  data_only : bool) -> Result<(), Error> {
        let rs_codec      = &self.rs_codec;
        let slice_buf     = &mut self.buf;
        let slice_present = &mut self.buf_present;

        let total_num = rs_codec.total_shard_count();

        for i in 0..total_num {
            slice_present[i] =
                sbx_block::check_if_buffer_valid(&slice_buf[i]);
        }

        let mut buf : SmallVec<[&mut [u8]; 32]> =
            SmallVec::with_capacity(total_num);
        for s in slice_buf.iter_mut() {
            buf.push(sbx_block::slice_data_buf_mut(self.version, s));
        }

        let res = {
            match
                if data_only { rs_codec.reconstruct_data(&mut buf,
                                                         slice_present)
                } else       { rs_codec.reconstruct(&mut buf,
                                                    slice_present) }
            {
                Ok(()) => Ok(()),
                Err(_) => Err(to_err(RSError::new(RSErrorKind::RepairFail,
                                                  self.version,
                                                  self.cur_seq_num,
                                                  total_num as u32,
                                                  self.block_type,
                                                  Some(slice_present))))
            }
        };

        add_cur_seq_num!(cur_block_set => self);

        res
    }

    pub fn verify(&mut self,
                  incre_cur_seq_num : bool)
                  -> Result<bool, Error> {
        let rs_codec      = &self.rs_codec;
        let slice_buf     = &self.buf;
        let slice_present = &self.buf_present;
        let par_buf       = &mut self.buf_par_verify;

        let par_num   = rs_codec.parity_shard_count();
        let total_num = rs_codec.total_shard_count();

        let mut slices : SmallVec<[&[u8]; 32]> =
            SmallVec::with_capacity(total_num);
        for s in slice_buf.iter() {
            slices.push(sbx_block::slice_data_buf(self.version, s));
        }
        let mut par : SmallVec<[&mut [u8]; 32]> =
            SmallVec::with_capacity(par_num);
        for p in par_buf.iter_mut() {
            par.push(sbx_block::slice_data_buf_mut(self.version, p));
        }

        let res =
            match rs_codec.verify_with_buffer(&slices, &mut par) {
                Ok(v)  => Ok(v),
                Err(_) => Err(to_err(RSError::new(RSErrorKind::VerifyFail,
                                                  self.version,
                                                  self.cur_seq_num,
                                                  total_num as u32,
                                                  self.block_type,
                                                  None)))
            };

        if incre_cur_seq_num {
            add_cur_seq_num!(cur_block_set => self);
        }

        res
    }
}
