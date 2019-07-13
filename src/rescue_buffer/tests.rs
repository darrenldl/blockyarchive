#![cfg(test)]
use super::*;

#[test]
#[should_panic]
fn cancel_slot_panics_when_empty1() {
    let mut buffer = RescueBuffer::new(2);

    buffer.cancel_slot();
}

#[test]
#[should_panic]
fn cancel_slot_panics_when_empty2() {
    let mut buffer = RescueBuffer::new(2);

    let _ = buffer.get_slot();

    buffer.cancel_slot();
    buffer.cancel_slot();
}

quickcheck! {
    #[should_panic]
    fn qc_cancel_slot_panics_when_empty(size: usize, cancels: usize) -> bool {
        let size = 1 + size % 1000;
        let cancels = 1 + cancels % 1000;

        let cancels = std::cmp::min(size, cancels);

        let mut buffer = RescueBuffer::new(2);

        for _ in 0..cancels {
            let _ = buffer.get_slot();
        }

        for _ in 0..cancels+1 {
            buffer.cancel_slot();
        }

        true
    }

    fn qc_cancel_slot_when_not_empty(size: usize,
                                     cancels: usize) -> bool {
        let size = 1 + size % 1000;
        let cancels = 1 + cancels % 1000;

        let cancels = std::cmp::min(size, cancels);

        let mut buffer = RescueBuffer::new(size);

        for _ in 0..cancels {
            let _ = buffer.get_slot();
        }

        for _ in 0..cancels {
            buffer.cancel_slot();
        }

        true
    }

    fn qc_get_slot_result(size: usize,
                          tries: usize) -> bool {
        let size = 1 + size % 1000;
        let tries = 2 + tries % 100;

        let mut buffer = RescueBuffer::new(size);

        for _ in 0..tries {
            for _ in 0..size {
                match buffer.get_slot() {
                    Some(_) => {}
                    None => panic!()
                }
            }

            match buffer.get_slot() {
                Some(_) => panic!(),
                None => {}
            }

            for _ in 0..size {
                buffer.cancel_slot()
            }
        }

        true
    }

    fn qc_new_lot_stats(size: usize) -> bool {
        let size = 1 + size % 1000;

        let buffer = RescueBuffer::new(size);

        buffer.size == size
            && buffer.slots_used == 0
    }

    fn qc_get_slot_and_cancel_slot_stats(size: usize,
                                         cancels: usize,
                                         tries: usize) -> bool {
        let size = 1 + size % 1000;
        let cancels = 1 + cancels % 1000;
        let tries = 2 + tries % 100;

        let mut buffer = RescueBuffer::new(size);

        let cancels = std::cmp::min(size, cancels);

        for _ in 0..tries {
            let mut res = true;

            for i in 0..cancels {
                res = res && buffer.slots_used == i;

                let _ = buffer.get_slot();

                res = res && buffer.slots_used == i + 1;
            }

            for i in (0..cancels).rev() {
                res = res && buffer.slots_used == i + 1;

                buffer.cancel_slot();

                res = res && buffer.slots_used == i;
            }

            if !res { return false; }
        }

        true
    }

    fn qc_cancel_slot_resets_slot_correctly(size: usize,
                                            cancels: usize,
                                            tries: usize) -> bool {
        let size = 1 + size % 1000;
        let cancels = 1 + cancels % 1000;
        let tries = 2 + tries % 100;

        let cancels = std::cmp::min(size, cancels);

        let mut buffer = RescueBuffer::new(size);

        for _ in 0..tries {
            let mut res = true;

            for _ in 0..cancels {
                match buffer.get_slot() {
                    Some(Slot {block, slot: _}) => {
                        block.set_version(Version::V2);
                        block.set_uid([0xFF; SBX_FILE_UID_LEN]);
                        block.set_seq_num(2000);
                    },
                    None => panic!()
                }
            }

            for _ in 0..cancels {
                buffer.cancel_slot();

                match buffer.get_slot() {
                    Some(Slot {block, slot: _}) => {
                        res = res && block.get_version() == Version::V1;
                        res = res && block.get_uid() == [0; SBX_FILE_UID_LEN];
                        res = res && block.get_seq_num() == SBX_FIRST_DATA_SEQ_NUM;
                    }
                    None => panic!()
                }

                buffer.cancel_slot();
            }

            if !res { return false; }
        }

        true
    }

    fn qc_new_slots_are_initialized_correctly(size: usize) -> bool {
        let size = 1 + size % 1000;

        let mut buffer = RescueBuffer::new(size);

        let mut res = true;

        for _ in 0..size {
            match buffer.get_slot() {
                Some(Slot {block, slot: _}) => {
                    res = res && block.get_version() == Version::V1;
                    res = res && block.get_uid() == [0; SBX_FILE_UID_LEN];
                    res = res && block.get_seq_num() == SBX_FIRST_DATA_SEQ_NUM;
                },
                None => {}
            }
        }

        res
    }

    fn qc_slots_are_reset_correctly_after_lot_reset(size: usize,
                                                    fill: usize) -> bool {
        let size = 1 + size % 1000;
        let fill = 1 + fill % 1000;

        let fill = std::cmp::min(size, fill);

        let mut buffer = RescueBuffer::new(size);

        for _ in 0..fill {
            match buffer.get_slot() {
                Some(Slot {block, slot: _}) => {
                    block.set_version(Version::V2);
                    block.set_uid([0xFF; SBX_FILE_UID_LEN]);
                    block.set_seq_num(2000);
                },
                None => panic!()
            }
        }

        buffer.reset();

        let mut res = true;

        for _ in 0..size {
            match buffer.get_slot() {
                Some(Slot {block, slot: _}) => {
                    res = res && block.get_version() == Version::V1;
                    res = res && block.get_uid() == [0; SBX_FILE_UID_LEN];
                    res = res && block.get_seq_num() == SBX_FIRST_DATA_SEQ_NUM;
                },
                None => {}
            }
        }

        res
    }

    fn qc_stats_are_reset_correctly_after_lot_reset(size: usize,
                                                    fill: usize) -> bool {
        let size = 1 + size % 1000;
        let fill = 1 + fill % 1000;

        let fill = std::cmp::min(size, fill);

        let mut buffer = RescueBuffer::new(size);

        for _ in 0..fill {
            match buffer.get_slot() {
                Some(Slot {block, slot: _}) => {
                    block.set_version(Version::V2);
                    block.set_uid([0xFF; SBX_FILE_UID_LEN]);
                    block.set_seq_num(2000);
                },
                None => panic!()
            }
        }

        buffer.reset();

        buffer.slots_used == 0
    }

    fn qc_is_full_is_correct(
        size: usize,
        tries: usize
    ) -> bool {
        let size = 1 + size % 1000;
        let tries = 2 + tries % 1000;

        let mut buffer = RescueBuffer::new(size);

        for _ in 0..tries {
            let mut res = true;

            for _ in 0..size {
                res = res && !buffer.is_full();

                let _ = buffer.get_slot();
            }

            res = res && buffer.is_full();

            for _ in 0..size {
                buffer.cancel_slot();

                res = res && !buffer.is_full();
            }

            if !res { return false; }
        }

        true
    }
}
