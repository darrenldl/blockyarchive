use rayon::prelude::*;
use reed_solomon_erasure::ReedSolomon;
use smallvec::SmallVec;
use std::io::SeekFrom;
use std::sync::Arc;

use crate::general_error::Error;
use crate::sbx_specs::{
    ver_uses_rs, Version, SBX_FIRST_DATA_SEQ_NUM, SBX_LARGEST_BLOCK_SIZE, SBX_LAST_SEQ_NUM,
};

use crate::sbx_block::{calc_data_block_write_pos, calc_data_chunk_write_pos, Block, BlockType};

use crate::sbx_block;

use crate::sbx_specs::{ver_to_block_size, ver_to_data_size, SBX_FILE_UID_LEN};

use crate::multihash::hash;
use crate::writer::Writer;

use crate::misc_utils;

mod buffer_tests;
mod lot_tests;

const DEFAULT_SINGLE_LOT_SIZE: usize = 100;

const LOT_COUNT_PER_CPU: usize = 50;

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

macro_rules! check_data_par_burst_consistent_with_version {
    (
        $data_par_burst:expr, $version:expr
    ) => {{
        match $data_par_burst {
            None => assert!(!ver_uses_rs($version)),
            Some(_) => assert!(ver_uses_rs($version)),
        }
    }};
}

macro_rules! check_data_par_burst_consistent_with_rs_codec {
    (
        $data_par_burst:expr, $rs_codec:expr
    ) => {{
        match $data_par_burst {
            None => match **$rs_codec {
                None => {}
                Some(_) => panic!(),
            },
            Some((data, par, _)) => match **$rs_codec {
                None => panic!(),
                Some(ref rs_codec) => {
                    assert!(data == rs_codec.data_shard_count());
                    assert!(par == rs_codec.parity_shard_count());
                }
            },
        }
    }};
}

enum GetSlotResult<'a> {
    None,
    Some(
        &'a mut Block,
        &'a mut [u8],
        &'a mut Option<u64>,
        &'a mut Option<usize>,
    ),
    LastSlot(
        &'a mut Block,
        &'a mut [u8],
        &'a mut Option<u64>,
        &'a mut Option<usize>,
    ),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputType {
    Data,
    Block(BlockArrangement),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BlockArrangement {
    OrderedAndNoMissing,
    OrderedButSomeMayBeMissing,
    Unordered,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OutputType {
    Block,
    Data,
    Disabled,
}

pub struct Slot<'a> {
    pub block: &'a mut Block,
    pub slot: &'a mut [u8],
    pub read_pos: &'a mut Option<u64>,
    pub content_len_exc_header: &'a mut Option<usize>,
}

pub struct SlotView<'a> {
    pub block: &'a Block,
    pub slot: &'a [u8],
    pub read_pos: &'a Option<u64>,
    pub write_pos: &'a Option<u64>,
    pub content_len_exc_header: &'a Option<usize>,
}

