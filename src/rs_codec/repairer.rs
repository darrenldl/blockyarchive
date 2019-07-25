#![allow(dead_code)]
use crate::sbx_block;
use crate::sbx_block::Block;
use crate::sbx_specs::{
    ver_to_block_size, Version, SBX_FIRST_DATA_SEQ_NUM, SBX_LARGEST_BLOCK_SIZE,
};
use reed_solomon_erasure::ReedSolomon;
use smallvec::SmallVec;
use std::sync::Arc;
use std::fmt;
use crate::json_printer::{BracketType, JSONPrinter};
use super::RSCodecState;

pub struct RSRepairer {
    index: usize,
    rs_codec: ReedSolomon,
    data_par_burst: (usize, usize, usize),
    version: Version,
    buf: SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_present: SmallVec<[bool; 32]>,
    ref_block: Block,
    active: bool,
    json_printer: Arc<JSONPrinter>,
}

pub struct RSRepairStats<'a> {
    pub version: Version,
    pub data_par_burst: (usize, usize, usize),
    pub successful: bool,
    pub start_seq_num: u32,
    pub present: &'a SmallVec<[bool; 32]>,
    pub missing_count: usize,
    pub present_count: usize,
    json_printer: Arc<JSONPrinter>,
}

impl<'a> fmt::Display for RSRepairStats<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let block_size = ver_to_block_size(self.version) as u64;

        let json_printer = &self.json_printer;

        let end_seq_num_inc = self.start_seq_num + self.present.len() as u32 - 1;

        if json_printer.json_enabled() {
            if self.missing_count > 0 {
                json_printer.write_open_bracket(f, None, BracketType::Curly)?;

                if self.successful {
                    write_maybe_json!(f, json_printer, "success : true")?;
                } else {
                    write_maybe_json!(f, json_printer, "success : false")?;
                }

                write_maybe_json!(f, json_printer, "block set start : {}", self.start_seq_num)?;
                write_maybe_json!(
                    f,
                    json_printer,
                    "block set end inclusive : {}",
                    end_seq_num_inc
                )?;

                {
                    json_printer.write_open_bracket(f, Some("blocks"), BracketType::Square)?;

                    for i in 0..self.present.len() {
                        if !self.present[i] {
                            json_printer.write_open_bracket(f, None, BracketType::Curly)?;

                            let seq_num = self.start_seq_num + i as u32;

                            let index = sbx_block::calc_data_block_write_index(
                                seq_num,
                                None,
                                Some(self.data_par_burst),
                            );
                            let block_pos = index * block_size;

                            write_maybe_json!(f, json_printer, "seq num : {}", seq_num)?;
                            write_maybe_json!(f, json_printer, "pos : {}", block_pos)?;

                            json_printer.write_close_bracket(f)?;
                        }
                    }

                    json_printer.write_close_bracket(f)?;
                }

                json_printer.write_close_bracket(f)?;

                Ok(())
            } else {
                Ok(())
            }
        } else {
            if self.missing_count > 0 {
                if self.successful {
                    write!(f, "Repair successful for ")?;
                } else {
                    write!(f, "Repair failed     for ")?;
                }

                write!(
                    f,
                    "block set [{}..={}], ",
                    self.start_seq_num, end_seq_num_inc
                )?;

                if self.successful {
                    write!(f, "repaired block no. : ")?;
                } else {
                    write!(f, "failed   block no. : ")?;
                }

                let mut first_num = true;
                for i in 0..self.present.len() {
                    if !self.present[i] {
                        let seq_num = self.start_seq_num + i as u32;

                        if !first_num {
                            writeln!(f, "")?;
                        }

                        let index = sbx_block::calc_data_block_write_index(
                            seq_num,
                            None,
                            Some(self.data_par_burst),
                        );
                        let block_pos = index * block_size;

                        write!(f, "{} at byte {} (0x{:X})", seq_num, block_pos, block_pos)?;

                        first_num = false;
                    }
                }
                Ok(())
            } else {
                Ok(())
            }
        }
    }
}

macro_rules! mark_active {
    (
        $self:ident
    ) => {{
        $self.active = true;
    }};
}

macro_rules! mark_inactive {
    (
        $self:ident
    ) => {{
        $self.active = false;
    }};
}

macro_rules! incre_index {
    (
        $self:ident
    ) => {{
        $self.index += 1;
    }};
}

macro_rules! reset_index {
    (
        $self:ident
    ) => {{
        $self.index = 0;
    }};
}

macro_rules! codec_ready {
    (
        $self:ident
    ) => {{
        $self.index == $self.rs_codec.total_shard_count()
    }};
}

