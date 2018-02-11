use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use super::time_utils;
use std::io::SeekFrom;

use super::progress_report::*;

use super::file_reader::FileReader;
use super::file_writer::FileWriter;

use super::sbx_specs::SBX_SCAN_BLOCK_SIZE;

use super::multihash;

use super::Error;
use super::sbx_specs::Version;

use super::sbx_block::MetadataID;
use super::sbx_block::Metadata;

use super::sbx_block::{Block, BlockType};
use super::sbx_block;
use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_specs::{ver_to_block_size,
                       ver_to_data_size,
                       ver_first_data_seq_num};

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    version                     : Version,
    pub meta_blocks_decoded     : u32,
    pub meta_par_blocks_decoded : u32,
    pub data_blocks_decoded     : u32,
    pub data_par_blocks_decoded : u32,
    blocks_decode_failed        : u32,
    total_blocks                : u32,
    start_time                  : f64,
    end_time                    : f64,
}

pub struct ScanStats {
    pub bytes_processed : u64,
    pub total_bytes     : u64,
    start_time          : f64,
    end_time            : f64,
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    no_meta  : bool,
    in_file  : String,
    out_file : String,
    silence_level : SilenceLevel
}

impl Param {
    pub fn new(no_meta  : bool,
               in_file  : &str,
               out_file : &str,
               silence_level : SilenceLevel) -> Param {
        Param {
            no_meta,
            in_file  : String::from(in_file),
            out_file : String::from(out_file),
            silence_level,
        }
    }
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

impl Stats {
    pub fn new(ref_block     : &Block,
               file_metadata : &fs::Metadata) -> Stats {
        let total_blocks =
            file_utils::calc_total_block_count(ref_block.get_version(),
                                               file_metadata) as u32;
        Stats {
            version                 : ref_block.get_version(),
            blocks_decode_failed    : 0,
            meta_blocks_decoded     : 0,
            meta_par_blocks_decoded : 0,
            data_blocks_decoded     : 0,
            data_par_blocks_decoded : 0,
            total_blocks,
            start_time              : 0.,
            end_time                : 0.,
        }
    }
}

impl ProgressReport for ScanStats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      { self.bytes_processed }

    fn total_units(&self)        -> u64      { self.total_bytes }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      {
        (self.meta_blocks_decoded
         + self.meta_par_blocks_decoded
         + self.data_blocks_decoded
         + self.data_par_blocks_decoded) as u64
    }

    fn total_units(&self)        -> u64      { self.total_blocks as u64 }
}

fn get_ref_block(reader   : &mut FileReader,
                 param    : &Param,
                 metadata : &fs::Metadata)
                 -> Result<Option<Block>, Error> {
    let stats = Arc::new(Mutex::new(ScanStats::new(&metadata)));

    let mut reporter = ProgressReporter::new(&stats,
                                             "Scan progress",
                                             "bytes",
                                             param.silence_level);

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    let mut block = Block::dummy();

    let mut meta_block = None;
    let mut data_block = None;

    reader.seek(SeekFrom::Start(0))?;

    reporter.start();

    loop {
        { // scan at 128 chunk size
            let len_read = reader.read(&mut buffer[0..SBX_SCAN_BLOCK_SIZE])?;

            stats.lock().unwrap().bytes_processed += len_read as u64;

            if len_read < SBX_SCAN_BLOCK_SIZE {
                break;
            }

            match block.sync_from_buffer_header_only(&buffer[0..SBX_SCAN_BLOCK_SIZE]) {
                Ok(()) => {},
                Err(_) => { continue; }
            }
        }

        { // get remaining bytes of block if necessary
            let block_size = ver_to_block_size(block.get_version());

            let remaining_size = block_size - SBX_SCAN_BLOCK_SIZE;

            let len_read = reader.read(&mut buffer[SBX_SCAN_BLOCK_SIZE..block_size])?;

            stats.lock().unwrap().bytes_processed += len_read as u64;

            if len_read < remaining_size {
                break;
            }

            match block.sync_from_buffer(&buffer[0..block_size]) {
                Ok(()) => {},
                Err(_) => { continue; }
            }
        }

        match block.block_type() {
            BlockType::Meta => {
                if let None = meta_block {
                    meta_block = Some(block.clone());
                }
            },
            BlockType::Data => {
                if let None = data_block {
                    data_block = Some(block.clone());
                }
            }
        }

        if param.no_meta {
            if let Some(_) = meta_block { break; }
            if let Some(_) = data_block { break; }
        } else {
            if let Some(_) = meta_block { break; }
        }
    }

    reporter.stop();

    reader.seek(SeekFrom::Start(0))?;

    Ok(if let Some(_) = meta_block {
        meta_block
    } else {
        data_block
    })
}

pub fn decode_file(param : &Param)
                   -> Result<Stats, Error> {
    let metadata = file_utils::get_file_metadata(&param.in_file)?;

    let mut reader = FileReader::new(&param.in_file)?;
    let mut writer = FileWriter::new(&param.out_file)?;

    // find a reference block
    let ref_block =
        match get_ref_block(&mut reader, param, &metadata)? {
            None => { return Err(Error::with_message("failed to find reference block")); },
            Some(x) => x,
        };

    let stats = Arc::new(Mutex::new(Stats::new(&ref_block, &metadata)));

    let mut block = Block::dummy();

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let seq_num_offset = ver_first_data_seq_num(ref_block.get_version()) as u32;

    let block_size   = ver_to_block_size(ref_block.get_version());

    let data_size    = ver_to_data_size(ref_block.get_version()) as u64;

    let hash_enabled =
        if ref_block.is_data() {
            false
        } else {
            match ref_block.get_meta_ref_by_id(MetadataID::HSH).unwrap() {
                None    => false,
                Some(_) => true,
            }
        };

    loop {
        // read at reference block block size
        let len_read = reader.read(sbx_block::slice_buf_mut(ref_block.get_version(),
                                                            &mut buffer))?;

        if len_read < block_size {
            break;
        }

        if let Err(_) = block.sync_from_buffer(&buffer) {
            stats.lock().unwrap().blocks_decode_failed += 1;
            continue;
        }

        if block.is_meta() { // do nothing if block is meta or meta parity
            stats.lock().unwrap().meta_blocks_decoded += 1;
        } else {
            stats.lock().unwrap().data_blocks_decoded += 1;

            // write data block
            let write_to = (block.get_seq_num() - seq_num_offset) as u64 * data_size;

            writer.seek(SeekFrom::Start(write_to as u64))?;

            writer.write(sbx_block::slice_data_buf(ref_block.get_version(),
                                                   &buffer))?;
        }
    }

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
