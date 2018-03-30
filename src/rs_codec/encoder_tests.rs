#![cfg(test)]
use super::encoder::*;
use sbx_block;
use sbx_specs::{Version,
                SBX_LARGEST_BLOCK_SIZE};
use reed_solomon_erasure::ReedSolomon;

use rand_utils::fill_random_bytes;

macro_rules! make_random_block_buffers {
    ($per_shard:expr, $size:expr) => {{
        let mut buffer = Vec::with_capacity(20);
        for _ in 0..$size {
            buffer.push(vec![0; $per_shard]);
        }

        for s in buffer.iter_mut() {
            fill_random_bytes(s);
        }

        buffer
    }}
}

#[test]
fn test_encoder_encode_correctly_simple_cases() {
    {
        let version = Version::V17;
        let r = ReedSolomon::new(10, 3).unwrap();
        let mut encoder = RSEncoder::new(version, 10, 3);

        let mut buffer = make_random_block_buffers!(10_000, 13);
        let mut buffer_copy = buffer.clone();

        assert!(!encoder.active());
        assert_eq!(10, encoder.unfilled_slot_count());
        assert_eq!(10, encoder.total_slot_count());

        {
            let mut refs = Vec::new();
            for b in buffer.iter_mut() {
                refs.push(sbx_block::slice_data_buf_mut(version, b));
            }

            r.encode(&mut refs).unwrap();
        }

        for _ in 0..2 {
            {
                let mut refs = Vec::new();
                for b in buffer_copy.iter_mut() {
                    refs.push(b);
                }

                for i in 0..9 {
                    assert_eq!(10 - i, encoder.unfilled_slot_count());
                    assert_eq!(None, encoder.encode_no_block_sync(refs[i]));
                    assert!(encoder.active());
                }

                assert_eq!(1, encoder.unfilled_slot_count());

                let mut i = 0;
                for b in encoder.encode_no_block_sync(refs[9]).unwrap().iter() {
                    assert_eq!(sbx_block::slice_data_buf(version, b),
                               sbx_block::slice_data_buf(version, &buffer[i + 10]));
                    i += 1;
                }
            }

            assert!(!encoder.active());
            assert_eq!(10, encoder.unfilled_slot_count());
            assert_eq!(10, encoder.total_slot_count());
        }
    }
}

quickcheck! {
    fn qc_encoder_encode_same_as_encode(data   : usize,
                                        parity : usize,
                                        reuse  : usize) -> bool {
        let data   = 1 + data % 10;
        let parity = 1 + parity % 10;

        let reuse = reuse % 10;

        let r = ReedSolomon::new(data, parity).unwrap();

        let mut buffer = make_random_block_buffers!(SBX_LARGEST_BLOCK_SIZE, data + parity);
        let mut buffer_copy = buffer.clone();

        let versions = vec![Version::V17, Version::V18, Version::V19];

        for version in versions.into_iter () {
            let mut encoder = RSEncoder::new(version, data, parity);

            {
                let mut refs = Vec::new();
                for b in buffer.iter_mut() {
                    refs.push(sbx_block::slice_data_buf_mut(version, b));
                }

                r.encode(&mut refs).unwrap();
            }

            for _ in 0..1 + reuse {
                {
                    let mut refs = Vec::new();
                    for b in buffer_copy.iter_mut() {
                        refs.push(b);
                    }

                    for i in 0..data - 1 {
                        if encoder.unfilled_slot_count() != data - i {
                            return false;
                        }

                        if !(encoder.encode_no_block_sync(refs[i]) == None
                             && encoder.active())
                        {
                            return false;
                        }
                    }

                    if encoder.unfilled_slot_count() != 1 {
                        return false;
                    }

                    let mut i = 0;
                    for b in encoder.encode_no_block_sync(refs[data - 1]).unwrap().iter() {
                        if sbx_block::slice_data_buf(version, b)
                            != sbx_block::slice_data_buf(version, &buffer[i + data])
                        {
                            return false;
                        }
                        i += 1;
                    }
                }

                if !(!encoder.active()
                     && encoder.unfilled_slot_count() == data
                     && encoder.total_slot_count() == data)
                {
                    return false;
                }
            }
        }

        return true;
    }
}
