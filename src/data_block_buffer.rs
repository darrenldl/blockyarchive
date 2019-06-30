use rayon::prelude::*;
use reed_solomon_erasure::ReedSolomon;
use smallvec::SmallVec;
use std::cmp::min;
use std::io::SeekFrom;
use std::sync::Arc;

use crate::general_error::Error;
use crate::sbx_specs::{Version, SBX_LAST_SEQ_NUM};

use crate::sbx_block::{calc_data_block_write_pos, calc_data_chunk_write_pos, Block, BlockType};

use crate::sbx_block;

use crate::sbx_specs::{ver_to_block_size, ver_to_data_size, SBX_FILE_UID_LEN};

use crate::writer::Writer;
use crate::multihash::hash;

use crate::misc_utils;

const DEFAULT_SINGLE_LOT_SIZE: usize = 10;

const LOT_COUNT_PER_CPU: usize = 10;

macro_rules! slice_slot_w_index {
    (
        $self:expr, $index:expr
    ) => {{
        let start = $index * $self.block_size;
        let end_exc = start + $self.block_size;

        &$self.data[start..end_exc]
    }};
    (
        mut => $self:expr, $index:expr
    ) => {{
        let start = $index * $self.block_size;
        let end_exc = start + $self.block_size;

        &mut $self.data[start..end_exc]
    }};
}

macro_rules! lot_is_full {
    (
        $self:expr
    ) => {{
        $self.slots_used == $self.directly_writable_slots
    }};
}

