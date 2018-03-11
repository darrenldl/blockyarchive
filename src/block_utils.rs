use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_specs::SBX_SCAN_BLOCK_SIZE;
use super::sbx_block::Block;
use super::file_reader::FileReader;
use super::file_reader::FileReaderParam;
use super::file_writer::FileWriter;
use super::file_writer::FileWriterParam;
use super::sbx_block::BlockType;

use std::sync::{Arc, Mutex};
use std::fs;
use super::file_utils;

use super::sbx_block;

use super::sbx_specs::{ver_to_usize,
                       ver_uses_rs,
                       SBX_FILE_UID_LEN,
                       SBX_MAX_BURST_ERR_RESISTANCE};

use super::progress_report::*;

use super::sbx_specs::ver_to_block_size;
use super::Error;

pub struct LazyReadResult {
    pub len_read : usize,
    pub usable   : bool,
    pub eof_seen : bool,
}

struct ScanStats {
    pub bytes_processed : u64,
    pub total_bytes     : u64,
    start_time          : f64,
    end_time            : f64,
}

impl ScanStats {
    pub fn new(file_metadata : &fs::Metadata) -> ScanStats {
        ScanStats {
            bytes_processed : 0,
            total_bytes     : file_metadata.len(),
            start_time      : 0.,
            end_time        : 0.,
        }
    }
}

impl ProgressReport for ScanStats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      { self.bytes_processed }

    fn total_units(&self)        -> u64      { self.total_bytes }
}

pub fn read_block_lazily(block  : &mut Block,
                         buffer : &mut [u8; SBX_LARGEST_BLOCK_SIZE],
                         reader : &mut FileReader)
                         -> Result<LazyReadResult, Error> {
    let mut total_len_read = 0;

    { // scan at 128 chunk size
        total_len_read += reader.read(&mut buffer[0..SBX_SCAN_BLOCK_SIZE])?.len_read;

        if total_len_read < SBX_SCAN_BLOCK_SIZE {
            return Ok(LazyReadResult { len_read : total_len_read,
                                       usable   : false,
                                       eof_seen : true            });
        }

        match block.sync_from_buffer_header_only(&buffer[0..SBX_SCAN_BLOCK_SIZE]) {
            Ok(()) => {},
            Err(_) => { return Ok(LazyReadResult { len_read : total_len_read,
                                                   usable   : false,
                                                   eof_seen : false           }); }
        }
    }

    { // get remaining bytes of block if necessary
        let block_size = ver_to_block_size(block.get_version());

        total_len_read +=
            reader.read(&mut buffer[SBX_SCAN_BLOCK_SIZE..block_size])?.len_read;

        if total_len_read < block_size {
            return Ok(LazyReadResult { len_read : total_len_read,
                                       usable   : false,
                                       eof_seen : true            });
        }

        match block.sync_from_buffer(&buffer[0..block_size], None) {
            Ok(()) => {},
            Err(_) => { return Ok(LazyReadResult { len_read : total_len_read,
                                                   usable   : false,
                                                   eof_seen : false           }); }
        }
    }

    Ok(LazyReadResult { len_read : total_len_read,
                        usable   : true,
                        eof_seen : false           })
}

