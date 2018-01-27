use super::reed_solomon_erasure::ReedSolomon;

pub struct RSCodec {
    cur_data_index       : usize,
    last_set_start_index : usize,
    rs_codec             : Option<ReedSolomon>,
    rs_codec_last        : Option<ReedSolomon>,
    data_shards          : usize,
    parity_shards        : usize,
}

impl RSCodec {
    pub fn new(data_shards   : usize,
               parity_shards : usize,
               total_shards  : usize) -> RSCodec {
        let last_set_size         = last_set_size(data_shards,
                                                  total_shards);
        let last_set_start_index  = last_set_start_index(data_shards,
                                                         total_shards);
        let last_set_parity_count = calc_parity_shards(data_shards,
                                                       parity_shards,
                                                       last_set_size);

        if total_shards == 0 {
            RSCodec {
                cur_data_index       : 0,
                last_set_start_index,
                rs_codec             : None,
                rs_codec_last        : None,
                data_shards,
                parity_shards,
            }
        } else {
            RSCodec {
                cur_data_index : 0,
                last_set_start_index,
                rs_codec       : Some(ReedSolomon::new(data_shards,
                                                       parity_shards).unwrap()),
                rs_codec_last  : Some(ReedSolomon::new(last_set_size,
                                                       last_set_parity_count).unwrap()),
                data_shards,
                parity_shards,
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

        if self.cur_data_index < self.last_set_start_index {
            let index = self.cur_data_index % self.data_shards;
            rs_codec.encode_single_sep(index,
                                       data_shard,
                                       parity_shards).unwrap();
            if index == self.data_shards - 1 {
                ready = Some(rs_codec.parity_shard_count());
            }
        } else {
            let index = self.cur_data_index - self.last_set_start_index;
            let parity = &mut parity_shards[0..rs_codec_last.parity_shard_count()];
            rs_codec_last.encode_single_sep(index,
                                            data_shard,
                                            parity).unwrap();
            if index == self.last_set_start_index - 1 {
                ready = Some(rs_codec_last.parity_shard_count());
            }
        }

        self.cur_data_index += 1;

        ready
    }
}

fn last_set_start_index(data_shards   : usize,
                        total_shards  : usize) -> usize {
    total_shards - last_set_size(data_shards, total_shards)
}

fn last_set_size(data_shards   : usize,
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
