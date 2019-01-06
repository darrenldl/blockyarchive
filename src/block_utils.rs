use std::sync::atomic::AtomicBool;

use std::sync::{Arc, Mutex};

use crate::file_reader::{FileReader, FileReaderParam};
use crate::sbx_block::{Block, BlockType};

use crate::file_utils;

use crate::misc_utils;

use crate::misc_utils::RequiredLenAndSeekTo;
use std::io::SeekFrom;

use smallvec::SmallVec;

use crate::sbx_block;

use crate::sbx_specs::{
    ver_to_block_size, ver_to_usize, ver_uses_rs, SBX_LARGEST_BLOCK_SIZE,
    SBX_MAX_BURST_ERR_RESISTANCE, SBX_SCAN_BLOCK_SIZE,
};

use crate::progress_report::*;

use crate::general_error::Error;

use crate::misc_utils::{PositionOrLength, RangeEnd};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RefBlockChoice {
    Any,
    Prefer(BlockType),
    MustBe(BlockType),
}

pub struct LazyReadResult {
    pub len_read: usize,
    pub usable: bool,
    pub eof_seen: bool,
}

struct ScanStats {
    pub bytes_processed: u64,
    pub total_bytes: u64,
    start_time: f64,
    end_time: f64,
}

impl ScanStats {
    pub fn new(file_size: u64) -> ScanStats {
        ScanStats {
            bytes_processed: 0,
            total_bytes: file_size,
            start_time: 0.,
            end_time: 0.,
        }
    }
}

impl ProgressReport for ScanStats {
    fn start_time_mut(&mut self) -> &mut f64 {
        &mut self.start_time
    }

    fn end_time_mut(&mut self) -> &mut f64 {
        &mut self.end_time
    }

    fn units_so_far(&self) -> u64 {
        self.bytes_processed
    }

    fn total_units(&self) -> u64 {
        self.total_bytes
    }
}

pub fn read_block_lazily(
    block: &mut Block,
    buffer: &mut [u8; SBX_LARGEST_BLOCK_SIZE],
    reader: &mut FileReader,
) -> Result<LazyReadResult, Error> {
    let mut total_len_read = 0;

    {
        // scan at 128 chunk size
        total_len_read += reader.read(&mut buffer[0..SBX_SCAN_BLOCK_SIZE])?.len_read;

        if total_len_read < SBX_SCAN_BLOCK_SIZE {
            return Ok(LazyReadResult {
                len_read: total_len_read,
                usable: false,
                eof_seen: true,
            });
        }

        match block.sync_from_buffer_header_only(&buffer[0..SBX_SCAN_BLOCK_SIZE]) {
            Ok(()) => {}
            Err(_) => {
                return Ok(LazyReadResult {
                    len_read: total_len_read,
                    usable: false,
                    eof_seen: false,
                });
            }
        }
    }

    {
        // get remaining bytes of block if necessary
        let block_size = ver_to_block_size(block.get_version());

        total_len_read += reader
            .read(&mut buffer[SBX_SCAN_BLOCK_SIZE..block_size])?
            .len_read;

        if total_len_read < block_size {
            return Ok(LazyReadResult {
                len_read: total_len_read,
                usable: false,
                eof_seen: true,
            });
        }

        match block.sync_from_buffer(&buffer[0..block_size], None) {
            Ok(()) => {}
            Err(_) => {
                return Ok(LazyReadResult {
                    len_read: total_len_read,
                    usable: false,
                    eof_seen: false,
                });
            }
        }
    }

    Ok(LazyReadResult {
        len_read: total_len_read,
        usable: true,
        eof_seen: false,
    })
}

