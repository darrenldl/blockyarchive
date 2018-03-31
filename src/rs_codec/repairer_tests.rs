#![cfg(test)]
use super::*;
use super::repairer::*;
use sbx_block;
use sbx_block::Block;
use sbx_block::BlockType;
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
fn test_repairer_repair_properly_simple_cases() {
    let version = Version::V17;
    let r = ReedSolomon::new(10, 3).unwrap();

    let ref_block = Block::new(version, &[0; 6], BlockType::Data);
    let mut repairer = RSRepairer::new(&ref_block, 10, 3, 0);

    let mut buffer = make_random_block_buffers!(4096, 13);

    assert!(!repairer.active());
    assert_eq!(13, repairer.unfilled_slot_count());
    assert_eq!(13, repairer.total_slot_count());

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
            for b in buffer.iter_mut() {
                refs.push(sbx_block::slice_buf(version, b));
            }

            // mark 0, 5, 11 as missing
            let mut i = 0;
            for b in refs.iter() {
                assert_eq!(13 - i, repairer.unfilled_slot_count());
                assert_eq!(13, repairer.total_slot_count());

                repairer.get_block_buffer().copy_from_slice(b);
                let codec_state =
                    if i == 0 || i == 5 || i == 11 {
                        repairer.mark_missing()
                    } else {
                        repairer.mark_present()
                    };

                assert_eq!(13 - i - 1, repairer.unfilled_slot_count());
                assert_eq!(13, repairer.total_slot_count());

                if i == 12 {
                    assert_eq!(RSCodecState::Ready, codec_state);
                } else {
                    assert_eq!(RSCodecState::NotReady, codec_state);
                }

                i += 1;
            }
        }

        let (stats, blocks) = repairer.repair_with_block_sync(1);

        assert_eq!(Version::V17, stats.version);
        assert_eq!((10, 3, 0), stats.data_par_burst);
        assert!(stats.successful);
        assert_eq!(1, stats.start_seq_num);
        assert_eq!(false, stats.present[0]);
        assert_eq!(true,  stats.present[1]);
        assert_eq!(true,  stats.present[2]);
        assert_eq!(true,  stats.present[3]);
        assert_eq!(true,  stats.present[4]);
        assert_eq!(false, stats.present[5]);
        assert_eq!(true,  stats.present[6]);
        assert_eq!(true,  stats.present[7]);
        assert_eq!(true,  stats.present[8]);
        assert_eq!(true,  stats.present[9]);
        assert_eq!(true,  stats.present[10]);
        assert_eq!(false, stats.present[11]);
        assert_eq!(true,  stats.present[12]);

        assert_eq!(13, stats.present.len());
        assert_eq!(3, stats.missing_count);
        assert_eq!(10, stats.present_count);

        assert_eq!(3, blocks.len());

        {
            let pos = sbx_block::calc_data_block_write_pos(Version::V17,
                                                           1 + 0,
                                                           None,
                                                           Some((10, 3, 0)));
            assert_eq!(pos, blocks[0].0);
            let mut block = Block::dummy();

            block.sync_from_buffer(blocks[0].1, None).unwrap();
            assert_eq!(1 + 0, block.get_seq_num());
            assert_eq!([0; 6], block.get_uid());
            assert_eq!(Version::V17, block.get_version());
        }
        {
            let pos = sbx_block::calc_data_block_write_pos(Version::V17,
                                                           1 + 5,
                                                           None,
                                                           Some((10, 3, 0)));
            assert_eq!(pos, blocks[1].0);
            let mut block = Block::dummy();

            block.sync_from_buffer(blocks[1].1, None).unwrap();
            assert_eq!(1 + 5, block.get_seq_num());
            assert_eq!([0; 6], block.get_uid());
            assert_eq!(Version::V17, block.get_version());
        }
        {
            let pos = sbx_block::calc_data_block_write_pos(Version::V17,
                                                           1 + 11,
                                                           None,
                                                           Some((10, 3, 0)));
            assert_eq!(pos, blocks[2].0);
            let mut block = Block::dummy();

            block.sync_from_buffer(blocks[2].1, None).unwrap();
            assert_eq!(1 + 11, block.get_seq_num());
            assert_eq!([0; 6], block.get_uid());
            assert_eq!(Version::V17, block.get_version());
        }
    }
}