enum GetSlotResult<'a> {
    None,
    Some(&'a mut [u8], &'a mut Option<usize>),
    LastSlot(&'a mut [u8], &'a mut Option<usize>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputType {
    Block,
    Data,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BlockArrangement {
    Ordered,
    Unordered,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OutputType {
    Block,
    Data,
    Disabled,
}

pub struct Slot<'a> {
    pub slot: &'a mut [u8],
    pub content_len_exc_header: &'a mut Option<usize>,
}

struct Lot {
    version: Version,
    input_type: InputType,
    output_type: OutputType,
    arrangement: BlockArrangement,
    data_par_burst: Option<(usize, usize, usize)>,
    meta_enabled: bool,
    block_size: usize,
    data_size: usize,
    lot_size: usize,
    slots_used: usize,
    padding_byte_count_in_non_padding_blocks: usize,
    directly_writable_slots: usize,
    blocks: Vec<Block>,
    data: Vec<u8>,
    check_block: Block,
    check_buffer: Vec<u8>,
    slot_write_pos: Vec<Option<u64>>,
    slot_content_len_exc_header: Vec<Option<usize>>,
    slot_is_padding: Vec<bool>,
    skip_good: bool,
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
        input_type: InputType,
        output_type: OutputType,
        arrangement: BlockArrangement,
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        lot_size: usize,
        skip_good: bool,
        rs_codec: &Arc<Option<ReedSolomon>>,
    ) -> Self {
        let block_size = ver_to_block_size(version);
        let data_size = ver_to_data_size(version);

        let rs_codec = Arc::clone(rs_codec);

        let directly_writable_slots = match input_type {
            InputType::Block => lot_size,
            InputType::Data => match data_par_burst {
                None => lot_size,
                Some((data, _, _)) => data,
            },
        };

        let uid = match uid {
            None => &[0; SBX_FILE_UID_LEN],
            Some(x) => x,
        };

        let mut blocks = Vec::with_capacity(lot_size);
        for _ in 0..lot_size {
            blocks.push(Block::new(version, uid, BlockType::Data));
        }

        Lot {
            version,
            input_type,
            output_type,
            arrangement,
            data_par_burst,
            meta_enabled,
            block_size,
            data_size,
            lot_size,
            slots_used: 0,
            padding_byte_count_in_non_padding_blocks: 0,
            directly_writable_slots,
            blocks,
            data: vec![0; block_size * lot_size],
            check_block: Block::dummy(),
            check_buffer: vec![0; block_size],
            slot_write_pos: vec![None; lot_size],
            slot_content_len_exc_header: vec![None; lot_size],
            slot_is_padding: vec![false; directly_writable_slots],
            skip_good,
            rs_codec,
        }
    }

    fn get_slot(&mut self) -> GetSlotResult {
        if self.slots_used < self.directly_writable_slots {
            let slot = slice_slot_w_index!(mut => self, self.slots_used);

            let slot = match self.input_type {
                InputType::Block => slot,
                InputType::Data => sbx_block::slice_data_buf_mut(self.version, slot),
            };

            let content_len = &mut self.slot_content_len_exc_header[self.slots_used];

            self.slots_used += 1;

            if lot_is_full!(self) {
                GetSlotResult::LastSlot(slot, content_len)
            } else {
                GetSlotResult::Some(slot, content_len)
            }
        } else {
            GetSlotResult::None
        }
    }

    fn cancel_last_slot(&mut self) {
        assert!(self.slots_used > 0);
        assert!(self.slots_used <= self.directly_writable_slots);

        self.slots_used -= 1;
    }

    fn fill_in_padding(&mut self) {
        if self.active() {
            for i in 0..self.slots_used {
                if let Some(len) = self.slot_content_len_exc_header[i] {
                    assert!(len > 0);
                    assert!(len <= self.data_size);

                    let slot = slice_slot_w_index!(mut => self, i);

                    if len < self.data_size {
                        self.padding_byte_count_in_non_padding_blocks +=
                            sbx_block::write_padding(self.version, len, slot);
                    }
                }
            }

            if let Some((data, _, _)) = self.data_par_burst {
                assert!(self.arrangement == BlockArrangement::Ordered);
                assert!(self.slots_used <= data);

                for i in self.slots_used..data {
                    let slot = slice_slot_w_index!(mut => self, i);

                    sbx_block::write_padding(self.version, 0, slot);

                    self.slot_is_padding[i] = true;
                }

                self.slots_used = data;
            }
        }
    }

    fn rs_encode(&mut self) {
        if self.active() {
            if let Some(ref rs_codec) = *self.rs_codec {
                assert!(self.slots_used == rs_codec.data_shard_count());

                let mut refs: SmallVec<[&mut [u8]; 32]> = SmallVec::with_capacity(self.lot_size);

                // collect references to data segments
                for slot in self.data.chunks_mut(self.block_size) {
                    refs.push(sbx_block::slice_data_buf_mut(self.version, slot));
                }

                rs_codec.encode(&mut refs).unwrap();

                self.slots_used = self.lot_size;
            }
        }
    }

    fn is_full(&self) -> bool {
        self.slots_used >= self.directly_writable_slots
    }

    fn active(&self) -> bool {
        self.slots_used > 0
    }

    fn sync_blocks_from_slots(&mut self) {
        for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
            if slot_index < self.slots_used {
                self.blocks[slot_index]
                    .sync_from_buffer(slot, None, None)
                    .unwrap();
            } else {
                break;
            }
        }
    }

    fn calc_slot_write_pos(&mut self) {
        for slot_index in 0..self.slots_used {
            let write_pos = match self.output_type {
                OutputType::Block => Some(calc_data_block_write_pos(
                    self.version,
                    self.blocks[slot_index].get_seq_num(),
                    Some(self.meta_enabled),
                    self.data_par_burst,
                )),
                OutputType::Data => {
                    let data_par = match self.data_par_burst {
                        None => None,
                        Some((data, par, _)) => Some((data, par)),
                    };

                    calc_data_chunk_write_pos(
                        self.version,
                        self.blocks[slot_index].get_seq_num(),
                        data_par,
                    )
                }
                OutputType::Disabled => panic!("Output is disabled"),
            };

            self.slot_write_pos[slot_index] = write_pos;
        }
    }

    fn set_block_seq_num_based_on_lot_start_seq_num(&mut self, lot_start_seq_num: u32) {
        for slot_index in 0..self.slots_used {
            if slot_index < self.slots_used {
                let tentative_seq_num = lot_start_seq_num as u64 + slot_index as u64;

                assert!(tentative_seq_num <= SBX_LAST_SEQ_NUM as u64);

                self.blocks[slot_index].set_seq_num(lot_start_seq_num + slot_index as u32);
            } else {
                break;
            }
        }
    }

    fn sync_blocks_to_slots(&mut self) {
        for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
            if slot_index < self.slots_used {
                self.blocks[slot_index].sync_to_buffer(None, slot).unwrap();
            } else {
                break;
            }
        }
    }

    fn encode(&mut self, lot_start_seq_num: u32) {
        assert!(self.input_type == InputType::Data);
        assert!(self.arrangement == BlockArrangement::Ordered);

        self.fill_in_padding();

        self.rs_encode();

        self.set_block_seq_num_based_on_lot_start_seq_num(lot_start_seq_num);

        self.sync_blocks_to_slots();
    }

    fn hash(&self, ctx: &mut hash::Ctx) {
        assert!(self.arrangement == BlockArrangement::Ordered);

        let slots_to_hash = match self.data_par_burst {
            None => self.slots_used,
            Some((data, _, _)) => min(data, self.slots_used),
        };

        for (slot_index, slot) in self.data.chunks(self.block_size).enumerate() {
            if slot_index < slots_to_hash {
                let content_len = match self.slot_content_len_exc_header[slot_index] {
                    None => self.data_size,
                    Some(len) => {
                        assert!(len > 0);
                        assert!(len <= self.data_size);
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

    fn reset(&mut self) {
        self.slots_used = 0;

        self.padding_byte_count_in_non_padding_blocks = 0;

        for len in self.slot_content_len_exc_header.iter_mut() {
            *len = None;
        }

        for is_padding in self.slot_is_padding.iter_mut() {
            *is_padding = false;
        }
    }

    fn data_padding_parity_block_count(&self) -> (usize, usize, usize) {
        let data = match self.data_par_burst {
            None => self.slots_used,
            Some((data, _, _)) => min(data, self.slots_used),
        };

        let mut padding = 0;
        for &is_padding in self.slot_is_padding.iter() {
            if is_padding {
                padding += 1;
            }
        }

        let parity = match self.data_par_burst {
            None => 0,
            Some((data, _, _)) => {
                if self.slots_used < data {
                    0
                } else {
                    self.slots_used - data
                }
            }
        };

        (data, padding, parity)
    }

    fn padding_byte_count_in_non_padding_blocks(&self) -> usize {
        self.padding_byte_count_in_non_padding_blocks
    }

    fn write(&mut self, seek: bool, writer: &mut Writer) -> Result<(), Error> {
        self.calc_slot_write_pos();

        for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
            if slot_index < self.slots_used {
                if let Some(write_pos) = self.slot_write_pos[slot_index] {
                    if seek {
                        writer.seek(SeekFrom::Start(write_pos)).unwrap()?;
                    }

                    if self.skip_good {
                        let cur_pos = writer.cur_pos().unwrap()?;

                        let check_buffer = match self.output_type {
                            OutputType::Block => &mut self.check_buffer,
                            OutputType::Data => &mut self.check_buffer[..self.data_size],
                            OutputType::Disabled => panic!("Output is disabled")
                        };

                        let read_res = writer.read(check_buffer).unwrap()?;
                        writer.seek(SeekFrom::Start(cur_pos)).unwrap()?;

                        let skip = match self.output_type {
                            OutputType::Block => {
                                let block = &self.blocks[slot_index];

                                read_res.eof_seen || {
                                    match self.check_block.sync_from_buffer(
                                        &check_buffer,
                                        None,
                                        None,
                                    ) {
                                        Ok(()) => self.check_block.get_version() == block.get_version()
                                            && self.check_block.get_uid() == block.get_uid()
                                            && self.check_block.get_seq_num() == block.get_seq_num(),
                                        Err(_) => false,
                                    }
                                }
                            }
                            OutputType::Data => {
                                read_res.eof_seen ||
                                    misc_utils::buffer_is_blank(check_buffer)
                            }
                            OutputType::Disabled => panic!("Output is disabled")
                        };

                        if skip { continue; }
                    }

                    let slot = match self.output_type {
                        OutputType::Block => slot,
                        OutputType::Data => sbx_block::slice_data_buf(self.version, slot),
                        OutputType::Disabled => panic!("Output is diabled"),
                    };

                    writer.write(slot)?;
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl DataBlockBuffer {
    pub fn new(
        version: Version,
        uid: Option<&[u8; SBX_FILE_UID_LEN]>,
        input_type: InputType,
        skip_good: bool,
        output_type: OutputType,
        arrangement: BlockArrangement,
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
                input_type,
                output_type,
                arrangement,
                data_par_burst,
                meta_enabled,
                lot_size,
                skip_good,
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
        self.lots_used > 0 || self.lots[self.lots_used].slots_used > 0
    }

    pub fn get_slot(&mut self) -> Option<Slot> {
        let lot_count = self.lots.len();

        if self.lots_used == lot_count {
            None
        } else {
            match self.lots[self.lots_used].get_slot() {
                GetSlotResult::LastSlot(slot, content_len_exc_header) => {
                    self.lots_used += 1;
                    Some(Slot {
                        slot,
                        content_len_exc_header,
                    })
                }
                GetSlotResult::Some(slot, content_len_exc_header) => Some(Slot {
                    slot,
                    content_len_exc_header,
                }),
                GetSlotResult::None => {
                    self.lots_used += 1;
                    None
                }
            }
        }
    }

    pub fn cancel_last_slot(&mut self) {
        assert!(self.active());

        let shift_back_one_slot = self.is_full() || self.lots[self.lots_used].slots_used == 0;

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

    pub fn padding_byte_count_in_non_padding_blocks(&self) -> usize {
        let mut byte_count = 0;

        for lot in self.lots.iter() {
            byte_count += lot.padding_byte_count_in_non_padding_blocks();
        }

        byte_count
    }

    pub fn hash(&self, ctx: &mut hash::Ctx) {
        for lot in self.lots.iter() {
            lot.hash(ctx);
        }
    }

    pub fn sync_blocks_from_slots(&mut self) {
        for lot in self.lots.iter_mut() {
            lot.sync_blocks_from_slots();
        }
    }

    fn write_internal(&mut self, seek: bool, writer: &mut Writer) -> Result<(), Error> {
        for lot in self.lots.iter_mut() {
            lot.write(seek, writer)?;
        }

        self.reset();

        Ok(())
    }

    pub fn write(&mut self, writer: &mut Writer) -> Result<(), Error> {
        self.write_internal(true, writer)
    }

    pub fn write_no_seek(&mut self, writer: &mut Writer) -> Result<(), Error> {
        self.write_internal(false, writer)
    }
}
