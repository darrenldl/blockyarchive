use crate::sbx_block;
use crate::sbx_specs::{ver_to_block_size, ver_uses_rs, Version, SBX_LARGEST_BLOCK_SIZE};
use reed_solomon_erasure::ReedSolomon;
use smallvec::SmallVec;

pub struct RSEncoder {
    index: usize,
    rs_codec: ReedSolomon,
    version: Version,
    par_buf: SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    active: bool,
}

macro_rules! mark_active {
    (
        $self:ident
    ) => {{
        $self.active = true;
    }};
}

macro_rules! mark_inactive {
    (
        $self:ident
    ) => {{
        $self.active = false;
    }};
}

macro_rules! incre_index {
    (
        $self:ident
    ) => {{
        $self.index += 1;
    }};
}

macro_rules! reset_index {
    (
        $self:ident
    ) => {{
        $self.index = 0;
    }};
}

macro_rules! codec_ready {
    (
        $self:ident
    ) => {{
        $self.index == $self.rs_codec.data_shard_count()
    }};
}

impl RSEncoder {
    pub fn new(version: Version, data_shards: usize, parity_shards: usize) -> RSEncoder {
        assert!(ver_uses_rs(version));

        let block_size = ver_to_block_size(version);

        let par_buf: SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; parity_shards];

        RSEncoder {
            index: 0,
            rs_codec: ReedSolomon::new(data_shards, parity_shards).unwrap(),
            version,
            par_buf,
            active: false,
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn unfilled_slot_count(&self) -> usize {
        self.total_slot_count() - self.index
    }

    pub fn total_slot_count(&self) -> usize {
        self.rs_codec.data_shard_count()
    }

    pub fn encode_no_block_sync(
        &mut self,
        data: &[u8],
    ) -> Option<&mut SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>> {
        let data = sbx_block::slice_data_buf(self.version, data);

        let version = self.version;
        let rs_codec = &self.rs_codec;
        let par_buf = &mut self.par_buf;

        {
            let mut parity: SmallVec<[&mut [u8]; 32]> = SmallVec::with_capacity(par_buf.len());

            for p in par_buf.iter_mut() {
                parity.push(sbx_block::slice_data_buf_mut(version, p));
            }
            rs_codec
                .encode_single_sep(self.index, data, &mut parity)
                .unwrap();
        }

        incre_index!(self);

        if codec_ready!(self) {
            reset_index!(self);
            mark_inactive!(self);
            Some(par_buf)
        } else {
            mark_active!(self);
            None
        }
    }
}
