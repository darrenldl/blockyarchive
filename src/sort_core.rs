use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use super::misc_utils;
use std::io::SeekFrom;

use super::progress_report::*;

use super::file_reader::FileReader;
use super::file_reader::FileReaderParam;
use super::file_writer::FileWriter;
use super::file_writer::FileWriterParam;

use super::sbx_specs::SBX_SCAN_BLOCK_SIZE;

use super::multihash;
use super::multihash::*;

use super::Error;
use super::sbx_specs::Version;

use super::sbx_block::{Block, BlockType};
use super::sbx_block;
use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_specs::{ver_to_block_size,
                       ver_to_data_size,
                       ver_uses_rs,
                       ver_to_usize};

use super::report_ref_block_info;

use std::str::from_utf8;

use super::time_utils;
use super::block_utils;

pub struct Param {
    no_meta       : bool,
    in_file       : String,
    out_file      : String,
    silence_level : SilenceLevel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    version                        : Version,
    pub meta_or_par_blocks_decoded : u64,
    pub data_or_par_blocks_decoded : u64,
    pub blocks_decode_failed       : u64,
    total_blocks                   : u64,
    start_time                     : f64,
    end_time                       : f64,
}

impl Stats {
    pub fn new(ref_block     : &Block,
               file_metadata : &fs::Metadata) -> Stats {
        let total_blocks =
            file_utils::calc_total_block_count(ref_block.get_version(),
                                               file_metadata);
        Stats {
            version                 : ref_block.get_version(),
            blocks_decode_failed    : 0,
            meta_or_par_blocks_decoded : 0,
            data_or_par_blocks_decoded : 0,
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
        (self.meta_or_par_blocks_decoded
         + self.data_or_par_blocks_decoded
         + self.blocks_decode_failed) as u64
    }

    fn total_units(&self)        -> u64      { self.total_blocks as u64 }
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let block_size             = ver_to_block_size(self.version);
        let time_elapsed           = (self.end_time - self.start_time) as i64;
        let (hour, minute, second) = time_utils::seconds_to_hms(time_elapsed);

        writeln!(f, "SBX version                        : {}", ver_to_usize(self.version))?;
        writeln!(f, "Block size used in checking        : {}", block_size)?;
        writeln!(f, "Number of blocks processed         : {}", self.units_so_far())?;
        writeln!(f, "Number of blocks sorted (metadata) : {}", self.meta_or_par_blocks_decoded)?;
        writeln!(f, "Number of blocks sorted (data)     : {}", self.data_or_par_blocks_decoded)?;
        writeln!(f, "Number of blocks failed to sort    : {}", self.blocks_decode_failed)?;
        writeln!(f, "Time elapsed                       : {:02}:{:02}:{:02}", hour, minute, second)?;

        Ok(())
    }
}

pub fn sort_file(param : &Param)
                 -> Result<Stats, Error> {
    let (ref_block_pos, ref_block) =
        match block_utils::get_ref_block(&param.in_file,
                                         param.no_meta,
                                         param.silence_level)? {
            None => { return Err(Error::with_message("Failed to find reference block")); },
            Some(x) => x,
        };

    report_ref_block_info(ref_block_pos, &ref_block);

    let metadata = file_utils::get_file_metadata(&param.in_file)?;
    let stats = Arc::new(Mutex::new(Stats::new(&ref_block, &metadata)));

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    let mut block = Block::dummy();

    let reporter = Arc::new(ProgressReporter::new(&stats,
                                                  "SBX block checking progress",
                                                  "bytes",
                                                  param.silence_level));

    let ver_usize = ver_to_usize(ref_block.get_version());

    reporter.start();

    loop {
        let read_res = reader.read(sbx_block::slice_buf_mut(ref_block.get_version(),
                                                            &mut buffer))?;

        break_if_eof_seen!(read_res);
    }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