pub fn get_ref_block(in_file            : &str,
                     use_any_block_type : bool,
                     silence_level      : SilenceLevel)
                     -> Result<Option<(u64, Block)>, Error> {
    let metadata = file_utils::get_file_metadata(in_file)?;

    let stats = Arc::new(Mutex::new(ScanStats::new(&metadata)));

    let reporter = ProgressReporter::new(&stats,
                                         "Reference block scanning progress",
                                         "bytes",
                                         silence_level);

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    let mut block = Block::dummy();

    let mut meta_block = None;
    let mut data_block = None;

    let mut reader = FileReader::new(in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    reporter.start();

    let mut block_pos       : u64;
    let mut bytes_processed : u64 = 0;

    loop {
        let lazy_read_res = read_block_lazily(&mut block,
                                              &mut buffer,
                                              &mut reader)?;

        block_pos        = bytes_processed;
        bytes_processed += lazy_read_res.len_read as u64;

        stats.lock().unwrap().bytes_processed = bytes_processed;

        break_if_eof_seen!(lazy_read_res);

        if !lazy_read_res.usable { continue; }

        match block.block_type() {
            BlockType::Meta => {
                if let None = meta_block {
                    meta_block = Some((block_pos, block.clone()));
                }
            },
            BlockType::Data => {
                if let None = data_block {
                    data_block = Some((block_pos, block.clone()));
                }
            }
        }

        if use_any_block_type {
            if let Some(_) = meta_block { break; }
            if let Some(_) = data_block { break; }
        } else {
            if let Some(_) = meta_block { break; }
        }
    }

    reporter.stop();

    Ok(if      let Some(x) = meta_block { Some(x) }
       else if let Some(x) = data_block { Some(x) }
       else                             { None    })
}

pub fn guess_burst_err_resistance_level(in_file       : &str,
                                        ref_block_pos : u64,
                                        ref_block     : &Block)
                                        -> Result<Option<usize>, Error> {
    let rs_enabled = ver_uses_rs(ref_block.get_version());

    if !rs_enabled { return Ok(None); }

    let ver_usize = ver_to_usize(ref_block.get_version());

    let data_shards = match ref_block.get_RSD().unwrap() {
        None => { return Err(Error::with_message(&format!("Reference block at byte {} (0x{:X}) is a metadata block but does not have RSD field(must be present to guess the burst error resistance level for version {})",
                                                          ref_block_pos,
                                                          ref_block_pos,
                                                          ver_usize))); },
        Some(x) => x,
    } as usize;

    let parity_shards = match ref_block.get_RSP().unwrap() {
        None => { return Err(Error::with_message(&format!("Reference block at byte {} (0x{:X}) is a metadata block but does not have RSP field(must be present to guess the burst error resistance level for version {})",
                                                          ref_block_pos,
                                                          ref_block_pos,
                                                          ver_usize))); },
        Some(x) => x,
    } as usize;

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    let mut block = Block::dummy();

    let mut reader = FileReader::new(in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    let mut seq_nums : [Option<u32>; SBX_MAX_BURST_ERR_RESISTANCE] =
        [None; SBX_MAX_BURST_ERR_RESISTANCE];

    let mut mismatches_for_level : [usize; SBX_MAX_BURST_ERR_RESISTANCE] =
        [0; SBX_MAX_BURST_ERR_RESISTANCE];

    let mut blocks_processed = 0;

    let pred = {
        let version = ref_block.get_version();
        let uid     = ref_block.get_uid();
        move |block : &Block| -> bool {
            block.get_version() == version
                && block.get_uid() == uid
        }
    };

    // record first up to 1000 seq nums
    loop {
        let read_res = reader.read(sbx_block::slice_buf_mut(ref_block.get_version(),
                                                            &mut buffer))?;

        break_if_eof_seen!(read_res);

        if blocks_processed >= SBX_MAX_BURST_ERR_RESISTANCE { break; }

        seq_nums[blocks_processed] =
            match block.sync_from_buffer(&buffer, Some(&pred)) {
                Ok(()) => Some(block.get_seq_num()),
                Err(_) => None,
            };

        blocks_processed += 1;
    }

    // count mismatches
    for level in 0..mismatches_for_level.len() {
        for index in 0..seq_nums.len() {
            let expected_seq_num =
                sbx_block::calc_rs_enabled_seq_num_at_index(index as u64,
                                                            data_shards,
                                                            parity_shards,
                                                            level);
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

    // if the best guess is completely rubbish, just return None
    if mismatches_for_level[best_guess] == SBX_MAX_BURST_ERR_RESISTANCE {
        return Ok(None);
    }

    Ok(Some(best_guess))
}
