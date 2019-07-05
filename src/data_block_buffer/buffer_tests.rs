#![cfg(test)]
use super::*;
use crate::multihash::hash;
use crate::multihash::HashType;
use crate::sbx_specs::Version;
use proptest::prelude::*;
use reed_solomon_erasure::ReedSolomon;

use crate::file_writer::{FileWriter, FileWriterParam};
use crate::writer::{Writer, WriterType};

use crate::sbx_block;

use crate::rand_utils::fill_random_bytes;

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
                                                       fills in 1usize..1000) {
    }
}
