use super::reed_solomon_erasure::ReedSolomon;
use super::smallvec::SmallVec;
use super::sbx_block;
use super::sbx_specs::Version;

pub struct RSCodec {
    cur_data_index            : usize,
    last_data_set_size        : usize,
    last_data_set_start_index : usize,
    rs_codec                  : Option<ReedSolomon>,
    rs_codec_last             : Option<ReedSolomon>,
    data_shards               : usize,
    parity_shards             : usize,
    total_shards              : usize,
    version                   : Version,
}

impl RSCodec {
    pub fn new(version       : Version,
               data_shards   : usize,
               parity_shards : usize,
               total_shards  : usize) -> RSCodec {
        let last_data_set_size         = last_data_set_size(data_shards,
                                                            total_shards);
        let last_data_set_start_index  = last_data_set_start_index(data_shards,
                                                                   total_shards);
        let last_data_set_parity_count = calc_parity_shards(data_shards,
                                                            parity_shards,
                                                            last_data_set_size);

        if total_shards == 0 {
            RSCodec {
                cur_data_index            : 0,
                last_data_set_size,
                last_data_set_start_index,
                rs_codec                  : None,
                rs_codec_last             : None,
                data_shards,
                parity_shards,
                total_shards,
                version,
            }
        } else {
            RSCodec {
                cur_data_index : 0,
                last_data_set_size,
                last_data_set_start_index,
                rs_codec                  : Some(ReedSolomon::new(data_shards,
                                                       parity_shards).unwrap()),
                rs_codec_last             : Some(ReedSolomon::new(last_data_set_size,
                                                       last_data_set_parity_count).unwrap()),
                data_shards,
                parity_shards,
                total_shards,
                version,
            }
        }
    }

    pub fn encode(&mut self,
                  data_shard    : &[u8],
                  parity_shards : &mut [&mut [u8]])
                  -> Option<usize> {
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

        let mut parity : SmallVec<[&mut [u8]; 32]> = SmallVec::new();

        if self.cur_data_index < self.last_data_set_start_index {
            let index = self.cur_data_index % self.data_shards;
            for p in &mut parity_shards[0..self.parity_shards].iter_mut() {
                parity.push(sbx_block::slice_data_buf_mut(self.version, p));
            }
            rs_codec.encode_single_sep(index,
                                       data,
                                       &mut parity).unwrap();
            if index == self.data_shards - 1 {
                ready = Some(rs_codec.parity_shard_count());
            }
        } else {
            let index = self.cur_data_index - self.last_data_set_start_index;
            for p in &mut parity_shards[0..rs_codec_last.parity_shard_count()].iter_mut() {
                parity.push(sbx_block::slice_data_buf_mut(self.version, p));
            }
            rs_codec_last.encode_single_sep(index,
                                            data,
                                            &mut parity).unwrap();
            if index == self.last_data_set_size - 1 {
                ready = Some(rs_codec_last.parity_shard_count());
            }
        }

        self.cur_data_index = (self.cur_data_index + 1) % self.total_shards;

        ready
    }
}

fn last_data_set_start_index(data_shards   : usize,
                        total_shards  : usize) -> usize {
    total_shards - last_data_set_size(data_shards, total_shards)
}

fn last_data_set_size(data_shards   : usize,
                 total_shards  : usize) -> usize {
    let size = total_shards % data_shards;
    if size == 0 {
        data_shards
    } else {
        size
    }
}

fn calc_parity_shards(data_shards   : usize,
                      parity_shards : usize,
                      set_size      : usize) -> usize {
    (set_size * parity_shards + (data_shards - 1)) / data_shards
}