impl RSRepairer {
    pub fn new(
        json_printer: &Arc<JSONPrinter>,
        ref_block: &Block,
        data_shards: usize,
        parity_shards: usize,
        burst: usize,
    ) -> RSRepairer {
        let version = ref_block.get_version();
        let block_size = ver_to_block_size(version);

        let buf: SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; data_shards + parity_shards];
        let buf_present: SmallVec<[bool; 32]> = smallvec![false; data_shards + parity_shards];

        RSRepairer {
            index: 0,
            rs_codec: ReedSolomon::new(data_shards, parity_shards).unwrap(),
            data_par_burst: (data_shards, parity_shards, burst),
            version,
            buf,
            buf_present,
            ref_block: ref_block.clone(),
            active: false,
            json_printer: Arc::clone(json_printer),
        }
    }

    pub fn get_block_buffer(&mut self) -> &mut [u8] {
        assert_not_ready!(self);

        sbx_block::slice_buf_mut(self.version, &mut self.buf[self.index])
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn unfilled_slot_count(&self) -> usize {
        self.total_slot_count() - self.index
    }

    pub fn total_slot_count(&self) -> usize {
        self.rs_codec.total_shard_count()
    }

    pub fn mark_present(&mut self) -> RSCodecState {
        assert_not_ready!(self);

        self.buf_present[self.index] = true;

        incre_index!(self);

        mark_active!(self);

        if codec_ready!(self) {
            RSCodecState::Ready
        } else {
            RSCodecState::NotReady
        }
    }

    pub fn mark_missing(&mut self) -> RSCodecState {
        assert_not_ready!(self);

        self.buf_present[self.index] = false;

        incre_index!(self);

        mark_active!(self);

        if codec_ready!(self) {
            RSCodecState::Ready
        } else {
            RSCodecState::NotReady
        }
    }

    fn missing_count(&self) -> usize {
        self.rs_codec.total_shard_count() - self.present_count()
    }

    fn present_count(&self) -> usize {
        let mut count = 0;
        for p in self.buf_present.iter() {
            if *p {
                count += 1;
            }
        }
        count
    }

    pub fn repair_with_block_sync(
        &mut self,
        seq_num: u32,
    ) -> (RSRepairStats, SmallVec<[(u64, &[u8]); 32]>) {
        assert_ready!(self);

        assert!(seq_num >= SBX_FIRST_DATA_SEQ_NUM);

        let mut repaired_blocks = SmallVec::with_capacity(self.rs_codec.parity_shard_count());

        let rs_codec = &self.rs_codec;

        let successful = {
            let mut buf: SmallVec<[&mut [u8]; 32]> =
                SmallVec::with_capacity(rs_codec.total_shard_count());
            for s in self.buf.iter_mut() {
                buf.push(sbx_block::slice_data_buf_mut(self.version, s));
            }

            // reconstruct data portion
            match rs_codec.reconstruct(&mut buf, &self.buf_present) {
                Ok(()) => true,
                Err(_) => false,
            }
        };

        let block_set_size = self.rs_codec.total_shard_count() as u32;

        let data_index = seq_num - SBX_FIRST_DATA_SEQ_NUM;

        let block_set_index = data_index / block_set_size;

        let first_data_index_in_cur_set = block_set_index * block_set_size;

        let first_seq_num_in_cur_set = first_data_index_in_cur_set + 1;

        // reconstruct header if successful
        if successful {
            for i in 0..block_set_size as usize {
                if !self.buf_present[i] {
                    self.ref_block
                        .set_seq_num(first_seq_num_in_cur_set + i as u32);
                    self.ref_block
                        .sync_to_buffer(None, &mut self.buf[i])
                        .unwrap();
                }
            }
            for i in 0..block_set_size as usize {
                let cur_seq_num = first_seq_num_in_cur_set + i as u32;
                if !self.buf_present[i] {
                    let pos = sbx_block::calc_data_block_write_pos(
                        self.version,
                        cur_seq_num,
                        None,
                        Some(self.data_par_burst),
                    );
                    repaired_blocks.push((pos, sbx_block::slice_buf(self.version, &self.buf[i])));
                }
            }
        }

        mark_inactive!(self);

        reset_index!(self);

        (
            RSRepairStats {
                version: self.version,
                data_par_burst: self.data_par_burst,
                successful,
                json_printer: Arc::clone(&self.json_printer),
                start_seq_num: first_seq_num_in_cur_set,
                present: &self.buf_present,
                missing_count: self.missing_count(),
                present_count: self.present_count(),
            },
            repaired_blocks,
        )
    }
}
