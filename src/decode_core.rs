use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use super::time_utils;
use std::io::SeekFrom;

use integer_utils::IntegerUtils;

use progress_report::ProgressReport;
use progress_report::ProgressReporter;

use super::progress_report;

use std::time::UNIX_EPOCH;

use super::file_reader::FileReader;
use super::file_writer::FileWriter;

use super::sbx_specs::SBX_SCAN_BLOCK_SIZE;

use super::multihash;

use super::Error;
use super::sbx_specs::Version;
use super::rs_codec::RSEncoder;

use super::sbx_block::{Block, BlockType};
use super::sbx_block;
use super::sbx_block::metadata::Metadata;
use super::sbx_specs::SBX_FILE_UID_LEN;
use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_specs::SBX_RS_METADATA_PARITY_COUNT;
use super::sbx_specs::ver_forces_meta_enabled;
use super::sbx_specs::{ver_to_block_size,
                       ver_to_data_size,
                       ver_supports_rs};

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    version                     : Version,
    pub meta_blocks_decoded     : u32,
    pub meta_par_blocks_decoded : u32,
    pub data_blocks_decoded     : u32,
    pub data_par_blocks_decoded : u32,
    total_blocks                : u32,
    start_time                  : f64,
    end_time                    : f64,
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    in_file  : String,
    out_file : String,
    silence_level : progress_report::SilenceLevel
}

impl Param {
    pub fn new(in_file  : &str,
               out_file : &str,
               silence_level : progress_report::SilenceLevel) -> Param {
        Param {
            in_file  : String::from(in_file),
            out_file : String::from(out_file),
            silence_level,
        }
    }
}

impl Stats {
    pub fn new(ref_block     : Block,
               param         : &Param,
               file_metadata : &fs::Metadata) -> Stats {
        let total_blocks =
            file_utils::calc_total_block_count(ref_block.get_version(),
                                               file_metadata) as u32;
        Stats {
            version                 : ref_block.get_version(),
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

fn get_ref_block(reader  : &mut FileReader,
                 no_meta : bool)
                 -> Result<Option<Block>, Error> {
    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    let mut block = Block::new(Version::V1,
                               b"\x00\x00\x00\x00\x00\x00",
                               BlockType::Data);

    let mut meta_block = None;
    let mut data_block = None;

    reader.seek(SeekFrom::Start(0))?;

    loop {
        // scan at 128 chunk size
        let len_read = reader.read(&mut buffer[0..SBX_SCAN_BLOCK_SIZE])?;

        if len_read < SBX_SCAN_BLOCK_SIZE {
            break;
        }

        match block.sync_from_buffer_header_only(&buffer[0..SBX_SCAN_BLOCK_SIZE]) {
            Ok(()) => {},
            Err(_) => { continue; }
        }

        let block_size = ver_to_block_size(block.get_version());

        let remaining_size = block_size - SBX_SCAN_BLOCK_SIZE;

        let len_read = reader.read(&mut buffer[SBX_SCAN_BLOCK_SIZE..block_size])?;

        if len_read < remaining_size {
            break;
        }

        match block.sync_from_buffer(&buffer[0..block_size]) {
            Ok(()) => {},
            Err(_) => { continue; }
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

        if no_meta {
            if let Some(_) = meta_block { break; }
            if let Some(_) = data_block { break; }
        } else {
            if let Some(_) = meta_block { break; }
        }
    }

    reader.seek(SeekFrom::Start(0))?;

    Ok(if let Some(_) = meta_block {
        meta_block
    } else {
        data_block
    })
}