struct Lot {
    version: Version,
    uid: [u8; SBX_FILE_UID_LEN],
    input_type: InputType,
    output_type: OutputType,
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
    slot_read_pos: Vec<Option<u64>>,
    slot_write_pos_usable: bool,
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
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        skip_good: bool,
        default_lot_size: usize,
        rs_codec: &Arc<Option<ReedSolomon>>,
    ) -> Self {
        assert!(default_lot_size > 0);

        check_data_par_burst_consistent_with_version!(data_par_burst, version);

        check_data_par_burst_consistent_with_rs_codec!(data_par_burst, rs_codec);

        let (lot_size, directly_writable_slots) = match input_type {
            InputType::Data => match data_par_burst {
                None => (default_lot_size, default_lot_size),
                Some((data, parity, _)) => (data + parity, data),
            },
            InputType::Block(_) => (default_lot_size, default_lot_size),
        };

        let block_size = ver_to_block_size(version);
        let data_size = ver_to_data_size(version);

        let rs_codec = Arc::clone(rs_codec);

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
            uid: *uid,
            input_type,
            output_type,
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
            check_buffer: vec![0; SBX_LARGEST_BLOCK_SIZE],
            slot_read_pos: vec![None; lot_size],
            slot_write_pos_usable: false,
            slot_write_pos: vec![None; lot_size],
            slot_content_len_exc_header: vec![None; lot_size],
            slot_is_padding: vec![false; lot_size],
            skip_good,
            rs_codec,
        }
    }

    fn get_slot(&mut self) -> GetSlotResult {
        if self.slots_used < self.directly_writable_slots {
            let block = &mut self.blocks[self.slots_used];

            let slot = slice_slot_w_index!(mut => self, self.slots_used);

            let slot = match self.input_type {
                InputType::Data => sbx_block::slice_data_buf_mut(self.version, slot),
                InputType::Block(_) => slot,
            };

            let read_pos = &mut self.slot_read_pos[self.slots_used];

            let content_len = &mut self.slot_content_len_exc_header[self.slots_used];

            self.slots_used += 1;

            if lot_is_full!(self) {
                GetSlotResult::LastSlot(block, slot, read_pos, content_len)
            } else {
                GetSlotResult::Some(block, slot, read_pos, content_len)
            }
        } else {
            GetSlotResult::None
        }
    }

    fn view_slots(&self) -> Vec<SlotView> {
        let mut res = Vec::with_capacity(self.slots_used);

        for i in 0..self.slots_used {
            res.push(SlotView {
                block: &self.blocks[i],
                slot: slice_slot_w_index!(self, i),
                read_pos: &self.slot_read_pos[i],
                write_pos: &self.slot_write_pos[i],
                content_len_exc_header: &self.slot_content_len_exc_header[i],
            });
        }

        res
    }

    fn cancel_slot(&mut self) {
        assert!(self.slots_used > 0);
        assert!(self.slots_used <= self.directly_writable_slots);

        self.slots_used -= 1;

        self.reset_slot(self.slots_used);
    }

    fn active(&self) -> bool {
        self.slots_used > 0
    }

    fn calc_slot_write_pos(&mut self) {
        assert!(self.output_type != OutputType::Disabled);

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
                OutputType::Disabled => panic!(),
            };

            self.slot_write_pos[slot_index] = write_pos;
        }

        self.slot_write_pos_usable = true;
    }

    fn fill_in_padding(&mut self) {
        assert!(self.input_type == InputType::Data);

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
        assert!(self.input_type == InputType::Data);

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

    fn set_block_seq_num_based_on_lot_start_seq_num(&mut self, lot_start_seq_num: u32) {
        assert!(self.input_type == InputType::Data);

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

        self.fill_in_padding();

        self.rs_encode();

        self.set_block_seq_num_based_on_lot_start_seq_num(lot_start_seq_num);

        self.sync_blocks_to_slots();
    }

    fn hash(&self, ctx: &mut hash::Ctx) {
        match self.input_type {
            InputType::Data => {}
            InputType::Block(arrangement) => assert!(
                arrangement == BlockArrangement::OrderedAndNoMissing
                    || arrangement == BlockArrangement::OrderedButSomeMayBeMissing
            ),
        }

        for (slot_index, slot) in self.data.chunks(self.block_size).enumerate() {
            if slot_index < self.slots_used {
                let block = &self.blocks[slot_index];

                if block.is_data()
                    && !self.slot_is_padding[slot_index]
                    && !block.is_parity_w_data_par_burst(self.data_par_burst)
                {
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
                }
            } else {
                break;
            }
        }
    }

    fn reset_slot(&mut self, slot_index: usize) {
        let block = &mut self.blocks[slot_index];
        block.set_version(self.version);
        block.set_uid(self.uid);
        block.set_seq_num(SBX_FIRST_DATA_SEQ_NUM);

        self.slot_read_pos[slot_index] = None;
        self.slot_write_pos[slot_index] = None;
        self.slot_content_len_exc_header[slot_index] = None;
        self.slot_is_padding[slot_index] = false;
    }

    fn reset(&mut self) {
        self.slots_used = 0;

        self.padding_byte_count_in_non_padding_blocks = 0;

        for block in self.blocks.iter_mut() {
            block.set_version(self.version);
            block.set_uid(self.uid);
            block.set_seq_num(SBX_FIRST_DATA_SEQ_NUM);
        }

        for pos in self.slot_read_pos.iter_mut() {
            *pos = None;
        }

        self.slot_write_pos_usable = false;

        for pos in self.slot_write_pos.iter_mut() {
            *pos = None;
        }

        for len in self.slot_content_len_exc_header.iter_mut() {
            *len = None;
        }

        for is_padding in self.slot_is_padding.iter_mut() {
            *is_padding = false;
        }
    }

    fn data_padding_parity_block_count(&self) -> (usize, usize, usize) {
        match self.input_type {
            InputType::Data => {}
            InputType::Block(arrangement) => {
                assert!(arrangement == BlockArrangement::OrderedAndNoMissing)
            }
        }

        let data = match self.data_par_burst {
            None => self.slots_used,
            Some((data, _, _)) => std::cmp::min(data, self.slots_used),
        };

        let mut padding = 0;
        for slot_index in 0..self.slots_used {
            if self.slot_is_padding[slot_index] {
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
        assert!(self.output_type != OutputType::Disabled);

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
                            OutputType::Disabled => panic!(),
                        };

                        let read_res = writer.read(check_buffer).unwrap()?;
                        writer.seek(SeekFrom::Start(cur_pos)).unwrap()?;

                        let do_write = match self.output_type {
                            OutputType::Block => {
                                let block = &self.blocks[slot_index];

                                read_res.eof_seen || {
                                    match self.check_block.sync_from_buffer(
                                        check_buffer,
                                        None,
                                        None,
                                    ) {
                                        Ok(()) => {
                                            self.check_block.get_version() != block.get_version()
                                                || self.check_block.get_uid() != block.get_uid()
                                                || self.check_block.get_seq_num()
                                                    != block.get_seq_num()
                                        }
                                        Err(_) => true,
                                    }
                                }
                            }
                            OutputType::Data => {
                                read_res.eof_seen || misc_utils::buffer_is_blank(check_buffer)
                            }
                            OutputType::Disabled => panic!(),
                        };

                        if !do_write {
                            continue;
                        }
                    }

                    let slot = match self.output_type {
                        OutputType::Block => slot,
                        OutputType::Data => {
                            let data = sbx_block::slice_data_buf(self.version, slot);
                            match self.slot_content_len_exc_header[slot_index] {
                                None => data,
                                Some(len) => &data[..len],
                            }
                        }
                        OutputType::Disabled => panic!(),
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
    fn new(
        version: Version,
        uid: Option<&[u8; SBX_FILE_UID_LEN]>,
        input_type: InputType,
        output_type: OutputType,
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        skip_good: bool,
        buffer_index: usize,
        total_buffer_count: usize,
    ) -> Self {
        let lot_count = num_cpus::get() * LOT_COUNT_PER_CPU;

        assert!(lot_count > 0);

        check_data_par_burst_consistent_with_version!(data_par_burst, version);

        let mut lots = Vec::with_capacity(lot_count);

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
                data_par_burst,
                meta_enabled,
                skip_good,
                DEFAULT_SINGLE_LOT_SIZE,
                &rs_codec,
            ))
        }

        let lot_size = lots[0].lot_size;

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

    pub fn new_multi(
        version: Version,
        uid: Option<&[u8; SBX_FILE_UID_LEN]>,
        input_type: InputType,
        output_type: OutputType,
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        skip_good: bool,
        total_buffer_count: usize,
    ) -> Vec<Self> {
        let mut res = Vec::with_capacity(total_buffer_count);

        for i in 0..total_buffer_count {
            res.push(Self::new(
                version,
                uid,
                input_type,
                output_type,
                data_par_burst,
                meta_enabled,
                skip_good,
                i,
                total_buffer_count,
            ));
        }

        res
    }

    pub fn lot_count(&self) -> usize {
        self.lots.len()
    }

    pub fn total_slot_count(&self) -> usize {
        self.lots[0].lot_size * self.lot_count()
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
                GetSlotResult::LastSlot(block, slot, read_pos, content_len_exc_header) => {
                    self.lots_used += 1;
                    Some(Slot {
                        block,
                        slot,
                        read_pos,
                        content_len_exc_header,
                    })
                }
                GetSlotResult::Some(block, slot, read_pos, content_len_exc_header) => Some(Slot {
                    block,
                    slot,
                    read_pos,
                    content_len_exc_header,
                }),
                GetSlotResult::None => {
                    self.lots_used += 1;
                    None
                }
            }
        }
    }

    pub fn view_slots(&self) -> Vec<SlotView> {
        let mut res = Vec::with_capacity(self.total_slot_count());

        for lot in self.lots.iter() {
            for slot in lot.view_slots().into_iter() {
                res.push(slot);
            }
        }

        res
    }

    pub fn cancel_slot(&mut self) {
        assert!(self.active());

        let shift_back_one_slot = self.is_full() || self.lots[self.lots_used].slots_used == 0;

        if shift_back_one_slot {
            self.lots_used -= 1;
        }

        self.lots[self.lots_used].cancel_slot();
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

        self.lots.par_iter_mut().for_each(|lot| {
            lot.reset();
        })
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

    pub fn calc_slot_write_pos(&mut self) {
        self.lots.par_iter_mut().for_each(|lot| {
            lot.calc_slot_write_pos();
        })
    }

    fn write_internal(&mut self, seek: bool, writer: &mut Writer) -> Result<(), Error> {
        self.calc_slot_write_pos();

        for lot in self.lots.iter_mut() {
            lot.write(seek, writer)?;
        }

        Ok(())
    }

    pub fn write(&mut self, writer: &mut Writer) -> Result<(), Error> {
        self.write_internal(true, writer)
    }

    pub fn write_no_seek(&mut self, writer: &mut Writer) -> Result<(), Error> {
        self.write_internal(false, writer)
    }
}