pub fn get_ref_block(
    in_file: &str,
    from_pos: Option<u64>,
    to_pos: Option<RangeEnd<u64>>,
    force_misalign: bool,
    ref_block_choice: RefBlockChoice,
    pr_verbosity_level: PRVerbosityLevel,
    json_enabled: bool,
    stop_flag: &Arc<AtomicBool>,
) -> Result<Option<(u64, Block)>, Error> {
    let file_size = file_utils::get_file_size(in_file)?;

    let RequiredLenAndSeekTo {
        required_len,
        seek_to,
    } = misc_utils::calc_required_len_and_seek_to_from_byte_range(
        from_pos,
        to_pos,
        force_misalign,
        0,
        PositionOrLength::Len(file_size),
        None,
    );

    let stats = Arc::new(Mutex::new(ScanStats::new(required_len)));

    let reporter = ProgressReporter::new(
        &stats,
        "Reference block scanning progress",
        "bytes",
        pr_verbosity_level,
        json_enabled,
    );

    let mut buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut block = Block::dummy();

    let mut meta_block = None;
    let mut data_block = None;

    let mut reader = FileReader::new(
        in_file,
        FileReaderParam {
            write: false,
            buffered: true,
        },
    )?;

    reader.seek(SeekFrom::Start(seek_to))?;

    reporter.start();

    let mut block_pos: u64;
    let mut bytes_processed: u64 = 0;

    loop {
        break_if_atomic_bool!(stop_flag);

        break_if_reached_required_len!(bytes_processed, required_len);

        let lazy_read_res = read_block_lazily(&mut block, &mut buffer, &mut reader)?;

        block_pos = bytes_processed;
        bytes_processed += lazy_read_res.len_read as u64;

        stats.lock().unwrap().bytes_processed = bytes_processed;

        break_if_eof_seen!(lazy_read_res);

        if !lazy_read_res.usable {
            continue;
        }

        match block.block_type() {
            BlockType::Meta => {
                if let None = meta_block {
                    meta_block = Some((block_pos, block.clone()));
                }
            }
            BlockType::Data => {
                if let None = data_block {
                    data_block = Some((block_pos, block.clone()));
                }
            }
        }

        match ref_block_choice {
            RefBlockChoice::Any => {
                if let Some(_) = meta_block {
                    break;
                }
                if let Some(_) = data_block {
                    break;
                }
            }
            RefBlockChoice::Prefer(bt) | RefBlockChoice::MustBe(bt) => match bt {
                BlockType::Meta => {
                    if let Some(_) = meta_block {
                        break;
                    }
                }
                BlockType::Data => {
                    if let Some(_) = data_block {
                        break;
                    }
                }
            },
        }
    }

    reporter.stop();

    Ok(match ref_block_choice {
        RefBlockChoice::Any => match (meta_block, data_block) {
            (Some(m), _) => Some(m),
            (_, Some(d)) => Some(d),
            (None, None) => None,
        },
        RefBlockChoice::Prefer(bt) => match bt {
            BlockType::Meta => match (meta_block, data_block) {
                (Some(m), _) => Some(m),
                (_, Some(d)) => Some(d),
                (None, None) => None,
            },
            BlockType::Data => match (meta_block, data_block) {
                (_, Some(d)) => Some(d),
                (Some(m), _) => Some(m),
                (None, None) => None,
            },
        },
        RefBlockChoice::MustBe(bt) => match bt {
            BlockType::Meta => match (meta_block, data_block) {
                (Some(m), _) => Some(m),
                (_, _) => None,
            },
            BlockType::Data => match (meta_block, data_block) {
                (_, Some(d)) => Some(d),
                (_, _) => None,
            },
        },
    })
}

pub fn guess_burst_err_resistance_level(
    in_file: &str,
    from_pos: Option<u64>,
    force_misalign: bool,
    ref_block_pos: u64,
    ref_block: &Block,
) -> Result<Option<usize>, Error> {
    let rs_enabled = ver_uses_rs(ref_block.get_version());

    if !rs_enabled {
        return Ok(None);
    }

    let data_shards = get_RSD_from_ref_block!(
        ref_block_pos,
        ref_block,
        "guess the burst error resistance level"
    );
    let parity_shards = get_RSP_from_ref_block!(
        ref_block_pos,
        ref_block,
        "guess the burst error resistance level"
    );

    let mut buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut block = Block::dummy();

    let mut reader = FileReader::new(
        in_file,
        FileReaderParam {
            write: false,
            buffered: true,
        },
    )?;

    let from_pos = from_pos.unwrap_or(0);

    let read_offset = if force_misalign {
        from_pos % SBX_SCAN_BLOCK_SIZE as u64
    } else {
        0
    };

    const BLOCKS_TO_SAMPLE_BASE_NUM: usize = 1024;

    let blocks_to_sample = (1 + parity_shards) + SBX_MAX_BURST_ERR_RESISTANCE + 1;

    let mut seq_nums: SmallVec<[Option<u32>; BLOCKS_TO_SAMPLE_BASE_NUM]> =
        smallvec![None; blocks_to_sample];

    let mut mismatches_for_level: [usize; SBX_MAX_BURST_ERR_RESISTANCE + 1] =
        [0; SBX_MAX_BURST_ERR_RESISTANCE + 1];

    let mut blocks_processed = 0;

    let pred = block_pred_same_ver_uid!(ref_block);

    reader.seek(SeekFrom::Start(read_offset))?;

    // record first up to 1 + parity count + 1000 seq nums
    loop {
        let read_res = reader.read(sbx_block::slice_buf_mut(
            ref_block.get_version(),
            &mut buffer,
        ))?;

        break_if_eof_seen!(read_res);

        if blocks_processed >= seq_nums.len() {
            break;
        }

        seq_nums[blocks_processed] = match block.sync_from_buffer(&buffer, Some(&pred)) {
            Ok(()) => Some(block.get_seq_num()),
            Err(_) => None,
        };

        blocks_processed += 1;
    }

    // count mismatches
    for level in 0..mismatches_for_level.len() {
        for index in 0..seq_nums.len() {
            let expected_seq_num = sbx_block::calc_seq_num_at_index(
                index as u64,
                None,
                Some((data_shards, parity_shards, level)),
            );

            if let Some(seq_num) = seq_nums[index] {
                if seq_num != expected_seq_num {
                    mismatches_for_level[level] += 1;
                }
            }
        }
    }

    // find level with fewest mismatches
    let mut best_guess = 0;
    for level in 0..mismatches_for_level.len() {
        if mismatches_for_level[level] < mismatches_for_level[best_guess] {
            best_guess = level;
        }
    }

    // if the best guess has same number of mismatches as other guesses,
    // then just return None
    let mut same_as_best_guess = 0;
    for level in 0..mismatches_for_level.len() {
        if mismatches_for_level[level] == mismatches_for_level[best_guess] {
            same_as_best_guess += 1;
        }
    }

    if same_as_best_guess == mismatches_for_level.len() {
        Ok(None)
    } else {
        Ok(Some(best_guess))
    }
}
