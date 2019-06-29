use rayon::prelude::*;
use reed_solomon_erasure::ReedSolomon;
use smallvec::SmallVec;
use std::io::SeekFrom;
use std::sync::Arc;

use crate::general_error::Error;
use crate::sbx_specs::{Version, SBX_LAST_SEQ_NUM};

use crate::sbx_block::{calc_data_block_write_pos, Block, BlockType};

use crate::sbx_block;

use crate::sbx_specs::{ver_to_block_size, ver_to_data_size, SBX_FILE_UID_LEN};

use crate::file_writer::FileWriter;
use crate::multihash::hash;

const DEFAULT_SINGLE_LOT_SIZE: usize = 10;

const LOT_COUNT_PER_CPU: usize = 10;

enum GetSlotResult<'a> {
    None,
    Some(&'a mut [u8], &'a mut Option<usize>),
    LastSlot(&'a mut [u8], &'a mut Option<usize>),
}

#[derive(Copy, Clone, Debug)]
pub enum InputType {
    Block,
    Data,
}

#[derive(Copy, Clone, Debug)]
pub enum InputMode {
    DataOnly,
    DataAndParity,
}

#[derive(Copy, Clone, Debug)]
pub enum OutputType {
    Block,
    Data,
    Disabled,
}

#[derive(Copy, Clone, Debug)]
pub enum OutputMode {
    DataOnly,
    DataAndParity,
    Disabled,
}

pub struct Slot<'a> {
    pub slot: &'a mut [u8],
    pub content_len_exc_header: &'a mut Option<usize>,
}

struct Lot {
    version: Version,
    block: Block,
    input_mode: InputMode,
    output_mode: OutputMode,
    data_par_burst: Option<(usize, usize, usize)>,
    meta_enabled: bool,
    block_size: usize,
    data_size: usize,
    lot_size: usize,
    data_block_count: usize,
    padding_block_count: usize,
    parity_block_count: usize,
    padding_byte_count_in_non_padding_blocks: usize,
    directly_writable_slots: usize,
    data: Vec<u8>,
    slot_write_pos: Vec<u64>,
    slot_content_len_exc_header: Vec<Option<usize>>,
    rs_codec: Arc<Option<ReedSolomon>>,
}

pub struct DataBlockBuffer {
    lots: Vec<Lot>,
    lot_size: usize,
    lots_used: usize,
    start_seq_num: Option<u32>,
    seq_num_incre: u32,
}

impl Lot {
    fn new(
        version: Version,
        uid: Option<&[u8; SBX_FILE_UID_LEN]>,
        input_mode: InputMode,
        output_mode: OutputMode,
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        lot_size: usize,
        rs_codec: &Arc<Option<ReedSolomon>>,
    ) -> Self {
        let block_size = ver_to_block_size(version);
        let data_size = ver_to_data_size(version);

        let rs_codec = Arc::clone(rs_codec);

        let directly_writable_slots = match input_mode {
            InputMode::Block => lot_size,
            InputMode::Data => match data_par_burst {
                None => lot_size,
                Some((data, _, _)) => data,
            }
        };

        let uid = match uid {
            None => &[0; SBX_FILE_UID_LEN],
            Some(x) => x,
        };

        Lot {
            version,
            block: Block::new(version, uid, BlockType::Data),
            input_mode,
            output_mode,
            data_par_burst,
            meta_enabled,
            block_size,
            data_size,
            lot_size,
            data_block_count: 0,
            padding_block_count: 0,
            parity_block_count: 0,
            padding_byte_count_in_non_padding_blocks: 0,
            directly_writable_slots,
            data: vec![0; block_size * lot_size],
            slot_write_pos: vec![0; lot_size],
            slot_content_len_exc_header: vec![None; lot_size],
            rs_codec,
        }
    }

    fn get_slot(&mut self) -> GetSlotResult {
        let slots_used = self.slots_used();

        if slots_used < self.directly_writable_slots {
            let start = slots_used * self.block_size;
            let end_exc = start + self.block_size;

            match self.data_par_burst {
                None => self.data_block_count += 1,
                Some((data, _, _)) => {
                    if slots_used < data {
                        self.data_block_count += 1;
                    } else {
                        self.parity_block_count += 1;
                    }
                }
            }

            let is_full = self.is_full();

            let slot = &mut self.data[start..end_exc];

            let slot = match self.input_mode {
                InputMode::Block => slot,
                InputMode::Data => sbx_block::slice_data_buf_mut(self.version, slot),
            };

            let content_len = &mut self.slot_content_len_exc_header[slots_used];

            if is_full {
                GetSlotResult::LastSlot(slot, content_len)
            } else {
                GetSlotResult::Some(slot, content_len)
            }
        } else {
            GetSlotResult::None
        }
    }

