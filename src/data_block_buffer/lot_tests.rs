#![cfg(test)]
use super::*;
use crate::multihash::hash;
use crate::multihash::HashType;
use crate::rand_utils::fill_random_bytes;
use crate::sbx_block;
use crate::sbx_specs::Version;
use reed_solomon_erasure::ReedSolomon;

#[test]
#[should_panic]
fn new_panics_if_version_inconsistent_with_data_par_burst1() {
    let rs_codec = Arc::new(Some(ReedSolomon::new(10, 2).unwrap()));

    Lot::new(
        Version::V17,
        None,
        InputType::Block(BlockArrangement::Unordered),
        OutputType::Block,
        None,
        true,
        false,
        10,
        &rs_codec,
    );
}

#[test]
#[should_panic]
fn new_panics_if_version_inconsistent_with_data_par_burst2() {
    let rs_codec = Arc::new(None);

    Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::Unordered),
        OutputType::Block,
        Some((3, 2, 0)),
        true,
        false,
        10,
        &rs_codec,
    );
}

#[test]
#[should_panic]
fn new_panics_if_data_par_burst_inconsistent_with_rs_codec1() {
    let rs_codec = Arc::new(None);

    Lot::new(
        Version::V17,
        None,
        InputType::Block(BlockArrangement::Unordered),
        OutputType::Block,
        Some((3, 2, 0)),
        true,
        false,
        10,
        &rs_codec,
    );
}

#[test]
#[should_panic]
fn new_panics_if_data_par_burst_inconsistent_with_rs_codec2() {
    let rs_codec = Arc::new(Some(ReedSolomon::new(10, 2).unwrap()));

    Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::Unordered),
        OutputType::Block,
        None,
        true,
        false,
        10,
        &rs_codec,
    );
}

#[test]
#[should_panic]
fn cancel_slot_panics_when_empty1() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::Unordered),
        OutputType::Block,
        None,
        true,
        false,
        10,
        &Arc::new(None),
    );

    lot.cancel_slot();
}

#[test]
#[should_panic]
fn cancel_slot_panics_when_empty2() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::Unordered),
        OutputType::Block,
        None,
        true,
        false,
        10,
        &Arc::new(None),
    );

    let _ = lot.get_slot();

    lot.cancel_slot();
    lot.cancel_slot();
}

#[test]
fn hash_when_correct_arrangment1() {
    let lot = Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::OrderedAndNoMissing),
        OutputType::Block,
        None,
        true,
        false,
        10,
        &Arc::new(None),
    );

    let mut hash_ctx = hash::Ctx::new(HashType::SHA256).unwrap();

    lot.hash(&mut hash_ctx);
}

#[test]
fn hash_when_correct_arrangment2() {
    let lot = Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::OrderedButSomeMayBeMissing),
        OutputType::Block,
        None,
        true,
        false,
        10,
        &Arc::new(None),
    );

    let mut hash_ctx = hash::Ctx::new(HashType::SHA256).unwrap();

    lot.hash(&mut hash_ctx);
}

#[test]
#[should_panic]
fn hash_panics_when_incorrect_arrangement() {
    let lot = Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::Unordered),
        OutputType::Block,
        None,
        true,
        false,
        10,
        &Arc::new(None),
    );

    let mut hash_ctx = hash::Ctx::new(HashType::SHA256).unwrap();

    lot.hash(&mut hash_ctx);
}

#[test]
#[should_panic]
fn calc_slot_write_pos_panics_when_output_is_disabled() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::Unordered),
        OutputType::Disabled,
        None,
        true,
        false,
        10,
        &Arc::new(None),
    );

    lot.calc_slot_write_pos();
}

#[test]
fn fill_in_padding_when_input_type_is_data() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Data,
        OutputType::Disabled,
        None,
        true,
        false,
        10,
        &Arc::new(None),
    );

    lot.fill_in_padding();
}

#[test]
#[should_panic]
fn fill_in_padding_panics_when_input_type_is_block_and_arrangement_is_ordered_and_no_missing() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::OrderedAndNoMissing),
        OutputType::Disabled,
        None,
        true,
        false,
        10,
        &Arc::new(None),
    );

    lot.fill_in_padding();
}

#[test]
#[should_panic]
fn encode_panics_when_input_type_is_block_and_arrangement_is_ordered_and_no_missing() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block(BlockArrangement::OrderedAndNoMissing),
        OutputType::Disabled,
        None,
        true,
        false,
        10,
        &Arc::new(None),
    );

    lot.encode(1);
}

