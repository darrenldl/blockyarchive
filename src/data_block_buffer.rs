use reed_solomon_erasure::ReedSolomon;
use smallvec::SmallVec;
use rayon::prelude::*;
use std::sync::Arc;
use std::io::SeekFrom;

use crate::sbx_specs::{Version, SBX_LAST_SEQ_NUM};
use crate::general_error::Error;

use crate::sbx_block::{
    calc_data_block_write_pos, Block, BlockType,
};

use crate::sbx_block;

use crate::sbx_specs::{
    ver_to_block_size,
    SBX_FILE_UID_LEN,
};

use crate::file_writer::{FileWriter};
use crate::multihash::hash;

const DEFAULT_SINGLE_LOT_SIZE: usize = 10;

struct Lot {
    version: Version,
    block: Block,
    data_par_burst: Option<(usize, usize, usize)>,
    meta_enabled: bool,
    block_size: usize,
    lot_size: usize,
    data_block_count: usize,
    padding_block_count: usize,
    parity_block_count: usize,
    directly_writable_slots: usize,
    data: Vec<u8>,
    write_pos_s: Vec<u64>,
    rs_codec: Arc<Option<ReedSolomon>>,
}

enum GetSlotResult<'a> {
    None,
    Some(&'a mut [u8]),
    LastSlot(&'a mut [u8]),
}

pub struct DataBlockBuffer {
    lots: Vec<Lot>,
    lot_size: usize,
    lots_used: usize,
    start_seq_num: Option<u32>,
    seq_num_incre: u32,
}

impl Lot {
    pub fn new(
        version: Version,
        uid: &[u8; SBX_FILE_UID_LEN],
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        lot_size: usize,
        rs_codec: &Arc<Option<ReedSolomon>>,
    ) -> Self {
        let block_size = ver_to_block_size(version);

        let rs_codec = Arc::clone(rs_codec);

        let directly_writable_slots = match data_par_burst {
            None => lot_size,
            Some((data, _, _)) => data,
        };

        Lot {
            version,
            block: Block::new(version, uid, BlockType::Data),
            data_par_burst,
            meta_enabled,
            block_size,
            lot_size,
            data_block_count: 0,
            padding_block_count: 0,
            parity_block_count: 0,
            directly_writable_slots,
            data: vec![0; block_size * lot_size],
            write_pos_s: vec![0; lot_size],
            rs_codec,
        }
    }

    pub fn get_slot(&mut self) -> GetSlotResult {
        if self.slots_used() < self.directly_writable_slots {
            let start = self.slots_used() * self.block_size;
            let end_exc = start + self.block_size;
            self.data_block_count += 1;
            if self.is_full() {
                GetSlotResult::LastSlot(&mut self.data[start..end_exc])
            } else {
                GetSlotResult::Some(&mut self.data[start..end_exc])
            }
        } else {
            GetSlotResult::None
        }
    }

    pub fn cancel_last_slot(&mut self) {
        assert!(self.data_block_count > 0);

        self.data_block_count -= 1;
    }

    pub fn active(&self) -> bool {
        self.slots_used() > 0
    }

    fn fill_in_padding(&mut self) {
        if let Some(_) = self.data_par_burst {
            for i in self.data_block_count..self.directly_writable_slots {
                let start = i * self.block_size;
                let end_exc = start + self.block_size;
                let slot = &mut self.data[start..end_exc];

                sbx_block::write_padding(self.version, 0, slot);

                self.padding_block_count += 1;
            }
        }
    }

    fn rs_encode(&mut self) {
        if let Some(ref rs_codec) = *self.rs_codec {
            let mut refs: SmallVec<[&mut [u8]; 32]> = SmallVec::with_capacity(self.lot_size);

            // collect references to data segments
            for slot in self.data.chunks_mut(self.block_size) {
                refs.push(slot);
            }

            rs_codec.encode(&mut refs).unwrap();

            self.parity_block_count = rs_codec.parity_shard_count();
        }
    }

    fn slots_used(&self) -> usize {
        self.data_block_count + self.padding_block_count + self.parity_block_count
    }

    pub fn is_full(&self) -> bool {
        self.data_block_count == self.directly_writable_slots
    }

    pub fn encode(&mut self, lot_start_seq_num: u32) {
        if self.active() {
            self.fill_in_padding();

            self.rs_encode();

            let slots_used = self.slots_used();

            for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
                if slot_index < slots_used {
                    let tentative_seq_num = lot_start_seq_num as u64 + slot_index as u64;

                    assert!(tentative_seq_num <= SBX_LAST_SEQ_NUM as u64);

                    self.block
                        .set_seq_num(lot_start_seq_num + slot_index as u32);

                    self.block.sync_to_buffer(None, slot).unwrap();

                    let write_pos = calc_data_block_write_pos(
                        self.version,
                        self.block.get_seq_num(),
                        Some(self.meta_enabled),
                        self.data_par_burst,
                    );

                    self.write_pos_s[slot_index] = write_pos;
                } else {
                    break;
                }
            }
        }
    }

    fn hash(&self, hash_ctx: &mut hash::Ctx) {
        let slots_used = self.slots_used();

        for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
            if slot_index < slots_used {
                let data = sbx_block::slice_data_buf(self.version, slot);

                hash_ctx.update(data);
            }
        }
    }

    fn reset(&mut self) {
        self.data_block_count = 0;
        self.padding_block_count = 0;
        self.parity_block_count = 0;
    }

    pub fn data_padding_parity_block_count(&self) -> (usize, usize, usize) {
        (
            self.data_block_count,
            self.padding_block_count,
            self.parity_block_count,
        )
    }

    pub fn write(&mut self, writer: &mut FileWriter) -> Result<(), Error> {
        if self.active() {
            let slots_used = self.slots_used();

            for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
                if slot_index < slots_used {
                    let write_pos = self.write_pos_s[slot_index];

                    writer.seek(SeekFrom::Start(write_pos))?;

                    writer.write(slot)?;
                } else {
                    break;
                }
            }

            self.reset();
        }

        Ok(())
    }
}

