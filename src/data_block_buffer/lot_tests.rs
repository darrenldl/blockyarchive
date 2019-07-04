#![cfg(test)]
use super::*;
use proptest::prelude::*;
use crate::sbx_specs::{Version};
use crate::multihash::hash;
use crate::multihash::{HashType};

use crate::file_writer::{FileWriter, FileWriterParam};
use crate::writer::{Writer, WriterType};

#[test]
#[should_panic]
fn cancel_slot_panics_when_empty1() {
    let mut lot = Lot::new(Version::V17,
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
    let mut lot = Lot::new(Version::V17,
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
    let lot = Lot::new(Version::V17,
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
    let lot = Lot::new(Version::V17,
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
    let lot = Lot::new(Version::V17,
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
fn write_panics_when_output_is_block() {
    let mut lot = Lot::new(Version::V17,
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

    let mut writer = Writer::new(WriterType::File(FileWriter::new(
        "tests/dummy",
        FileWriterParam {
            read: false,
            append: false,
            truncate: true,
            buffered: false,
        },
    ).unwrap()));

    lot.write(false, &mut writer).unwrap();
}

#[test]
fn write_panics_when_output_is_data() {
    let mut lot = Lot::new(Version::V17,
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

    let mut writer = Writer::new(WriterType::File(FileWriter::new(
        "tests/dummy",
        FileWriterParam {
            read: false,
            append: false,
            truncate: true,
            buffered: false,
        },
    ).unwrap()));

    lot.write(false, &mut writer).unwrap();
}

#[test]
#[should_panic]
fn write_panics_when_output_is_disabled() {
    let mut lot = Lot::new(Version::V17,
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

    let mut writer = Writer::new(WriterType::File(FileWriter::new(
        "tests/dummy",
        FileWriterParam {
            read: false,
            append: false,
            truncate: true,
            buffered: false,
        },
    ).unwrap()));

    lot.write(false, &mut writer).unwrap();
}

#[test]
fn encode_when_input_type_is_data_and_arrangement_is_ordered_and_no_missing() {
    let mut lot = Lot::new(Version::V17,
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
    let mut lot = Lot::new(Version::V17,
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
    let mut lot = Lot::new(Version::V17,
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
    let mut lot = Lot::new(Version::V17,
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

        let mut lot = Lot::new(Version::V17,
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

        let mut lot = Lot::new(Version::V17,
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
                          data in 1usize..128,
                          parity in 1usize..128,
                          burst in 1usize..100,
                          tries in 2usize..100) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V17,
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
                             &Arc::new(None),
                    )
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
                        data in 1usize..128,
                        parity in 1usize..128,
                        burst in 1usize..100) {
        {
            let lot = Lot::new(Version::V17,
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
                               &Arc::new(None),
            );

            assert_eq!(lot.lot_size, size);
            assert_eq!(lot.slots_used, 0);
            assert_eq!(lot.padding_byte_count_in_non_padding_blocks, 0);
            assert_eq!(lot.directly_writable_slots, data);
        }
    }

    #[test]
    fn pt_get_slot_and_cancel_slot_stats(size in 1usize..1000,
                                         cancels in 1usize..1000,
                                         data in 1usize..128,
                                         parity in 1usize..128,
                                         burst in 1usize..100,
                                         tries in 2usize..100) {
        let cancels = std::cmp::min(size, cancels);

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V17,
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
                             &Arc::new(None),
                    )
                };

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
                                            data in 1usize..128,
                                            parity in 1usize..128,
                                            burst in 1usize..100,
                                            tries in 2usize..100) {
        let cancels = std::cmp::min(size, cancels);

        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V17,
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
                             &Arc::new(None),
                    )
                };

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
                                              data in 1usize..128,
                                              parity in 1usize..128,
                                              burst in 1usize..100) {
        for lot_case in 0..2 {
            let mut lot =
                if lot_case == 0 {
                    Lot::new(Version::V17,
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
                             &Arc::new(None),
                    )
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
}
