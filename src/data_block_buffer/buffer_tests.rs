#![cfg(test)]
use super::*;
use crate::sbx_specs::Version;

quickcheck! {
    fn qc_new_buffer_stats(buffer_index: usize,
                           total_buffer_count: usize,
                           data: usize,
                           parity: usize,
                           burst: usize) -> bool {
        let buffer_index = 1 + buffer_index % 1000;
        let total_buffer_count = 1 + total_buffer_count % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;

        for buffer_case in 0..2 {
            let buffer =
                if buffer_case == 0 {
                    DataBlockBuffer::new(Version::V1,
                                         None,
                                         InputType::Data,
                                         OutputType::Block,
                                         None,
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                } else {
                    DataBlockBuffer::new(Version::V17,
                                         None,
                                         InputType::Data,
                                         OutputType::Block,
                                         Some((data, parity, burst)),
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                };

            let lot_count = num_cpus::get() * LOT_COUNT_PER_CPU;

            let lot_size =
                if buffer_case == 0 {
                    DEFAULT_SINGLE_LOT_SIZE
                } else {
                    data + parity
                };

            let res =
                buffer.lots.len() == lot_count
                && buffer.lot_size == lot_size
                && buffer.lots_used == 0
                && buffer.start_seq_num == Some(1 + (buffer_index * buffer.total_slot_count()) as u32)
                && buffer.seq_num_incre == (buffer.total_slot_count() * total_buffer_count) as u32
                && buffer.lot_count() == lot_count
                && buffer.total_slot_count() == lot_count * lot_size;

            if !res { return false; }
        }

        true
    }

    #[should_panic]
    fn qc_cancel_slot_panics_when_empty(buffer_index: usize,
                                        total_buffer_count: usize,
                                        cancels: usize) -> bool {
        let buffer_index = 1 + buffer_index % 1000;
        let total_buffer_count = 1 + total_buffer_count % 1000;
        let cancels = 1 + cancels % 1000;

        let mut buffer = DataBlockBuffer::new(Version::V1,
                                              None,
                                              InputType::Block(BlockArrangement::Unordered),
                                              OutputType::Block,
                                              None,
                                              true,
                                              false,
                                              buffer_index,
                                              total_buffer_count,
        );

        let size = buffer.total_slot_count();

        let cancels = std::cmp::min(size, cancels);

        for _ in 0..cancels {
            let _ = buffer.get_slot();
        }

        for _ in 0..cancels+1 {
            buffer.cancel_slot();
        }

        true
    }

    fn qc_cancel_slot_when_not_empty(buffer_index: usize,
                                     total_buffer_count: usize,
                                     cancels: usize) -> bool {
        let buffer_index = 1 + buffer_index % 1000;
        let total_buffer_count = 1 + total_buffer_count % 1000;
        let cancels = 1 + cancels % 1000;

        let mut buffer = DataBlockBuffer::new(Version::V1,
                                              None,
                                              InputType::Block(BlockArrangement::Unordered),
                                              OutputType::Block,
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

        true
    }

    fn qc_active_if_and_only_if_at_least_one_slot_in_use(buffer_index: usize,
                                                         total_buffer_count: usize,
                                                         data: usize,
                                                         parity: usize,
                                                         burst: usize,
                                                         fill: usize,
                                                         tries: usize) -> bool {
        let buffer_index = 1 + buffer_index % 1000;
        let total_buffer_count = 1 + total_buffer_count % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let fill = 1 + fill % 1000;
        let tries = 2 + tries % 100;

        for buffer_case in 0..2 {
            let mut buffer =
                if buffer_case == 0 {
                    DataBlockBuffer::new(Version::V1,
                                         None,
                                         InputType::Block(BlockArrangement::Unordered),
                                         OutputType::Block,
                                         None,
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                } else {
                    DataBlockBuffer::new(Version::V17,
                                         None,
                                         InputType::Block(BlockArrangement::Unordered),
                                         OutputType::Block,
                                         Some((data, parity, burst)),
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                };

            let size = buffer.lots.len() * buffer.lot_size;

            let fill = std::cmp::min(size, fill);

            for _ in 0..tries {
                let mut res = true;

                res = res && !buffer.active();

                for _ in 0..fill {
                    let _ = buffer.get_slot();

                    res = res && buffer.active();
                }

                for _ in 0..fill {
                    res = res && buffer.active();

                    buffer.cancel_slot();
                }

                res = res && !buffer.active();

                if !res { return false; }
            }
        }

        true
    }

    fn qc_stats_are_reset_correctly_after_buffer_reset(buffer_index: usize,
                                                       total_buffer_count: usize,
                                                       data: usize,
                                                       parity: usize,
                                                       burst: usize,
                                                       fill: usize) -> bool {
        let buffer_index = 1 + buffer_index % 1000;
        let total_buffer_count = 1 + total_buffer_count % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let fill = 1 + fill % 1000;

        for buffer_case in 0..2 {
            let mut buffer =
                if buffer_case == 0 {
                    DataBlockBuffer::new(Version::V1,
                                         None,
                                         InputType::Block(BlockArrangement::Unordered),
                                         OutputType::Block,
                                         None,
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                } else {
                    DataBlockBuffer::new(Version::V17,
                                         None,
                                         InputType::Block(BlockArrangement::Unordered),
                                         OutputType::Block,
                                         Some((data, parity, burst)),
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                };

            let size = buffer.total_slot_count();

            let fill = std::cmp::min(size, fill);

            for _ in 0..fill {
                let _ = buffer.get_slot();
            }

            buffer.reset();

            let res = buffer.lots_used == 0;

            if !res { return false; }
        }

        true
    }

    fn qc_is_full_is_correct(
        buffer_index: usize,
        total_buffer_count: usize,
        data: usize,
        parity: usize,
        burst: usize,
        tries: usize
    ) -> bool {
        let buffer_index = 1 + buffer_index % 1000;
        let total_buffer_count = 1 + total_buffer_count % 1000;
        let data = 1 + data % 30;
        let parity = 1 + parity % 30;
        let burst = 1 + burst % 100;
        let tries = 2 + tries % 10;

        for buffer_case in 0..2 {
            let mut buffer =
                if buffer_case == 0 {
                    DataBlockBuffer::new(Version::V1,
                                         None,
                                         InputType::Block(BlockArrangement::Unordered),
                                         OutputType::Block,
                                         None,
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                } else {
                    DataBlockBuffer::new(Version::V17,
                                         None,
                                         InputType::Block(BlockArrangement::Unordered),
                                         OutputType::Block,
                                         Some((data, parity, burst)),
                                         true,
                                         false,
                                         buffer_index,
                                         total_buffer_count,
                    )
                };

            let size = buffer.total_slot_count();

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
        }

        true
    }
}
