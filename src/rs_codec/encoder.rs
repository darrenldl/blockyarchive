use super::super::reed_solomon_erasure::ReedSolomon;
use super::super::smallvec::SmallVec;
use super::super::sbx_block;
use super::super::sbx_specs::Version;
use super::super::sbx_block::BlockType;
use super::super::sbx_specs::ver_to_block_size;
use super::super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;

use super::RSError;
use super::to_err;

pub struct RSEncoder {
    cur_data_index            : u64,
    last_data_set_size        : usize,
    last_data_set_start_index : u64,
    rs_codec                  : Option<ReedSolomon>,
    rs_codec_last             : Option<ReedSolomon>,
    data_shards               : usize,
    parity_shards             : usize,
    total_blocks              : u64,
    version                   : Version,
    parity_buf                : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    parity_buf_last           : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
}

impl RSEncoder {
    pub fn new(version       : Version,
               data_shards   : usize,
               parity_shards : usize,
               total_blocks  : u64) -> RSEncoder {
        let last_data_set_size         = last_data_set_size(data_shards,
                                                            total_shards);
        let last_data_set_start_index  = last_data_set_start_index(data_shards,
                                                                   total_shards);
        let last_data_set_parity_count = calc_parity_shards(data_shards,
                                                            parity_shards,
                                                            last_data_set_size);

        let mut parity_buf : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            SmallVec::with_capacity(parity_shards);
        for _ in 0..parity_shards {
            let mut v : SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]> = SmallVec::new();
            for _ in 0..SBX_LARGEST_BLOCK_SIZE {
                v.push(0);
            }
            parity_buf.push(v);
        }

        let mut parity_buf_last : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            SmallVec::with_capacity(last_data_set_parity_count);
        for _ in 0..last_data_set_parity_count {
            let mut v : SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]> = SmallVec::new();
            for _ in 0..SBX_LARGEST_BLOCK_SIZE {
                v.push(0);
            }
            parity_buf_last.push(v);
        }

        if total_shards == 0 {
            RSEncoder {
                cur_data_index            : 0,
                last_data_set_size,
                last_data_set_start_index,
                rs_codec                  : None,
                rs_codec_last             : None,
                data_shards,
                parity_shards,
                total_blocks,
                version,
                parity_buf,
                parity_buf_last,
            }
        } else {
            RSEncoder {
                cur_data_index : 0,
                last_data_set_size,
                last_data_set_start_index,
                rs_codec                  : Some(ReedSolomon::new(data_shards,
                                                       parity_shards).unwrap()),
                rs_codec_last             : Some(ReedSolomon::new(last_data_set_size,
                                                       last_data_set_parity_count).unwrap()),
                data_shards,
                parity_shards,
                total_blocks,
                version,
                parity_buf,
                parity_buf_last,
            }
        }
    }

    pub fn encode(&mut self,
                  data_shard    : &[u8])
                  -> Option<&mut SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>> {
        let mut ready = None;

        let rs_codec      = match self.rs_codec {
            None        => { return None; },
            Some(ref r) => r,
        };
        let rs_codec_last = match self.rs_codec_last {
            None        => { return None; },
            Some(ref r) => r
        };

        let data = sbx_block::slice_data_buf(self.version, data_shard);

        if self.cur_data_index < self.last_data_set_start_index as u64 {
            let index = (self.cur_data_index % self.data_shards as u64) as usize;
            {
                let mut parity : SmallVec<[&mut [u8]; 32]> = SmallVec::new();

                for p in &mut self.parity_buf[0..self.parity_shards].iter_mut() {
                    parity.push(sbx_block::slice_data_buf_mut(self.version, p));
                }
                rs_codec.encode_single_sep(index,
                                           data,
                                           &mut parity).unwrap();
            }
            if index == self.data_shards - 1 {
                ready = Some(&mut self.parity_buf);
            }
        } else {
            let index =
                (self.cur_data_index - self.last_data_set_start_index as u64) as usize;
            {
                let mut parity : SmallVec<[&mut [u8]; 32]> = SmallVec::new();

                for p in &mut self.parity_buf_last[0..rs_codec_last.parity_shard_count()].iter_mut() {
                    parity.push(sbx_block::slice_data_buf_mut(self.version, p));
                }
                rs_codec_last.encode_single_sep(index,
                                                data,
                                                &mut parity).unwrap();
            }
            if index == self.last_data_set_size - 1 {
                ready = Some(&mut self.parity_buf_last);
            }
        }

        self.cur_data_index = (self.cur_data_index + 1) % self.total_shards;

        ready
    }
}
