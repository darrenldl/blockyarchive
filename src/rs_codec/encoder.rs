use super::super::reed_solomon_erasure::ReedSolomon;
use super::super::smallvec::SmallVec;
use super::super::sbx_block;
use super::super::sbx_specs::Version;
use super::super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::super::sbx_specs::ver_to_block_size;
use super::*;

pub struct RSEncoder {
    active                    : bool,
    cur_data_index            : u32,
    last_data_set_start_index : u32,
    rs_codec_normal           : ReedSolomon,
    rs_codec_last             : ReedSolomon,
    dat_num_normal            : usize,
    par_num_normal            : usize,
    dat_num_last              : usize,
    par_num_last              : usize,
    total_data_chunks         : u32,
    version                   : Version,
    par_buf_normal            : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    par_buf_last              : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
}

macro_rules! add_cur_data_index {
    (
        $self:ident, $val:expr
    ) => {{
        $self.cur_data_index = $self.cur_data_index.wrapping_add($val) % $self.total_data_chunks;
    }};
    (
        1 => $self:ident
    ) => {{
        add_cur_data_index!($self, 1);
    }}
}

macro_rules! pick_asset {
    (
        encode => $self:ident
    ) => {{
        if $self.in_normal_data_set() {
            (&$self.rs_codec_normal,
             &mut $self.par_buf_normal)
        } else {
            (&$self.rs_codec_last,
             &mut $self.par_buf_last)
        }
    }}
}

impl RSEncoder {
    pub fn new(version           : Version,
               data_shards       : usize,
               parity_shards     : usize,
               total_data_chunks : u32) -> RSEncoder {
        let last_data_set_size         =
            from_data_block_count::last_data_set_size(data_shards,
                                                      total_data_chunks);
        let last_data_set_start_index  =
            from_data_block_count::last_data_set_start_index(data_shards,
                                                             total_data_chunks);
        let last_data_set_parity_count = calc_parity_shards(data_shards,
                                                            parity_shards,
                                                            last_data_set_size);
        let block_size = ver_to_block_size(version);

        let par_buf_normal : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; parity_shards];

        let par_buf_last : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; last_data_set_parity_count];

        RSEncoder {
            active : total_data_chunks != 0,
            cur_data_index            : 0,
            last_data_set_start_index,
            rs_codec_normal : ReedSolomon::new(data_shards,
                                               parity_shards).unwrap(),
            rs_codec_last   : ReedSolomon::new(last_data_set_size,
                                               last_data_set_parity_count).unwrap(),
            dat_num_normal : data_shards,
            par_num_normal : parity_shards,
            dat_num_last   : last_data_set_size,
            par_num_last   : last_data_set_parity_count,
            total_data_chunks,
            version,
            par_buf_normal,
            par_buf_last,
        }
    }

    fn in_normal_data_set(&self) -> bool {
        self.cur_data_index < self.last_data_set_start_index as u32
    }

    pub fn encode(&mut self,
                  data : &[u8])
                  -> Option<&mut SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>> {
        if !self.active { return None; }

        let data = sbx_block::slice_data_buf(self.version, data);

        let version = self.version;

        let (rs_codec, par_buf) = pick_asset!(encode => self);

        let data_shards = rs_codec.data_shard_count();

        let index = (self.cur_data_index
                     % rs_codec.data_shard_count() as u32) as usize;

        {
            let mut parity : SmallVec<[&mut [u8]; 32]> = SmallVec::with_capacity(par_buf.len());

            for p in par_buf.iter_mut() {
                parity.push(sbx_block::slice_data_buf_mut(version, p));
            }
            rs_codec.encode_single_sep(index,
                                       data,
                                       &mut parity).unwrap();
        }

        add_cur_data_index!(1 => self);

        if index == data_shards - 1 {
            Some(par_buf)
        } else {
            None
        }
    }
}