quickcheck! {
    #[should_panic]
    fn qc_cancel_slot_panics_when_empty(size: usize,
                                        cancels: usize) -> bool {
        let size = 1 + size % 1000;
        let cancels = 1 + cancels % 1000;

        let cancels = std::cmp::min(size, cancels);

        let mut lot = Lot::new(Version::V1,
                               None,
                               InputType::Block(BlockArrangement::Unordered),
                               OutputType::Block,
                               None,
                               true,
                               false,
                               size,
                               &Arc::new(None),
        );

        for _ in 0..cancels {
            let _ = lot.get_slot();
        }

        for _ in 0..cancels+1 {
            lot.cancel_slot();
        }

        true
    }

    fn qc_cancel_slot_when_not_empty(size: usize,
                                     cancels: usize) -> bool {
        let size = 1 + size % 1000;
        let cancels = 1 + cancels % 1000;

        let cancels = std::cmp::min(size, cancels);

        let mut lot = Lot::new(Version::V1,
                               None,
                               InputType::Block(BlockArrangement::Unordered),
                               OutputType::Block,
                               None,
                               true,
                               false,
                               size,
                               &Arc::new(None),
        );

        for _ in 0..cancels {
            let _ = lot.get_slot();
        }

        for _ in 0..cancels {
            lot.cancel_slot();
        }

        true
    }

    fn qc_get_slot_result(size: usize,
                          data: usize,
                          parity: usize,
                          burst: usize,
                          tries: usize) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let tries = 2 + tries % 100;

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            for _ in 0..tries {
                for _ in 0..size-1 {
                    match lot.get_slot() {
                        GetSlotResult::None => panic!(),
                        GetSlotResult::Some(_, _, _, _) => {},
                        GetSlotResult::LastSlot(_, _, _, _) => panic!(),
                    }
                }

                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(_, _, _, _) => panic!(),
                    GetSlotResult::LastSlot(_, _, _, _) => {},
                }

                match lot.get_slot() {
                    GetSlotResult::None => {},
                    GetSlotResult::Some(_, _, _, _) => panic!(),
                    GetSlotResult::LastSlot(_, _, _, _) => panic!(),
                }

                for _ in 0..size {
                    lot.cancel_slot();
                }
            }
        }

        true
    }

    fn qc_new_lot_stats(size: usize,
                        data: usize,
                        parity: usize,
                        burst: usize) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;

        ({
            let lot = Lot::new(Version::V1,
                               None,
                               InputType::Block(BlockArrangement::Unordered),
                               OutputType::Block,
                               None,
                               true,
                               false,
                               size,
                               &Arc::new(None),
            );

            lot.lot_size == size
                && lot.slots_used == 0
                && lot.padding_byte_count_in_non_padding_blocks == 0
                && lot.directly_writable_slots == size
        })
        &&
        ({
            let lot = Lot::new(Version::V17,
                               None,
                               InputType::Block(BlockArrangement::Unordered),
                               OutputType::Block,
                               Some((data, parity, burst)),
                               true,
                               false,
                               size,
                               &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
            );

            lot.lot_size == size
                && lot.slots_used == 0
                && lot.padding_byte_count_in_non_padding_blocks == 0
                && lot.directly_writable_slots == size
        })
        &&
        ({
            let lot = Lot::new(Version::V1,
                               None,
                               InputType::Data,
                               OutputType::Block,
                               None,
                               true,
                               false,
                               size,
                               &Arc::new(None),
            );

            lot.lot_size == size
                && lot.slots_used == 0
                && lot.padding_byte_count_in_non_padding_blocks == 0
                && lot.directly_writable_slots == size
        })
        &&
        ({
            let lot = Lot::new(Version::V17,
                               None,
                               InputType::Data,
                               OutputType::Block,
                               Some((data, parity, burst)),
                               true,
                               false,
                               size,
                               &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
            );

            lot.lot_size == data + parity
                && lot.slots_used == 0
                && lot.padding_byte_count_in_non_padding_blocks == 0
                && lot.directly_writable_slots == data
        })
    }

    fn qc_get_slot_and_cancel_slot_stats(size: usize,
                                         cancels: usize,
                                         data: usize,
                                         parity: usize,
                                         burst: usize,
                                         tries: usize) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let tries = 2 + tries % 100;

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let cancels = std::cmp::min(size, cancels);

            for _ in 0..tries {
                let mut res = true;

                for i in 0..cancels {
                    res = res && lot.slots_used == i;

                    let _ = lot.get_slot();

                    res = res && lot.slots_used == i + 1;
                }

                for i in (0..cancels).rev() {
                    res = res && lot.slots_used == i + 1;

                    lot.cancel_slot();

                    res = res && lot.slots_used == i;
                }

                if !res { return false; }
            }
        }

        true
    }

    fn qc_cancel_slot_resets_slot_correctly(size: usize,
                                            cancels: usize,
                                            data: usize,
                                            parity: usize,
                                            burst: usize,
                                            tries: usize) -> bool {
        let size = 1 + size % 1000;
        let cancels = 1 + cancels % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let tries = 2 + tries % 100;

        for lot_case in 1..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let cancels = std::cmp::min(size, cancels);

            for _ in 0..tries {
                let mut res = true;

                for _ in 0..cancels {
                    match lot.get_slot() {
                        GetSlotResult::None => {},
                        GetSlotResult::Some(block, _data, write_pos, content_len_exc_header)
                            | GetSlotResult::LastSlot(block, _data, write_pos, content_len_exc_header) => {
                                block.set_version(Version::V2);
                                block.set_uid([0xFF; SBX_FILE_UID_LEN]);
                                block.set_seq_num(2000);

                                *write_pos = Some(10);

                                *content_len_exc_header = Some(100);
                            }
                    }
                }

                let version = lot.version;
                let uid = lot.uid;

                for _ in 0..cancels {
                    lot.cancel_slot();

                    match lot.get_slot() {
                        GetSlotResult::None => panic!(),
                        GetSlotResult::Some(block, _data, read_pos, content_len_exc_header)
                            | GetSlotResult::LastSlot(block, _data, read_pos, content_len_exc_header) => {
                                res = res && block.get_version() == version;
                                res = res && block.get_uid() == uid;
                                res = res && block.get_seq_num() == 1;

                                res = res && *read_pos == None;

                                res = res && *content_len_exc_header == None;
                        },
                    }

                    lot.cancel_slot();
                }

                if !res { return false; }
            }
        }

        true
    }

    fn qc_new_slots_are_initialized_correctly(size: usize,
                                              data: usize,
                                              parity: usize,
                                              burst: usize) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let version = lot.version;
            let uid = lot.uid;

            let mut res = true;

            for _ in 0..size {
                match lot.get_slot() {
                    GetSlotResult::None => {},
                    GetSlotResult::Some(block, _data, read_pos, content_len_exc_header)
                        | GetSlotResult::LastSlot(block, _data, read_pos, content_len_exc_header) => {
                            res = res && block.get_version() == version;
                            res = res && block.get_uid() == uid;
                            res = res && block.get_seq_num() == 1;

                            res = res && *read_pos == None;

                            res = res && *content_len_exc_header == None;
                        },
                }
            }

            if !res { return false; }
        }

        true
    }

    fn qc_slots_are_reset_correctly_after_lot_reset(size: usize,
                                                    data: usize,
                                                    parity: usize,
                                                    burst: usize,
                                                    fill: usize) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let fill = 1 + fill % 1000;

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let fill = std::cmp::min(size, fill);

            for _ in 0..fill {
                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(block, _data, read_pos, content_len_exc_header)
                        | GetSlotResult::LastSlot(block, _data, read_pos, content_len_exc_header) => {
                            block.set_version(Version::V1);
                            block.set_uid([0xFF; SBX_FILE_UID_LEN]);
                            block.set_seq_num(2000);

                            *read_pos = Some(10);

                            *content_len_exc_header = Some(100);
                        },
                }
            }

            lot.reset();

            let version = lot.version;
            let uid = lot.uid;

            let mut res = true;

            for _ in 0..size {
                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(block, _data, read_pos, content_len_exc_header)
                        | GetSlotResult::LastSlot(block, _data, read_pos, content_len_exc_header) => {
                            res = res && block.get_version() == version;
                            res = res && block.get_uid() == uid;
                            res = res && block.get_seq_num() == 1;

                            res = res && *read_pos == None;

                            res = res && *content_len_exc_header == None;
                        },
                }
            }

            if !res { return false; }
        }

        true
    }

    fn qc_stats_are_reset_correctly_after_lot_reset(size: usize,
                                                    data: usize,
                                                    parity: usize,
                                                    burst: usize,
                                                    fill: usize) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let fill = 1 + fill % 1000;

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block(BlockArrangement::Unordered),
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let fill = std::cmp::min(size, fill);

            for _ in 0..fill {
                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(block, _data, read_pos, content_len_exc_header)
                        | GetSlotResult::LastSlot(block, _data, read_pos, content_len_exc_header) => {
                            block.set_version(Version::V1);
                            block.set_uid([0xFF; SBX_FILE_UID_LEN]);
                            block.set_seq_num(2000);

                            *read_pos = Some(10);

                            *content_len_exc_header = Some(100);
                        },
                }
            }

            lot.reset();

            let res =
                lot.slots_used == 0
                && lot.padding_byte_count_in_non_padding_blocks == 0;

            if !res { return false; }
        }

        true
    }

    fn qc_data_padding_parity_block_count_result_is_correct(size: usize,
                                                            lot_start_seq_num: u32,
                                                            data: usize,
                                                            parity: usize,
                                                            burst: usize,
                                                            fill: usize) -> bool {
        let size = 1 + size % 1000;
        let lot_start_seq_num = 1 + lot_start_seq_num % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let fill = 1 + fill % 1000;

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let writable_slots =
                if lot_case == 0 {
                    size
                } else {
                    data
                };

            let fill = std::cmp::min(writable_slots, fill);

            for _ in 0..fill {
                let _ = lot.get_slot();
            }

            {
                let (d, pad, p) = lot.data_padding_parity_block_count();

                if lot_case == 0 {
                    let res =
                        d == fill
                        && pad == 0
                        && p == 0;

                    if !res { return false; }
                } else {
                    if fill < data {
                        let res = d == fill && p == 0;
                        if !res { return false; }
                    } else {
                        let res = d == data && p == fill - data;
                        if !res { return false; }
                    }

                    let res = pad == 0;

                    if !res { return false; }
                }
            }

            lot.encode(lot_start_seq_num);

            {
                let (d, pad, p) = lot.data_padding_parity_block_count();

                if lot_case == 0 {
                    let res =
                        d == fill
                        && pad == 0
                        && p == 0;

                    if !res { return false; }
                } else {
                    let res =
                        d == data
                        && pad == data - fill
                        && p == parity;

                    if !res { return false; }
                }
            }
        }

        true
    }

    fn qc_encode_updates_stats_and_blocks_correctly(size: usize,
                                                    lot_start_seq_num: u32,
                                                    data: usize,
                                                    parity: usize,
                                                    burst: usize,
                                                    fill: usize) -> bool {
        let size = 1 + size % 1000;
        let lot_start_seq_num = 1 + lot_start_seq_num % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let fill = 1 + fill % 1000;

        for lot_case in 1..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let size =
                if lot_case == 0 {
                    size
                } else {
                    data + parity
                };

            let writable_slots =
                if lot_case == 0 {
                    size
                } else {
                    data
                };

            let fill = std::cmp::min(writable_slots, fill);

            let mut res = true;

            res = res && lot.slots_used == 0;

            for _ in 0..fill {
                let _ = lot.get_slot();
            }

            lot.encode(lot_start_seq_num);

            if lot_case == 0 {
                res = res && lot.slots_used == fill;
            } else {
                res = res && lot.slots_used == size;
            }

            for i in 0..lot.slots_used {
                res = res && lot.blocks[i].get_seq_num() == lot_start_seq_num + i as u32;
            }

            if !res { return false; }
        }

        true
    }

    fn qc_active_if_and_only_if_at_least_one_slot_in_use(size: usize,
                                                         data: usize,
                                                         parity: usize,
                                                         burst: usize,
                                                         fill: usize,
                                                         tries: usize) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let fill = 1 + fill % 1000;
        let tries = 2 + tries % 1000;

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block(BlockArrangement::OrderedAndNoMissing),
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block(BlockArrangement::OrderedAndNoMissing),
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let fill = std::cmp::min(size, fill);

            for _ in 0..tries {
                let mut res = true;

                res = res && !lot.active();

                for _ in 0..fill {
                    let _ = lot.get_slot();

                    res = res && lot.active();
                }

                for _ in 0..fill {
                    res = res && lot.active();

                    lot.cancel_slot();
                }

                res = res && !lot.active();

                if !res { return false; }
            }
        }

        true
    }

    fn qc_fill_in_padding_counts_padding_bytes_and_marks_padding_blocks_correctly(
        size: usize,
        data: usize,
        parity: usize,
        burst: usize,
        fill: usize,
        content_len: Vec<usize>,
        data_is_partial: Vec<bool>
    ) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let fill = 1 + fill % 1000;

        let mut content_len = content_len;
        let mut data_is_partial = data_is_partial;

        content_len.push(1);
        data_is_partial.push(true);

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let writable_slots =
                if lot_case == 0 {
                    size
                } else {
                    data
                };

            let fill = std::cmp::min(writable_slots, fill);

            let mut padding_bytes_in_non_padding_blocks = 0;

            for i in 0..fill {
                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(_block, _data, read_pos, content_len_exc_header)
                        | GetSlotResult::LastSlot(_block, _data, read_pos, content_len_exc_header) => {
                            if data_is_partial[i % data_is_partial.len()] {
                                let len = content_len[i % content_len.len()] % 496 + 1;
                                *content_len_exc_header = Some(len);
                                padding_bytes_in_non_padding_blocks += 496 - len;
                            }

                            *read_pos = Some(10);
                        },
                }
            }

            lot.fill_in_padding();

            let mut res = true;

            res = res && padding_bytes_in_non_padding_blocks == lot.padding_byte_count_in_non_padding_blocks;

            if lot_case == 0 {
                for is_padding in lot.slot_is_padding.iter() {
                    res = res && !*is_padding;
                }
            } else {
                for i in 0..fill {
                    res = res && !lot.slot_is_padding[i];
                }
                for i in fill..data {
                    res = res && lot.slot_is_padding[i];
                }
            }

            if !res { return false; }
        }

        true
    }

    fn qc_hash_ignores_metadata_and_parity_blocks_and_uses_content_len_correctly(
        size: usize,
        data: usize,
        parity: usize,
        burst: usize,
        fill: usize,
        seq_nums: Vec<u32>,
        content_len: Vec<usize>,
        data_is_partial: Vec<bool>
    ) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let fill = 1 + fill % 1000;

        let mut seq_nums = seq_nums;
        let mut content_len = content_len;
        let mut data_is_partial = data_is_partial;

        seq_nums.push(1);
        content_len.push(1);
        data_is_partial.push(true);

        for lot_case in 0..1 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block(BlockArrangement::OrderedAndNoMissing),
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block(BlockArrangement::OrderedAndNoMissing),
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let data_par_burst =
                if lot_case == 0 {
                    None
                } else {
                    Some((data, parity, burst))
                };

            let version = if lot_case == 0 {
                Version::V1
            } else {
                Version::V17
            };

            let fill = std::cmp::min(size, fill);

            let mut hash_ctx1 = hash::Ctx::new(HashType::SHA256).unwrap();

            for i in 0..fill {
                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(block, data, read_pos, content_len_exc_header)
                        | GetSlotResult::LastSlot(block, data, read_pos, content_len_exc_header) => {
                            let seq_num = seq_nums[i % seq_nums.len()];

                            fill_random_bytes(data);

                            let len =
                                if data_is_partial[i % data_is_partial.len()] {
                                    let len = content_len[i % content_len.len()] % 496 + 1;
                                    *content_len_exc_header = Some(len);
                                    len
                                } else {
                                    496
                                };

                            *read_pos = Some(10);

                            block.set_seq_num(seq_num);

                            if block.is_meta() {
                            } else if block.is_parity_w_data_par_burst(data_par_burst) {
                            } else {
                                hash_ctx1.update(&sbx_block::slice_data_buf(version, data)[..len]);
                            }
                        },
                }
            }

            let hash_res1 = hash_ctx1.finish_into_hash_bytes();

            let mut hash_ctx2 = hash::Ctx::new(HashType::SHA256).unwrap();

            lot.hash(&mut hash_ctx2);

            let hash_res2 = hash_ctx2.finish_into_hash_bytes();

            assert_eq!(hash_res1, hash_res2);
        }

        true
    }

    fn qc_lot_is_full_is_correct(
        size: usize,
        data: usize,
        parity: usize,
        burst: usize,
        tries: usize
    ) -> bool {
        let size = 1 + size % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let tries = 2 + tries % 1000;

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block(BlockArrangement::OrderedAndNoMissing),
                             OutputType::Block,
                             None,
                             true,
                             false,
                             size,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block(BlockArrangement::OrderedAndNoMissing),
                             OutputType::Block,
                             Some((data, parity, burst)),
                             true,
                             false,
                             size,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            for _ in 0..tries {
                let mut res = true;

                for _ in 0..size {
                    res = res && !lot_is_full!(lot);

                    let _ = lot.get_slot();
                }

                res = res && lot_is_full!(lot);

                for _ in 0..size {
                    lot.cancel_slot();

                    res = res && !lot_is_full!(lot);
                }

                if !res { return false; }
            }
        }

        true
    }
}
