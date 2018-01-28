use super::super::reed_solomon_erasure::ReedSolomon;
use super::super::smallvec::SmallVec;
use super::super::sbx_block;
use super::super::sbx_specs::Version;
use super::super::sbx_block::BlockType;
use super::super::sbx_block::Block;
use super::super::sbx_specs::ver_to_block_size;
use super::super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;

use super::Error;

pub struct RSRepairer {
    data_type                 : BlockType,
    cur_index                 : u64,
    last_data_set_size        : usize,
    last_set_start_seq_num    : u64,
    rs_codec                  : Option<ReedSolomon>,
    rs_codec_last             : Option<ReedSolomon>,
    data_shards               : usize,
    parity_shards             : usize,
    total_blocks              : u64,
    version                   : Version,
    buf                       : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_slice_present         : SmallVec<[bool; 32]>,
    buf_last                  : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_last_slice_present    : SmallVec<[bool; 32]>,
    block_type                : BlockType,
    ref_block                 : Block,
}

impl RSRepairer {
    pub fn new(version       : Version,
               ref_block     : &Block,
               data_shards   : usize,
               parity_shards : usize,
               total_blocks  : u64) -> RSRepairer {
        let last_data_set_size         = last_data_set_size(data_shards,
                                                            total_shards);
        let last_set_start_seq_num     = last_set_start_seq_num(data_shards,
                                                                parity_shards,
                                                                total_shards);
        let last_data_set_start_index  = last_data_set_start_index(data_shards,
                                                                   total_shards);
        let last_data_set_parity_count = calc_parity_shards(data_shards,
                                                            parity_shards,
                                                            last_data_set_size);

        let mut buf : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            SmallVec::with_capacity(data_shards + parity_shards);
        for _ in 0..data_shards + parity_shards {
            let mut v : SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]> = SmallVec::new();
            for _ in 0..SBX_LARGEST_BLOCK_SIZE {
                v.push(0);
            }
            buf.push(v);
        }

        let mut buf_last : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            SmallVec::with_capacity(last_data_set_size + last_data_set_parity_count);
        for _ in 0..last_data_set_size + last_data_set_parity_count {
            let mut v : SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]> = SmallVec::new();
            for _ in 0..SBX_LARGEST_BLOCK_SIZE {
                v.push(0);
            }
            buf_last.push(v);
        }

        if total_shards == 0 {
            RSRepairer {
                cur_index              : 0,
                last_data_set_size,
                last_set_start_seq_num,
                rs_codec               : None,
                rs_codec_last          : None,
                data_shards,
                parity_shards,
                total_shards,
                version,
                buf,
                buf_last,
                ref_block              : ref_block.clone(),
            }
        } else {
            RSRepairer {
                cur_index              : 0,
                last_data_set_size,
                last_set_start_seq_num,
                rs_codec               : Some(ReedSolomon::new(data_shards,
                                                                  parity_shards).unwrap()),
                rs_codec_last          : Some(ReedSolomon::new(last_data_set_size,
                                                                  last_data_set_parity_count).unwrap()),
                data_shards,
                parity_shards,
                total_shards,
                version,
                buf,
                buf_last,
                ref_block              : ref_block.clone(),
            }
        }
    }

    pub fn repair(&mut self) -> &SmallVec<[bool; 32]> {
        if self.cur_index < self.last_set_start_seq_num {
            for i in 0..self.data_shards + self.parity_shards {
                self.buf_slice_present[i] =
                    self.ref_block.check_if_buffer_contains_valid_block(&buf[i],
                                                                        self.block_type);
            }
            &self.buf_last_slice_present
        } else {
            let last_set_parity_count = calc_parity_shards(self.data_shards,
                                                           self.parity_shards,
                                                           self.last_data_set_size);
            for i in 0..self.last_data_set_size + last_set_parity_count {
                self.buf_last_slice_present[i] =
                    self.ref_block.check_if_buffer_contains_valid_block(&buf_last[i],
                                                                        self.block_type);
            }
            &self.buf_last_slice_present
        }
    }
}
