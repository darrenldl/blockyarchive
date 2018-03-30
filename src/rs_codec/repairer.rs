#![allow(dead_code)]
use reed_solomon_erasure::ReedSolomon;
use smallvec::SmallVec;
use sbx_block;
use sbx_block::Block;
use sbx_specs::{Version,
                ver_to_block_size,
                SBX_LARGEST_BLOCK_SIZE,
                SBX_FIRST_DATA_SEQ_NUM};

use std::fmt;

use super::RSCodecState;

pub struct RSRepairer {
    index          : usize,
    rs_codec       : ReedSolomon,
    data_par_burst : Option<(usize, usize, usize)>,
    version        : Version,
    buf            : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]>,
    buf_present    : SmallVec<[bool; 32]>,
    ref_block      : Block,
    active         : bool,
}

pub struct RSRepairStats<'a> {
    pub version        : Version,
    pub data_par_burst : Option<(usize, usize, usize)>,
    pub successful     : bool,
    pub start_seq_num  : u32,
    pub present        : &'a SmallVec<[bool; 32]>,
    pub missing_count  : usize,
    pub present_count  : usize,
}

impl<'a> fmt::Display for RSRepairStats<'a> {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let block_size = ver_to_block_size(self.version) as u64;

        if self.missing_count > 0 {
            if self.successful {
                write!(f, "Repair successful for ")?;
            } else {
                write!(f, "Repair failed     for ")?;
            }

            write!(f, "block set [{}..={}], ",
                   self.start_seq_num,
                   self.start_seq_num + self.present.len() as u32 - 1)?;

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

                    let index     =
                        sbx_block::calc_data_block_write_index(seq_num,
                                                               None,
                                                               self.data_par_burst);
                    let block_pos = index * block_size;

                    write!(f, "{} at byte {} (0x{:X})",
                           seq_num,
                           block_pos,
                           block_pos)?;

                    first_num = false;
                }
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

macro_rules! mark_active {
    (
        $self:ident
    ) => {{
        $self.active = true;
    }}
}

macro_rules! mark_inactive {
    (
        $self:ident
    ) => {{
        $self.active = false;
    }}
}

macro_rules! incre_index {
    (
        $self:ident
    ) => {{
        $self.index += 1;
    }}
}

macro_rules! reset_index {
    (
        $self:ident
    ) => {{
        $self.index = 0;
    }}
}

macro_rules! codec_ready {
    (
        $self:ident
    ) => {{
        $self.index == $self.rs_codec.total_shard_count()
    }}
}

impl RSRepairer {
    pub fn new(version       : Version,
               ref_block     : &Block,
               data_shards   : usize,
               parity_shards : usize,
               burst         : usize) -> RSRepairer {
        let block_size = ver_to_block_size(version);

        let buf : SmallVec<[SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>; 32]> =
            smallvec![smallvec![0; block_size]; data_shards + parity_shards];
        let buf_present : SmallVec<[bool; 32]> =
            smallvec![false; data_shards + parity_shards];

        RSRepairer {
            index          : 0,
            rs_codec       : ReedSolomon::new(data_shards,
                                           parity_shards).unwrap(),
            data_par_burst : Some((data_shards, parity_shards, burst)),
            version,
            buf,
            buf_present,
            ref_block      : ref_block.clone(),
            active         : false,
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
            if *p { count += 1; }
        }
        count
    }

    pub fn repair_with_block_sync(&mut self,
                                  seq_num : u32)
                                  ->
        (RSRepairStats,
         SmallVec<[(u64, &[u8]); 32]>)
    {
        assert_ready!(self);

        let mut repaired_blocks =
            SmallVec::with_capacity(self.rs_codec.parity_shard_count());

        let rs_codec      = &self.rs_codec;

        let successful;
        {
            let mut buf : SmallVec<[&mut [u8]; 32]> =
                SmallVec::with_capacity(rs_codec.total_shard_count());
            for s in self.buf.iter_mut() {
                buf.push(sbx_block::slice_data_buf_mut(self.version, s));
            }

            // reconstruct data portion
            successful =
                match rs_codec.reconstruct(&mut buf, &self.buf_present) {
                    Ok(()) => true,
                    Err(_) => false
                };
        }

        let block_set_size = self.rs_codec.total_shard_count() as u32;

        let data_index = seq_num - SBX_FIRST_DATA_SEQ_NUM;

        let block_set_index = data_index / block_set_size;

        let first_data_index_in_cur_set = block_set_index * block_set_size;

        let first_seq_num_in_cur_set = first_data_index_in_cur_set + 1;

        // reconstruct header if successful
        if successful {
            for i in 0..block_set_size as usize {
                if !self.buf_present[i] {
                    self.ref_block.set_seq_num(first_seq_num_in_cur_set + i as u32);
                    self.ref_block.sync_to_buffer(None,
                                                  &mut self.buf[i]).unwrap();
                }
            }
            for i in 0..block_set_size as usize {
                let cur_seq_num = first_seq_num_in_cur_set + i as u32;
                if !self.buf_present[i] {
                    let pos = sbx_block::calc_data_block_write_pos(self.version,
                                                                   cur_seq_num,
                                                                   None,
                                                                   self.data_par_burst);
                    repaired_blocks.push((pos, sbx_block::slice_buf(self.version,
                                                                    &self.buf[i])));
                }
            }
        }

        mark_inactive!(self);

        reset_index!(self);

        (RSRepairStats { version        : self.version,
                         data_par_burst : self.data_par_burst,
                         successful,
                         start_seq_num  : first_seq_num_in_cur_set,
                         present        : &self.buf_present,
                         missing_count  : self.missing_count(),
                         present_count  : self.present_count(), },
         repaired_blocks)
    }
}
