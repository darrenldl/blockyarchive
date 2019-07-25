use std::collections::HashMap;
use std::collections::LinkedList;
use crate::general_error::Error;
use crate::misc_utils;
use crate::sbx_specs::{
    ver_to_block_size, Version, SBX_FILE_UID_LEN, SBX_FIRST_DATA_SEQ_NUM, SBX_LARGEST_BLOCK_SIZE,
};
use crate::sbx_block::Block;
use crate::file_writer::{FileWriter, FileWriterParam};

mod tests;

macro_rules! slice_slot_w_index {
    (
        full => $self:expr, $index:expr
    ) => {{
        let start = $index * SBX_LARGEST_BLOCK_SIZE;
        let end_exc = start + SBX_LARGEST_BLOCK_SIZE;

        &$self.data[start..end_exc]
    }};
    (
        full => mut => $self:expr, $index:expr
    ) => {{
        let start = $index * SBX_LARGEST_BLOCK_SIZE;
        let end_exc = start + SBX_LARGEST_BLOCK_SIZE;

        &mut $self.data[start..end_exc]
    }};
    (
        depend_on_block_ver => $self:expr, $index:expr
    ) => {{
        let version = $self.blocks[$index].get_version();
        let block_size = ver_to_block_size(version);

        let start = $index * SBX_LARGEST_BLOCK_SIZE;
        let end_exc = start + block_size;

        &$self.data[start..end_exc]
    }};
    (
        depend_on_block_ver => mut => $self:expr, $index:expr
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
    uid_to_slot_indices: HashMap<[u8; SBX_FILE_UID_LEN], LinkedList<usize>>,
}

impl RescueBuffer {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let mut blocks = Vec::with_capacity(size);

        for _ in 0..size {
            blocks.push(Block::dummy());
        }

        RescueBuffer {
            size,
            slots_used: 0,
            blocks,
            data: vec![0; size * SBX_LARGEST_BLOCK_SIZE],
            uid_to_slot_indices: HashMap::with_capacity(size),
        }
    }

    pub fn get_slot(&mut self) -> Option<Slot> {
        if self.slots_used == self.size {
            None
        } else {
            let slot = slice_slot_w_index!(full => mut => self, self.slots_used);
            let block = &mut self.blocks[self.slots_used];

            self.slots_used += 1;

            Some(Slot { block, slot })
        }
    }

    pub fn group_by_uid(&mut self) {
        for i in 0..self.slots_used {
            let uid = self.blocks[i].get_uid();

            match self.uid_to_slot_indices.get_mut(&uid) {
                Some(l) => l.push_back(i),
                None => {
                    let mut l = LinkedList::new();
                    l.push_front(i);
                    self.uid_to_slot_indices.insert(uid, l);
                }
            }
        }
    }

    pub fn write(&mut self, out_dir: &str) -> Result<(), Error> {
        for (uid, l) in self.uid_to_slot_indices.iter() {
            let uid_str = misc_utils::bytes_to_upper_hex_string(uid);
            let path = misc_utils::make_path(&[out_dir, &uid_str]);

            let mut writer = FileWriter::new(
                &path,
                FileWriterParam {
                    read: false,
                    append: true,
                    truncate: false,
                    buffered: true,
                },
            )?;

            for &i in l.iter() {
                let slot = slice_slot_w_index!(depend_on_block_ver => self, i);

                writer.write(slot)?;
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.slots_used = 0;

        self.uid_to_slot_indices.clear();

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

        self.reset_slot(self.slots_used);
    }

    pub fn is_full(&self) -> bool {
        self.slots_used == self.size
    }
}