impl DataBlockBuffer {
    pub fn new(
        version: Version,
        uid: &[u8; SBX_FILE_UID_LEN],
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        lot_count: usize,
        buffer_index: usize,
        total_buffer_count: usize,
    ) -> Self {
        assert!(lot_count > 0);

        let mut lots = Vec::with_capacity(lot_count);

        let lot_size = match data_par_burst {
            None => DEFAULT_SINGLE_LOT_SIZE,
            Some((data, parity, _)) => data + parity,
        };

        let rs_codec = Arc::new(match data_par_burst {
            None => None,
            Some((data, parity, _)) => Some(ReedSolomon::new(data, parity).unwrap()),
        });

        for _ in 0..lot_count {
            lots.push(Lot::new(
                version,
                uid,
                data_par_burst,
                meta_enabled,
                lot_size,
                &rs_codec,
            ))
        }

        let total_slot_count_per_buffer = lot_count * lot_size;

        let seq_num_incre = (total_slot_count_per_buffer * total_buffer_count) as u32;

        let start_seq_num = 1 + (buffer_index * total_slot_count_per_buffer) as u32;

        DataBlockBuffer {
            lots,
            lot_size,
            lots_used: 0,
            start_seq_num: Some(start_seq_num),
            seq_num_incre,
        }
    }

    pub fn lot_count(&self) -> usize {
        self.lots.len()
    }

    pub fn is_full(&self) -> bool {
        self.lots_used == self.lot_count()
    }

    pub fn active(&self) -> bool {
        self.lots_used > 0 || self.lots[self.lots_used].slots_used() > 0
    }

    pub fn get_slot(&mut self) -> Option<&mut [u8]> {
        let lot_count = self.lots.len();

        if self.lots_used == lot_count {
            None
        } else {
            match self.lots[self.lots_used].get_slot() {
                GetSlotResult::LastSlot(slot) => {
                    self.lots_used += 1;
                    Some(slot)
                }
                GetSlotResult::Some(slot) => Some(slot),
                GetSlotResult::None => {
                    self.lots_used += 1;
                    None
                }
            }
        }
    }

    pub fn cancel_last_slot(&mut self) {
        assert!(self.active());

        let shift_back_one_slot = self.is_full() || self.lots[self.lots_used].slots_used() == 0;

        if shift_back_one_slot {
            self.lots_used -= 1;
        }

        self.lots[self.lots_used].cancel_last_slot();
    }

    pub fn encode(&mut self) -> Result<(), Error> {
        let start_seq_num = self.start_seq_num.unwrap();
        let lot_size = self.lot_size;

        self.lots
            .par_iter_mut()
            .enumerate()
            .for_each(|(lot_index, lot)| {
                let lot_start_seq_num = start_seq_num + (lot_index * lot_size) as u32;

                lot.encode(lot_start_seq_num);
            });

        let seq_num_incre_will_overflow =
            std::u32::MAX - self.seq_num_incre < self.start_seq_num.unwrap();

        if self.is_full() && !seq_num_incre_will_overflow {
            self.start_seq_num = Some(start_seq_num + self.seq_num_incre);
        } else {
            self.start_seq_num = None;
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.lots_used = 0;

        for lot in self.lots.iter_mut() {
            lot.reset();
        }
    }

    pub fn data_padding_parity_block_count(&self) -> (usize, usize, usize) {
        let mut data_blocks = 0;
        let mut padding_blocks = 0;
        let mut parity_blocks = 0;

        for lot in self.lots.iter() {
            let (data, padding, parity) = lot.data_padding_parity_block_count();

            data_blocks += data;
            padding_blocks += padding;
            parity_blocks += parity;
        }

        (data_blocks, padding_blocks, parity_blocks)
    }

    pub fn write(&mut self, writer: &mut FileWriter) -> Result<(), Error> {
        for lot in self.lots.iter_mut() {
            lot.write(writer)?;
        }

        self.reset();

        Ok(())
    }
}
