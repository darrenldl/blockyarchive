#![cfg(test)]
use super::*;
use crate::sbx_specs::Version;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn pt_cancel_slot_panics_when_empty(buffer_index in 1usize..1000,
                                        total_buffer_count in 1usize..1000,
                                        cancels in 1usize..1000) {
        let mut buffer = DataBlockBuffer::new(Version::V1,
                                              None,
                                              InputType::Block,
                                              OutputType::Block,
                                              BlockArrangement::Unordered,
                                              None,
                                              true,
                                              false,
                                              buffer_index,
                                              total_buffer_count,
        );

        let size = buffer.lots.len() * buffer.lot_size;

        let cancels = std::cmp::min(size, cancels);

        for _ in 0..cancels {
            let _ = buffer.get_slot();
        }

        for _ in 0..cancels+1 {
            buffer.cancel_slot();
        }
    }

    #[test]
    fn pt_cancel_slot_when_not_empty(buffer_index in 1usize..1000,
                                     total_buffer_count in 1usize..1000,
                                     cancels in 1usize..1000) {
        let mut buffer = DataBlockBuffer::new(Version::V1,
                                              None,
                                              InputType::Block,
                                              OutputType::Block,
                                              BlockArrangement::Unordered,
                                              None,
                                              true,
                                              false,
                                              buffer_index,
                                              total_buffer_count,
        );

        let size = buffer.lots.len() * buffer.lot_size;

        let cancels = std::cmp::min(size, cancels);

        for _ in 0..cancels {
            let _ = buffer.get_slot();
        }

        for _ in 0..cancels {
            buffer.cancel_slot();
        }
    }

    #[test]
    fn pt_stats_are_reset_correctly_after_buffer_reset(buffer_index in 1usize..1000,
                                                       total_buffer_count in 1usize..1000,
                                                       data in 1usize..30,
                                                       parity in 1usize..30,
                                                       burst in 1usize..100,
                                                       fill in 1usize..1000) {
        for buffer_case in 0..2 {
            let mut buffer =
                if buffer_case == 0 {
                    DataBlockBuffer::new(Version::V1,
                                         None,
                                         InputType::Block,
                                         OutputType::Block,
                                         BlockArrangement::Unordered,
                                         None,
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                } else {
                    DataBlockBuffer::new(Version::V17,
                                         None,
                                         InputType::Block,
                                         OutputType::Block,
                                         BlockArrangement::Unordered,
                                         Some((data, parity, burst)),
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                };

            let size = buffer.lots.len() * buffer.lot_size;

            let fill = std::cmp::min(size, fill);

            for _ in 0..fill {
                let _ = buffer.get_slot();
            }

            buffer.reset();

            assert_eq!(buffer.lots_used, 0);
        }
    }
}
