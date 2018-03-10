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
    block_type     : BlockType,
    ref_block      : Block,
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

        RSRepairer {
            index          : 0,
            rs_codec       : ReedSolomon::new(data_shards,
                                              parity_shards).unwrap(),
            version,
            buf,
            buf_par_verify,
            buf_present,
            block_type,
            ref_block      : ref_block.clone(),
        }
    }

    pub fn get_shard_buffer(&mut self) -> &mut [u8] {
        let index = self.index;

        add_index!(1 => self);

        self.buf_present[index] = true;

        sbx_block::slice_data_buf_mut(self.version, &mut self.buf[index])
    }

    pub fn mark_missing(&mut self) {
        self.buf_present[self.index] = false;

        add_index!(1 => self);
    }

    pub fn repair(&mut self) -> bool {
        assert_eq!(0, self.index);

        let rs_codec      = &self.rs_codec;

        let mut buf : SmallVec<[&mut [u8]; 32]> =
            SmallVec::with_capacity(rs_codec.total_shard_count());
        for s in self.buf.iter_mut() {
            buf.push(sbx_block::slice_data_buf_mut(self.version, s));
        }

        match rs_codec.reconstruct(&mut buf, &self.buf_present) {
            Ok(()) => true,
            Err(_) => false
        }
    }

    pub fn verify(&mut self) -> bool {
        assert_eq!(0, self.index);

        let rs_codec      = &self.rs_codec;

        let mut slices : SmallVec<[&[u8]; 32]> =
            SmallVec::with_capacity(rs_codec.total_shard_count());
        for s in self.buf.iter() {
            slices.push(sbx_block::slice_data_buf(self.version, s));
        }
        let mut par : SmallVec<[&mut [u8]; 32]> =
            SmallVec::with_capacity(rs_codec.parity_shard_count());
        for p in self.buf_par_verify.iter_mut() {
            par.push(sbx_block::slice_data_buf_mut(self.version, p));
        }

        rs_codec.verify_with_buffer(&slices, &mut par).unwrap()
    }
}
