use super::super::reed_solomon_erasure::ReedSolomon;
use super::super::smallvec::SmallVec;
use super::super::sbx_block;
use super::super::sbx_specs::Version;
use super::super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::super::sbx_specs::ver_to_block_size;
use super::*;

pub struct RSEncoder {
    cur_data_index            : u64,
    last_data_set_start_index : u64,
    rs_codec_normal           : Option<ReedSolomon>,
    rs_codec_last             : Option<ReedSolomon>,
    dat_num_normal            : usize,
    par_num_normal            : usize,
    dat_num_last              : usize,
    par_num_last              : usize,
    total_data_chunks         : u64,
    version                   : Version,
    par_buf_normal            : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    par_buf_last              : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
}

impl RSEncoder {
    pub fn new(version           : Version,
               data_shards       : usize,
               parity_shards     : usize,
               total_data_chunks : u64) -> RSEncoder {
        let last_data_set_size         = last_data_set_size(data_shards,
                                                            total_data_chunks);
        let last_data_set_start_index  = last_data_set_start_index(data_shards,
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
            cur_data_index            : 0,
            last_data_set_start_index,
            rs_codec_normal :
            if total_data_chunks == 0 { None }
            else { Some(ReedSolomon::new(data_shards, parity_shards).unwrap()) },
            rs_codec_last   :
            if total_data_chunks == 0 { None }
            else { Some(ReedSolomon::new(last_data_set_size,
                                      last_data_set_parity_count).unwrap()) },
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

    pub fn encode(&mut self,
                  data : &[u8])
                  -> Option<&mut SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>> {
        let mut ready = None;

        let rs_codec_normal = match self.rs_codec_normal {
            None        => { return None; },
            Some(ref r) => r,
        };
        let rs_codec_last   = match self.rs_codec_last {
            None        => { return None; },
            Some(ref r) => r
        };

        let data = sbx_block::slice_data_buf(self.version, data);

        if self.cur_data_index < self.last_data_set_start_index as u64 {
            let index = (self.cur_data_index % self.dat_num_normal as u64) as usize;
            {
                let mut parity : SmallVec<[&mut [u8]; 32]> = SmallVec::new();

                for p in &mut self.par_buf_normal[0..self.par_num_normal] {
                    parity.push(sbx_block::slice_data_buf_mut(self.version, p));
                }
                rs_codec_normal.encode_single_sep(index,
                                           data,
                                           &mut parity).unwrap();
            }
            if index == self.dat_num_normal - 1 {
                ready = Some(&mut self.par_buf_normal);
            }
        } else {
            let index =
                (self.cur_data_index - self.last_data_set_start_index as u64) as usize;
            {
                let mut parity : SmallVec<[&mut [u8]; 32]> = SmallVec::new();

                for p in &mut self.par_buf_last[0..self.par_num_last] {
                    parity.push(sbx_block::slice_data_buf_mut(self.version, p));
                }
                rs_codec_last.encode_single_sep(index,
                                                data,
                                                &mut parity).unwrap();
            }
            if index == self.dat_num_last - 1 {
                ready = Some(&mut self.par_buf_last);
            }
        }

        self.cur_data_index = (self.cur_data_index + 1) % self.total_data_chunks;

        ready
    }
}
