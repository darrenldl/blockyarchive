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

macro_rules! in_normal_block_set {
    (
        $self:ident
    ) => {{
        $self.cur_seq_num < $self.last_block_set_start_seq_num
    }}
}

macro_rules! add_cur_seq_num {
    (
        $self:ident, $val:expr
    ) => {{
        let mut cur_seq_num_from_start = $self.cur_seq_num  - $self.start_seq_num;
        let total_blocks_from_start    = $self.total_blocks - $self.start_seq_num;

        cur_seq_num_from_start = cur_seq_num_from_start.wrapping_add($val);
        cur_seq_num_from_start = cur_seq_num_from_start % total_blocks_from_start;

        $self.cur_seq_num = cur_seq_num_from_start + $self.start_seq_num;
    }};
    (
        cur_block_set => $self:ident
    ) => {{
        let cur_block_set_size =
            if in_normal_block_set!($self) {
                $self.rs_codec_normal.total_shard_count()
            } else {
                $self.rs_codec_last.total_shard_count()
            } as u64;

        add_cur_seq_num!($self, cur_block_set_size);
    }}
}

macro_rules! pick_asset {
    (
        repair => $self:ident
    ) => {{
        if in_normal_block_set!($self) {
            (&$self.rs_codec_normal,
             &mut $self.buf_normal,
             &mut $self.buf_normal_slice_present)
        } else {
            (&$self.rs_codec_last,
             &mut $self.buf_last,
             &mut $self.buf_last_slice_present)
        }
    }};
    (
        verify => $self:ident
    ) => {{
        if in_normal_block_set!($self) {
            (&$self.rs_codec_normal,
             &$self.buf_normal,
             &mut $self.buf_normal_par_verify)
        } else {
            (&$self.rs_codec_last,
             &$self.buf_last,
             &mut $self.buf_last_par_verify)
        }
    }};
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
            active                 : total_data_chunks != 0,
            cur_seq_num            : start_seq_num,
            start_seq_num,
            last_block_set_start_seq_num,
            rs_codec_normal        : ReedSolomon::new(data_shards,
                                                      parity_shards).unwrap(),
            rs_codec_last          : ReedSolomon::new(last_data_set_size,
                                                      last_data_set_parity_count).unwrap(),
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

    pub fn get_buf(&self) -> &SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> {
        if in_normal_block_set!(self) {
            &self.buf_normal
        } else {
            &self.buf_last
        }
    }

    pub fn get_buf_mut(&mut self) -> &mut SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> {
        if in_normal_block_set!(self) {
            &mut self.buf_normal
        } else {
            &mut self.buf_last
        }
    }

    pub fn repair(&mut self,
                  data_only : bool) -> Result<(), Error> {
        let (rs_codec, slice_buf, slice_present) = pick_asset!(repair => self);

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
                                                  total_num,
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
        let (rs_codec, slice_buf, par_buf) = pick_asset!(verify => self);

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
                                                  total_num,
                                                  self.block_type,
                                                  None)))
            };

        if incre_cur_seq_num {
            add_cur_seq_num!(cur_block_set => self);
        }

        res
    }
}
