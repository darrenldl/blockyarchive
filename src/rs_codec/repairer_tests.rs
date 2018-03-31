#![cfg(test)]
use super::*;
use super::repairer::*;
use sbx_block;
use sbx_block::Block;
use sbx_block::BlockType;
use sbx_specs::{Version,
                SBX_LARGEST_BLOCK_SIZE};
use reed_solomon_erasure::ReedSolomon;

use rand;

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

                let codec_state =
                    if i == 0 || i == 5 || i == 11 {
                        fill_random_bytes(repairer.get_block_buffer());
                        repairer.mark_missing()
                    } else {
                        repairer.get_block_buffer().copy_from_slice(b);
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

    for _ in 0..2 {
        {
            let mut refs = Vec::new();
            for b in buffer.iter_mut() {
                refs.push(sbx_block::slice_buf(version, b));
            }

            // mark 0, 2, 3, 12 as missing
            let mut i = 0;
            for b in refs.iter() {
                assert_eq!(13 - i, repairer.unfilled_slot_count());
                assert_eq!(13, repairer.total_slot_count());

                repairer.get_block_buffer().copy_from_slice(b);
                let codec_state =
                    if i == 0 || i == 2 || i == 3 || i == 12 {
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
        assert!(!stats.successful);
        assert_eq!(1, stats.start_seq_num);
        assert_eq!(false, stats.present[0]);
        assert_eq!(true,  stats.present[1]);
        assert_eq!(false, stats.present[2]);
        assert_eq!(false, stats.present[3]);
        assert_eq!(true,  stats.present[4]);
        assert_eq!(true,  stats.present[5]);
        assert_eq!(true,  stats.present[6]);
        assert_eq!(true,  stats.present[7]);
        assert_eq!(true,  stats.present[8]);
        assert_eq!(true,  stats.present[9]);
        assert_eq!(true,  stats.present[10]);
        assert_eq!(true,  stats.present[11]);
        assert_eq!(false, stats.present[12]);

        assert_eq!(13, stats.present.len());
        assert_eq!(4, stats.missing_count);
        assert_eq!(9, stats.present_count);

        assert_eq!(0, blocks.len());
    }
}

quickcheck! {
    fn qc_repairer_repair_properly(data    : usize,
                                   parity  : usize,
                                   burst   : usize,
                                   corrupt : usize,
                                   reuse   : usize,
                                   seq_num : u32) -> bool {
        let data   = 1 + data % 10;
        let parity = 1 + parity % 10;
        let burst  = 1 + burst % 10;

        let reuse = reuse % 10;

        let seq_num = if seq_num == 0 { 1 } else { seq_num };

        let r = ReedSolomon::new(data, parity).unwrap();

        let mut buffer = make_random_block_buffers!(SBX_LARGEST_BLOCK_SIZE, data + parity);

        let versions = vec![Version::V17, Version::V18, Version::V19];

        // success case
        for &version in versions.iter () {
            let mut uid : [u8; 6] = [0; 6];
            fill_random_bytes(&mut uid);
            let ref_block = Block::new(version, &uid, BlockType::Data);
            let mut repairer = RSRepairer::new(&ref_block, data, parity, burst);

            if !(!repairer.active()
                 && repairer.unfilled_slot_count() == data + parity
                 && repairer.total_slot_count() == data + parity) {
                return false;
            }

            let corrupt = corrupt % (parity + 1);

            {
                let mut refs = Vec::new();
                for b in buffer.iter_mut() {
                    refs.push(sbx_block::slice_data_buf_mut(version, b));
                }

                r.encode(&mut refs).unwrap();
            }

            let mut corrupt_pos_s = Vec::with_capacity(corrupt);
            for _ in 0..corrupt {
                let mut pos = rand::random::<usize>() % (data + parity);

                while let Some(_) = corrupt_pos_s.iter().find(|&&x| x == pos) {
                    pos = rand::random::<usize>() % (data + parity);
                }

                corrupt_pos_s.push(pos);
            }

            corrupt_pos_s.sort();

            for _ in 0..1 + reuse {
                {
                    let mut refs = Vec::new();
                    for b in buffer.iter_mut() {
                        refs.push(sbx_block::slice_buf(version, b));
                    }

                    let mut i = 0;
                    for b in refs.iter() {
                        if repairer.unfilled_slot_count() != data + parity - i {
                            return false;
                        }
                        if repairer.total_slot_count() != data + parity {
                            return false;
                        }

                        let corrupted =
                            match corrupt_pos_s.iter().find(|&&x| x == i) {
                                None    => false,
                                Some(_) => true,
                            };

                        let codec_state =
                            if corrupted {
                                fill_random_bytes(repairer.get_block_buffer());
                                repairer.mark_missing()
                            } else {
                                repairer.get_block_buffer().copy_from_slice(b);
                                repairer.mark_present()
                            };

                        if data + parity - i - 1 != repairer.unfilled_slot_count() {
                            return false;
                        }
                        if data + parity != repairer.total_slot_count() {
                            return false;
                        }

                        if i == data + parity - 1 {
                            if codec_state != RSCodecState::Ready {
                                return false;
                            }
                        } else {
                            if codec_state != RSCodecState::NotReady {
                                return false;
                            }
                        }

                        i += 1;
                    }
                }

                let (stats, blocks) = repairer.repair_with_block_sync(seq_num);

                let block_set_size = (data + parity) as u32;
                let start_seq_num = (seq_num - 1) / block_set_size * block_set_size + 1;

                if !(stats.version == version
                     && stats.data_par_burst == (data, parity, burst)
                     && stats.successful)
                {
                    return false;
                }

                for i in 0..data + parity {
                    let corrupted =
                        match corrupt_pos_s.iter().find(|&&x| x == i) {
                            None    => false,
                            Some(_) => true,
                        };

                    if !((corrupted && !stats.present[i])
                         || (!corrupted && stats.present[i]))
                    {
                        return false;
                    }
                }

                if !(stats.present.len() == data + parity
                     && stats.missing_count == corrupt_pos_s.len()
                     && stats.present_count == data + parity - corrupt_pos_s.len()
                     && stats.start_seq_num == start_seq_num) {
                    return false;
                }

                if blocks.len() != corrupt_pos_s.len() {
                    return false;
                }

                for (i, &(p, ref b)) in blocks.iter().enumerate() {
                    let pos = sbx_block::calc_data_block_write_pos(version,
                                                                   start_seq_num + corrupt_pos_s[i] as u32,
                                                                   None,
                                                                   Some((data, parity, burst)));
                    if p != pos {
                        return false;
                    }

                    let mut block = Block::dummy();

                    block.sync_from_buffer(b, None).unwrap();
                    if !(block.get_seq_num() == start_seq_num + corrupt_pos_s[i] as u32
                         && block.get_uid() == uid
                         && block.get_version() == version) {
                        return false;
                    }
                }
            }
        }

        // failure case
        for &version in versions.iter () {
            let mut uid : [u8; 6] = [0; 6];
            fill_random_bytes(&mut uid);
            let ref_block = Block::new(version, &uid, BlockType::Data);
            let mut repairer = RSRepairer::new(&ref_block, data, parity, burst);

            if !(!repairer.active()
                 && repairer.unfilled_slot_count() == data + parity
                 && repairer.total_slot_count() == data + parity) {
                return false;
            }

            let corrupt = parity + 1 + corrupt % (data);

            {
                let mut refs = Vec::new();
                for b in buffer.iter_mut() {
                    refs.push(sbx_block::slice_data_buf_mut(version, b));
                }

                r.encode(&mut refs).unwrap();
            }

            let mut corrupt_pos_s = Vec::with_capacity(corrupt);
            for _ in 0..corrupt {
                let mut pos = rand::random::<usize>() % (data + parity);

                while let Some(_) = corrupt_pos_s.iter().find(|&&x| x == pos) {
                    pos = rand::random::<usize>() % (data + parity);
                }

                corrupt_pos_s.push(pos);
            }

            corrupt_pos_s.sort();

            for _ in 0..1 + reuse {
                {
                    let mut refs = Vec::new();
                    for b in buffer.iter_mut() {
                        refs.push(sbx_block::slice_buf(version, b));
                    }

                    let mut i = 0;
                    for b in refs.iter() {
                        if repairer.unfilled_slot_count() != data + parity - i {
                            return false;
                        }
                        if repairer.total_slot_count() != data + parity {
                            return false;
                        }

                        let corrupted =
                            match corrupt_pos_s.iter().find(|&&x| x == i) {
                                None    => false,
                                Some(_) => true,
                            };

                        let codec_state =
                            if corrupted {
                                fill_random_bytes(repairer.get_block_buffer());
                                repairer.mark_missing()
                            } else {
                                repairer.get_block_buffer().copy_from_slice(b);
                                repairer.mark_present()
                            };

                        if data + parity - i - 1 != repairer.unfilled_slot_count() {
                            return false;
                        }
                        if data + parity != repairer.total_slot_count() {
                            return false;
                        }

                        if i == data + parity - 1 {
                            if codec_state != RSCodecState::Ready {
                                return false;
                            }
                        } else {
                            if codec_state != RSCodecState::NotReady {
                                return false;
                            }
                        }

                        i += 1;
                    }
                }

                let (stats, blocks) = repairer.repair_with_block_sync(seq_num);

                let block_set_size = (data + parity) as u32;
                let start_seq_num = (seq_num - 1) / block_set_size * block_set_size + 1;

                if !(stats.version == version
                     && stats.data_par_burst == (data, parity, burst)
                     && !stats.successful)
                {
                    return false;
                }

                for i in 0..data + parity {
                    let corrupted =
                        match corrupt_pos_s.iter().find(|&&x| x == i) {
                            None    => false,
                            Some(_) => true,
                        };

                    if !((corrupted && !stats.present[i])
                         || (!corrupted && stats.present[i]))
                    {
                        return false;
                    }
                }

                if !(stats.present.len() == data + parity
                     && stats.missing_count == corrupt_pos_s.len()
                     && stats.present_count == data + parity - corrupt_pos_s.len()
                     && stats.start_seq_num == start_seq_num) {
                    return false;
                }

                if blocks.len() != 0 {
                    return false;
                }
            }
        }

        true
    }
}
