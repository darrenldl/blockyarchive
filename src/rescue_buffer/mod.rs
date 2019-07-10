use crate::sbx_specs::{
    ver_uses_rs, Version, SBX_FIRST_DATA_SEQ_NUM, SBX_LARGEST_BLOCK_SIZE, SBX_LAST_SEQ_NUM, SBX_FILE_UID_LEN, ver_to_block_size,
};

use crate::sbx_block::{calc_data_block_write_pos, calc_data_chunk_write_pos, Block, BlockType};

use crate::file_writer::FileWriter;

macro_rules! slice_slot_w_index {
    (
        $self:expr, $index:expr
    ) => {{
        let version = $self.blocks[$index].get_version();
        let block_size = ver_to_block_size(version);

        let start = $index * SBX_LARGEST_BLOCK_SIZE;
        let end_exc = start + block_size;

        &$self.data[start..end_exc]
    }};
    (
        mut => $self:expr, $index:expr
    ) => {{
        let version = $self.blocks[$index].get_version();
        let block_size = ver_to_block_size(version);

        let start = $index * SBX_LARGEST_BLOCK_SIZE;
        let end_exc = start + block_size;

        &mut $self.data[start..end_exc]
    }};
}

pub struct Slot<'a> {
    pub block: &'a mut Block,
    pub slot: &'a mut [u8],
}

pub struct RescueBuffer {
    size: usize,
    slots_used: usize,
    blocks: Vec<Block>,
    data: Vec<u8>,
}

impl RescueBuffer {
    pub fn new(size: usize) -> Self {
        RescueBuffer {
            size,
            slots_used: 0,
            blocks: Vec::with_capacity(size),
            data: Vec::with_capacity(size * SBX_LARGEST_BLOCK_SIZE),
        }
    }

    pub fn get_slot(&mut self) -> Option<Slot> {
        if self.slots_used == self.size {
            None
        } else {
            let slot = slice_slot_w_index!(mut => self, self.slots_used);
            let block = &mut self.blocks[self.slots_used];

            self.slots_used += 1;

            Some(Slot {
                block,
                slot,
            })
        }
    }

    pub fn reset(&mut self) {
        self.slots_used = 0;

        for block in self.blocks.iter_mut() {
            block.set_version(Version::V1);
            block.set_uid([0; SBX_FILE_UID_LEN]);
            block.set_seq_num(SBX_FIRST_DATA_SEQ_NUM);
        }
    }

    pub fn reset_slot(&mut self, slot_index: usize) {
        let block = &mut self.blocks[slot_index];
        block.set_version(Version::V1);
        block.set_uid([0; SBX_FILE_UID_LEN]);
        block.set_seq_num(SBX_FIRST_DATA_SEQ_NUM);
    }

    pub fn cancel_slot(&mut self) {
        assert!(self.slots_used > 0);

        self.slots_used -= 1;
    }

    pub fn is_full(&self) -> bool {
        self.slots_used == self.size
    }
}
