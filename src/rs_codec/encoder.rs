use reed_solomon_erasure::ReedSolomon;
use smallvec::SmallVec;
use sbx_block;
use sbx_specs::{Version,
                SBX_LARGEST_BLOCK_SIZE,
                ver_to_block_size};

pub struct RSEncoder {
    index    : usize,
    rs_codec : ReedSolomon,
    version  : Version,
    par_buf  : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
}

macro_rules! add_index {
    (
        $self:ident, $val:expr
    ) => {{
        $self.index =
            ($self.index + $val) % ($self.rs_codec.data_shard_count() + 1);
    }};
    (
        1 => $self:ident
    ) => {{
        add_index!($self, 1);
    }}
}

macro_rules! codec_ready {
    (
        $self:ident
    ) => {{
        $self.index == $self.rs_codec.data_shard_count()
    }}
}

impl RSEncoder {
    pub fn new(version       : Version,
               data_shards   : usize,
               parity_shards : usize) -> RSEncoder {
        let block_size = ver_to_block_size(version);

        let par_buf : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; parity_shards];

        RSEncoder {
            index : 0,
            rs_codec   : ReedSolomon::new(data_shards,
                                          parity_shards).unwrap(),
            version,
            par_buf,
        }
    }

    pub fn unfilled_slot_count(&self) -> usize {
        self.rs_codec.data_shard_count() - self.index
    }

    pub fn encode_no_block_sync(&mut self,
                                data : &[u8])
                                -> Option<&mut SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>> {
        if codec_ready!(self) {
            // reset index by wrapping around
            add_index!(1 => self);
        }

        let data = sbx_block::slice_data_buf(self.version, data);

        let version  = self.version;
        let rs_codec = &self.rs_codec;
        let par_buf  = &mut self.par_buf;

        {
            let mut parity : SmallVec<[&mut [u8]; 32]> =
                SmallVec::with_capacity(par_buf.len());

            for p in par_buf.iter_mut() {
                parity.push(sbx_block::slice_data_buf_mut(version, p));
            }
            rs_codec.encode_single_sep(self.index,
                                       data,
                                       &mut parity).unwrap();
        }

        add_index!(1 => self);

        if codec_ready!(self) {
            Some(par_buf)
        } else {
            None
        }
    }
}
