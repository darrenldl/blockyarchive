#![cfg(test)]
use super::*;
use crate::multihash::hash;
use crate::multihash::HashType;
use crate::sbx_specs::Version;
use proptest::prelude::*;
use reed_solomon_erasure::ReedSolomon;

use crate::file_writer::{FileWriter, FileWriterParam};
use crate::writer::{Writer, WriterType};

#[test]
#[should_panic]
fn new_panics_if_version_inconsistent_with_data_par_burst1() {
    let rs_codec = Arc::new(Some(ReedSolomon::new(10, 2).unwrap()));

    Lot::new(
        Version::V17,
        None,
        InputType::Block,
        OutputType::Block,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
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
        InputType::Block,
        OutputType::Block,
        BlockArrangement::Unordered,
        Some((3, 2, 0)),
        true,
        10,
        false,
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
        InputType::Block,
        OutputType::Block,
        BlockArrangement::Unordered,
        Some((3, 2, 0)),
        true,
        10,
        false,
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
        InputType::Block,
        OutputType::Block,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
        &rs_codec,
    );
}

#[test]
#[should_panic]
fn cancel_slot_panics_when_empty1() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block,
        OutputType::Block,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
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
        InputType::Block,
        OutputType::Block,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
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
        InputType::Block,
        OutputType::Block,
        BlockArrangement::OrderedAndNoMissing,
        None,
        true,
        10,
        false,
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
        InputType::Block,
        OutputType::Block,
        BlockArrangement::OrderedButSomeMayBeMissing,
        None,
        true,
        10,
        false,
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
        InputType::Block,
        OutputType::Block,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
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
        InputType::Block,
        OutputType::Disabled,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    lot.calc_slot_write_pos();
}

#[test]
fn write_does_not_panic_when_output_is_block() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block,
        OutputType::Block,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    let mut writer = Writer::new(WriterType::File(
        FileWriter::new(
            "tests/dummy",
            FileWriterParam {
                read: false,
                append: false,
                truncate: true,
                buffered: false,
            },
        )
        .unwrap(),
    ));

    lot.write(false, &mut writer).unwrap();
}

#[test]
fn write_does_not_panic_when_output_is_data() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block,
        OutputType::Data,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    let mut writer = Writer::new(WriterType::File(
        FileWriter::new(
            "tests/dummy",
            FileWriterParam {
                read: false,
                append: false,
                truncate: true,
                buffered: false,
            },
        )
        .unwrap(),
    ));

    lot.write(false, &mut writer).unwrap();
}

#[test]
#[should_panic]
fn write_panics_when_output_is_disabled() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block,
        OutputType::Disabled,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    let mut writer = Writer::new(WriterType::File(
        FileWriter::new(
            "tests/dummy",
            FileWriterParam {
                read: false,
                append: false,
                truncate: true,
                buffered: false,
            },
        )
        .unwrap(),
    ));

    lot.write(false, &mut writer).unwrap();
}

#[test]
fn fill_in_padding_when_input_type_is_data_and_arrangement_is_ordered_and_no_missing() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Data,
        OutputType::Disabled,
        BlockArrangement::OrderedAndNoMissing,
        None,
        true,
        10,
        false,
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
        InputType::Block,
        OutputType::Disabled,
        BlockArrangement::OrderedAndNoMissing,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    lot.fill_in_padding();
}

#[test]
#[should_panic]
fn fill_in_padding_panics_when_input_type_is_data_and_arrangement_is_not_ordered_and_no_missing1() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Data,
        OutputType::Disabled,
        BlockArrangement::OrderedButSomeMayBeMissing,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    lot.fill_in_padding();
}

#[test]
#[should_panic]
fn fill_in_padding_panics_when_input_type_is_data_and_arrangement_is_not_ordered_and_no_missing2() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Data,
        OutputType::Disabled,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    lot.fill_in_padding();
}

#[test]
fn encode_when_input_type_is_data_and_arrangement_is_ordered_and_no_missing() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Data,
        OutputType::Disabled,
        BlockArrangement::OrderedAndNoMissing,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    lot.encode(1);
}

#[test]
#[should_panic]
fn encode_panics_when_input_type_is_block_and_arrangement_is_ordered_and_no_missing() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Block,
        OutputType::Disabled,
        BlockArrangement::OrderedAndNoMissing,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    lot.encode(1);
}

#[test]
#[should_panic]
fn encode_panics_when_input_type_is_data_and_arrangement_is_not_ordered_and_no_missing1() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Data,
        OutputType::Disabled,
        BlockArrangement::OrderedButSomeMayBeMissing,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    lot.encode(1);
}

#[test]
#[should_panic]
fn encode_panics_when_input_type_is_data_and_arrangement_is_not_ordered_and_no_missing2() {
    let mut lot = Lot::new(
        Version::V1,
        None,
        InputType::Data,
        OutputType::Disabled,
        BlockArrangement::Unordered,
        None,
        true,
        10,
        false,
        &Arc::new(None),
    );

    lot.encode(1);
}