    fn cancel_last_slot(&mut self) {
        assert!(self.data_block_count > 0);

        self.data_block_count -= 1;
    }

    fn active(&self) -> bool {
        self.slots_used() > 0
    }

    fn fill_in_padding(&mut self) {
        for i in 0..self.data_block_count {
            if let Some(len) = self.slot_content_len_exc_header[i] {
                assert!(len > 0);
                assert!(len <= self.data_size);

                let start = i * self.block_size;
                let end_exc = start + self.block_size;
                let slot = &mut self.data[start..end_exc];

                if len < self.block_size {
                    self.padding_byte_count_in_non_padding_blocks += sbx_block::write_padding(self.version, len, slot);
                }
            }
        }

        if let Some((data, _, _)) = self.data_par_burst {
            for i in self.data_block_count..data {
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

    fn is_full(&self) -> bool {
        match self.input_mode {
            InputMode::Block => self.slots_used() == self.directly_writable_slots,
            InputMode::Data => match self.data_par_burst {
                None => self.data_block_count == self.directly_writable_slots,
            }self.data_block_count == 
        }
        self.data_block_count == self.directly_writable_slots
    }

    fn encode(&mut self, lot_start_seq_num: u32) {
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

                    self.slot_write_pos[slot_index] = write_pos;
                } else {
                    break;
                }
            }
        }
    }

    fn hash(&self, ctx: &mut hash::Ctx) {
        if self.active() {
            let slots_used = self.slots_used();

            let slots_to_hash = self.data_block_count;

            eprintln!("slots_to_hash : {}", slots_to_hash);
            eprintln!("lot_size : {}", self.lot_size);
            eprintln!("slots_used : {}", slots_used);
            eprintln!("directly_writable_slots : {}", self.directly_writable_slots);


            for (slot_index, slot) in self.data.chunks(self.block_size).enumerate() {
                if slot_index < slots_to_hash {
                    let content_len = match self.slot_content_len_exc_header[slot_index] {
                        None => self.data_size,
                        Some(len) => {
                            assert!(len > 0);
                            assert!(len <= self.data_size);
                            eprintln!("content_len : {}", len);
                            len
                        }
                    };

                    let data = &sbx_block::slice_data_buf(self.version, slot)[..content_len];

                    ctx.update(data);
                } else {
                    break;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.data_block_count = 0;
        self.padding_block_count = 0;
        self.parity_block_count = 0;
        for len in self.slot_content_len_exc_header.iter_mut() {
            *len = None;
        }
    }

    fn data_padding_parity_block_count(&self) -> (usize, usize, usize) {
        (
            self.data_block_count,
            self.padding_block_count,
            self.parity_block_count,
        )
    }

    fn write(&mut self, writer: &mut FileWriter) -> Result<(), Error> {
        if self.active() {
            let slots_used = self.slots_used();

            for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
                if slot_index < slots_used {
                    let write_pos = self.slot_write_pos[slot_index];

                    writer.seek(SeekFrom::Start(write_pos))?;

                    let slot = match self.output_mode {
                        OutputMode::Block => slot,
                        OutputMode::Data => sbx_block::slice_data_buf(self.version, slot),
                        OutputMode::Disabled => panic!("Output is diabled"),
                    };

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
        uid: Option<&[u8; SBX_FILE_UID_LEN]>,
        input_mode: InputMode,
        output_mode: OutputMode,
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        buffer_index: usize,
        total_buffer_count: usize,
    ) -> Self {
        let lot_count = num_cpus::get() * LOT_COUNT_PER_CPU;

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
                input_mode,
                output_mode,
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

    pub fn get_slot(&mut self) -> Option<Slot> {
        let lot_count = self.lots.len();

        if self.lots_used == lot_count {
            None
        } else {
            match self.lots[self.lots_used].get_slot() {
                GetSlotResult::LastSlot(slot, content_len_exc_header) => {
                    self.lots_used += 1;
                    Some(Slot { slot, content_len_exc_header })
                }
                GetSlotResult::Some(slot, content_len_exc_header) =>
                    Some(Slot { slot, content_len_exc_header}),
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

    pub fn reset(&mut self) {
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

    pub fn hash(&self, ctx: &mut hash::Ctx) {
        for lot in self.lots.iter() {
            lot.hash(ctx);
        }
    }

    pub fn write(&mut self, writer: &mut FileWriter) -> Result<(), Error> {
        for lot in self.lots.iter_mut() {
            lot.write(writer)?;
        }

        self.reset();

        Ok(())
    }
}