proptest! {
    #[test]
    #[should_panic]
    fn pt_cancel_slot_panics_when_empty(size in 1usize..1000,
                                             cancels in 1usize..1000) {
        let cancels = std::cmp::min(size, cancels);

        let mut lot = Lot::new(Version::V1,
                               None,
                               InputType::Block,
                               OutputType::Block,
                               BlockArrangement::Unordered,
                               None,
                               true,
                               size,
                               false,
                               &Arc::new(None),
        );

        for _ in 0..cancels {
            let _ = lot.get_slot();
        }

        for _ in 0..cancels+1 {
            lot.cancel_slot();
        }
    }

    #[test]
    fn pt_cancel_slot_when_not_empty(size in 1usize..1000,
                                          cancels in 1usize..1000) {
        let cancels = std::cmp::min(size, cancels);

        let mut lot = Lot::new(Version::V1,
                               None,
                               InputType::Block,
                               OutputType::Block,
                               BlockArrangement::Unordered,
                               None,
                               true,
                               size,
                               false,
                               &Arc::new(None),
        );

        for _ in 0..cancels {
            let _ = lot.get_slot();
        }

        for _ in 0..cancels {
            lot.cancel_slot();
        }
    }

    #[test]
    fn pt_get_slot_result(size in 1usize..1000,
                          data in 1usize..30,
                          parity in 1usize..30,
                          burst in 1usize..100,
                          tries in 2usize..100) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let size =
                if lot_case == 0 {
                    size
                } else {
                    data + parity
                };

            for _ in 0..tries {
                for _ in 0..size-1 {
                    match lot.get_slot() {
                        GetSlotResult::None => panic!(),
                        GetSlotResult::Some(_, _, _) => {},
                        GetSlotResult::LastSlot(_, _, _) => panic!(),
                    }
                }

                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(_, _, _) => panic!(),
                    GetSlotResult::LastSlot(_, _, _) => {},
                }

                match lot.get_slot() {
                    GetSlotResult::None => {},
                    GetSlotResult::Some(_, _, _) => panic!(),
                    GetSlotResult::LastSlot(_, _, _) => panic!(),
                }

                for _ in 0..size {
                    lot.cancel_slot();
                }
            }
        }
    }

    #[test]
    fn pt_new_lot_stats(size in 1usize..1000,
                        data in 1usize..30,
                        parity in 1usize..30,
                        burst in 1usize..100) {
        {
            let lot = Lot::new(Version::V1,
                               None,
                               InputType::Block,
                               OutputType::Block,
                               BlockArrangement::Unordered,
                               None,
                               true,
                               size,
                               false,
                               &Arc::new(None),
            );

            assert_eq!(lot.lot_size, size);
            assert_eq!(lot.slots_used, 0);
            assert_eq!(lot.padding_byte_count_in_non_padding_blocks, 0);
            assert_eq!(lot.directly_writable_slots, size);
        }
        {
            let lot = Lot::new(Version::V17,
                               None,
                               InputType::Block,
                               OutputType::Block,
                               BlockArrangement::Unordered,
                               Some((data, parity, burst)),
                               true,
                               size,
                               false,
                               &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
            );

            assert_eq!(lot.lot_size, data + parity);
            assert_eq!(lot.slots_used, 0);
            assert_eq!(lot.padding_byte_count_in_non_padding_blocks, 0);
            assert_eq!(lot.directly_writable_slots, data + parity);
        }
        {
            let lot = Lot::new(Version::V1,
                               None,
                               InputType::Data,
                               OutputType::Block,
                               BlockArrangement::Unordered,
                               None,
                               true,
                               size,
                               false,
                               &Arc::new(None),
            );

            assert_eq!(lot.lot_size, size);
            assert_eq!(lot.slots_used, 0);
            assert_eq!(lot.padding_byte_count_in_non_padding_blocks, 0);
            assert_eq!(lot.directly_writable_slots, size);
        }
        {
            let lot = Lot::new(Version::V17,
                               None,
                               InputType::Data,
                               OutputType::Block,
                               BlockArrangement::Unordered,
                               Some((data, parity, burst)),
                               true,
                               size,
                               false,
                               &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
            );

            assert_eq!(lot.lot_size, data + parity);
            assert_eq!(lot.slots_used, 0);
            assert_eq!(lot.padding_byte_count_in_non_padding_blocks, 0);
            assert_eq!(lot.directly_writable_slots, data);
        }
    }

    #[test]
    fn pt_get_slot_and_cancel_slot_stats(size in 1usize..1000,
                                         cancels in 1usize..1000,
                                         data in 1usize..30,
                                         parity in 1usize..30,
                                         burst in 1usize..100,
                                         tries in 2usize..100) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let size =
                if lot_case == 0 {
                    size
                } else {
                    data + parity
                };

            let cancels = std::cmp::min(size, cancels);

            for _ in 0..tries {
                for i in 0..cancels {
                    assert_eq!(lot.slots_used, i);

                    let _ = lot.get_slot();

                    assert_eq!(lot.slots_used, i+1);
                }

                for i in (0..cancels).rev() {
                    assert_eq!(lot.slots_used, i+1);

                    lot.cancel_slot();

                    assert_eq!(lot.slots_used, i);
                }
            }
        }
    }

    #[test]
    fn pt_cancel_slot_resets_slot_correctly(size in 1usize..1000,
                                            cancels in 1usize..1000,
                                            data in 1usize..30,
                                            parity in 1usize..30,
                                            burst in 1usize..100,
                                            tries in 2usize..100) {
        for lot_case in 1..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let size =
                if lot_case == 0 {
                    size
                } else {
                    data + parity
                };

            let cancels = std::cmp::min(size, cancels);

            for _ in 0..tries {
                for _ in 0..cancels {
                    match lot.get_slot() {
                        GetSlotResult::None => {},
                        GetSlotResult::Some(block, _data, content_len_exc_header)
                            | GetSlotResult::LastSlot(block, _data, content_len_exc_header) => {
                                block.set_version(Version::V1);
                                block.set_uid([0xFF; SBX_FILE_UID_LEN]);
                                block.set_seq_num(2000);

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
                        GetSlotResult::Some(block, _data, content_len_exc_header)
                            | GetSlotResult::LastSlot(block, _data, content_len_exc_header) => {
                                assert_eq!(block.get_version(), version);
                                assert_eq!(block.get_uid(), uid);
                                assert_eq!(block.get_seq_num(), 1);

                                assert_eq!(*content_len_exc_header, None);
                        },
                    }

                    lot.cancel_slot();
                }
            }
        }
    }

    #[test]
    fn pt_new_slots_are_initialized_correctly(size in 1usize..1000,
                                              data in 1usize..30,
                                              parity in 1usize..30,
                                              burst in 1usize..100) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let size =
                if lot_case == 0 {
                    size
                } else {
                    data + parity
                };

            let version = lot.version;
            let uid = lot.uid;

            for _ in 0..size {
                match lot.get_slot() {
                    GetSlotResult::None => {},
                    GetSlotResult::Some(block, _data, content_len_exc_header)
                        | GetSlotResult::LastSlot(block, _data, content_len_exc_header) => {
                            assert_eq!(block.get_version(), version);
                            assert_eq!(block.get_uid(), uid);
                            assert_eq!(block.get_seq_num(), 1);

                            assert_eq!(*content_len_exc_header, None);
                        },
                }
            }
        }
    }

    #[test]
    fn pt_slots_are_reset_correctly_after_lot_reset(size in 1usize..1000,
                                                    data in 1usize..30,
                                                    parity in 1usize..30,
                                                    burst in 1usize..100,
                                                    fill in 1usize..1000) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let size =
                if lot_case == 0 {
                    size
                } else {
                    data + parity
                };

            let fill = std::cmp::min(size, fill);

            for _ in 0..fill {
                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(block, _data, content_len_exc_header)
                        | GetSlotResult::LastSlot(block, _data, content_len_exc_header) => {
                            block.set_version(Version::V1);
                            block.set_uid([0xFF; SBX_FILE_UID_LEN]);
                            block.set_seq_num(2000);

                            *content_len_exc_header = Some(100);
                        },
                }
            }

            lot.reset();

            let version = lot.version;
            let uid = lot.uid;

            for _ in 0..size {
                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(block, _data, content_len_exc_header)
                        | GetSlotResult::LastSlot(block, _data, content_len_exc_header) => {
                            assert_eq!(block.get_version(), version);
                            assert_eq!(block.get_uid(), uid);
                            assert_eq!(block.get_seq_num(), 1);

                            assert_eq!(*content_len_exc_header, None);
                        },
                }
            }
        }
    }

    #[test]
    fn pt_stats_are_reset_correctly_after_lot_reset(size in 1usize..1000,
                                                    data in 1usize..30,
                                                    parity in 1usize..30,
                                                    burst in 1usize..100,
                                                    fill in 1usize..1000) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::Unordered,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let size =
                if lot_case == 0 {
                    size
                } else {
                    data + parity
                };

            let fill = std::cmp::min(size, fill);

            for _ in 0..fill {
                match lot.get_slot() {
                    GetSlotResult::None => panic!(),
                    GetSlotResult::Some(block, _data, content_len_exc_header)
                        | GetSlotResult::LastSlot(block, _data, content_len_exc_header) => {
                            block.set_version(Version::V1);
                            block.set_uid([0xFF; SBX_FILE_UID_LEN]);
                            block.set_seq_num(2000);

                            *content_len_exc_header = Some(100);
                        },
                }
            }

            lot.reset();

            assert_eq!(lot.slots_used, 0);
            assert_eq!(lot.padding_byte_count_in_non_padding_blocks, 0);
        }
    }

    #[test]
    fn pt_data_padding_parity_block_count_result_is_correct(size in 1usize..1000,
                                                            lot_start_seq_num in 1u32..1000,
                                                            data in 1usize..30,
                                                            parity in 1usize..30,
                                                            burst in 1usize..100,
                                                            fill in 1usize..1000) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             BlockArrangement::OrderedAndNoMissing,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             BlockArrangement::OrderedAndNoMissing,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
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
                    assert_eq!(d, fill);
                    assert_eq!(pad, 0);
                    assert_eq!(p, 0);
                } else {
                    if fill < data {
                        assert_eq!(d, fill);
                        assert_eq!(p, 0);
                    } else {
                        assert_eq!(d, data);
                        assert_eq!(p, fill - data);
                    }
                    assert_eq!(pad, 0);
                }
            }

            lot.encode(lot_start_seq_num);

            {
                let (d, pad, p) = lot.data_padding_parity_block_count();

                if lot_case == 0 {
                    assert_eq!(d, fill);
                    assert_eq!(pad, 0);
                    assert_eq!(p, 0);
                } else {
                    assert_eq!(d, data);
                    assert_eq!(pad, data - fill);
                    assert_eq!(p, parity);
                }
            }
        }
    }

    #[test]
    fn pt_encode_updates_stats_and_blocks_correctly(size in 1usize..1000,
                                                    lot_start_seq_num in 1u32..1000,
                                                    data in 1usize..30,
                                                    parity in 1usize..30,
                                                    burst in 1usize..100,
                                                    fill in 1usize..1000) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             BlockArrangement::OrderedAndNoMissing,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             BlockArrangement::OrderedAndNoMissing,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let writable_slots =
                if lot_case == 0 {
                    size
                } else {
                    data
                };

            let size =
                if lot_case == 0 {
                    size
                } else {
                    data + parity
                };

            let fill = std::cmp::min(writable_slots, fill);

            assert_eq!(lot.slots_used, 0);

            for _ in 0..fill {
                let _ = lot.get_slot();
            }

            lot.encode(lot_start_seq_num);

            if lot_case == 0 {
                assert_eq!(lot.slots_used, fill);
            } else {
                assert_eq!(lot.slots_used, size);
            }

            for i in 0..lot.slots_used {
                assert_eq!(lot.blocks[i].get_seq_num(), lot_start_seq_num + i as u32);
            }
        }
    }

    #[test]
    fn pt_active_if_and_only_if_at_least_one_slot_in_use(size in 1usize..1000,
                                                         data in 1usize..30,
                                                         parity in 1usize..30,
                                                         burst in 1usize..100,
                                                         fill in 1usize..1000,
                                                         tries in 2usize..100) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::OrderedAndNoMissing,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Block,
                             OutputType::Block,
                             BlockArrangement::OrderedAndNoMissing,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
                             &Arc::new(Some(ReedSolomon::new(data, parity).unwrap())),
                    )
                };

            let size =
                if lot_case == 0 {
                    size
                } else {
                    data + parity
                };

            let fill = std::cmp::min(size, fill);

            for _ in 0..tries {
                assert!(!lot.active());

                for _ in 0..fill {
                    let _ = lot.get_slot();

                    assert!(lot.active());
                }

                for _ in 0..fill {
                    assert!(lot.active());

                    lot.cancel_slot();
                }

                assert!(!lot.active());
            }
        }
    }

    #[test]
    fn pt_fill_in_padding_marks_padding_blocks_correctly(size in 1usize..1000,
                                                         data in 1usize..30,
                                                         parity in 1usize..30,
                                                         burst in 1usize..100,
                                                         fill in 1usize..1000) {

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V1,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             BlockArrangement::OrderedAndNoMissing,
                             None,
                             true,
                             size,
                             false,
                             &Arc::new(None),
                    )
                } else {
                    Lot::new(Version::V17,
                             None,
                             InputType::Data,
                             OutputType::Block,
                             BlockArrangement::OrderedAndNoMissing,
                             Some((data, parity, burst)),
                             true,
                             size,
                             false,
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

            lot.fill_in_padding();

            if lot_case == 0 {
                for is_padding in lot.slot_is_padding.iter() {
                    assert_eq!(*is_padding, false);
                }
            } else {
                for i in 0..fill {
                    assert_eq!(lot.slot_is_padding[i], false);
                }
                for i in fill..data {
                    assert_eq!(lot.slot_is_padding[i], true);
                }
            }
        }
    }
}
