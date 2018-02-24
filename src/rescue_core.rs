use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use std::io::SeekFrom;

use super::progress_report::*;
use super::log::*;

use super::file_reader::FileReader;
use super::file_writer::FileWriter;

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
                       ver_first_data_seq_num,
                       ver_supports_rs};

pub struct Stats {
    pub meta_or_par_blocks_processed : u64,
    pub data_or_par_blocks_processed : u64,
    pub bytes_processed              : u64,
    total_bytes                      : u64,
    start_time                       : f64,
    end_time                         : f64,
}

impl Stats {
    pub fn new(param : &Param) -> Stats {
        Stats {
            meta_or_par_blocks_processed : 0,
            data_or_par_blocks_processed : 0,
            bytes_processed              : 0,
            total_bytes                  : 0,
            start_time                   : 0.,
            end_time                     : 0.,
        }
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      { self.bytes_processed }

    fn total_units(&self)        -> u64      { self.total_bytes }
}

impl Log for Stats {
    fn serialise(&self) -> String {
        let mut string = String::with_capacity(200);
        string.push(&format!("bytes_processed={}\n",
                             self.bytes_processed));
        string.push(&format!("blocks_processed={}\n",
                             self.meta_or_par_blocks_processed
                             + self.data_or_par_blocks_processed));
        string.push(&format!("meta_blocks_processed={}\n",
                             self.meta_or_par_blocks_processed));
        string.push(&format!("data_blocks_processed={}\n",
                             self.data_or_par_blocks_processed));
    }

    use nom::digit;
    use std::num::ParseIntError;

    fn deserialise(&mut self, string : &str) -> Result<(), ()> {
        named!(stats_p Result<(u64, u64, u64, u64), ParseIntError>,
               do_parse!(
                   tag!("bytes_processed=") >>
                       bytes_processed  : digit >>
                       tag!("blocks_processed=") >>
                       blocks_processed : digit >>
                       tag!("meta_blocks_processed=") >>
                       meta_or_par_blocks_processed : digit >>
                       tag!("data_blocks_processed=") >>
                       data_or_par_blocks_processed : digit >>
                       (bytes_processed.parse::<u64>()?,
                        blocks_processed.parse::<u64>()?,
                        meta_or_par_blocks_processed.parse::<u64>()?,
                        data_or_par_blocks_processed.parse::<u64>()?)
               )
        );

        match stats_p(string) {
            IResult::Done(_, Ok(bytes, blocks, meta, data)) => {
                self.bytes_processed              = bytes;
                self.blocks_processed             = blocks;
                self.meta_or_par_blocks_processed = meta;
                self.data_or_par_blocks_processed = data;
                Ok(())
            },
            IResult::Done(_, Err(ParseIntError))            => Err(()),
            _                                               => Err(())
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "")
    }
}

pub struct Param {
    in_file  : String,
    out_dir  : String,
    log_file : Option<String>,
    from_pos : Option<u64>,
    to_pos   : Option<u64>,
    silence_level : SilenceLevel,
}

impl Param {
    pub fn new(in_file  : &str,
               out_dir  : &str,
               log_file : Option<&str>,
               from_pos : Option<u64>,
               to_pos   : Option<u64>,
               silence_level : SilenceLevel) -> Param {
        Param {
            in_file  : String::from(in_file),
            out_dir  : String::from(out_file),
            log_file : match log_file {
                None    => None,
                Some(x) => String::From(x),
            },
            from_pos,
            to_pos,
            silence_level,
        }
    }
}

pub fn rescue_from_file(param : &Param)
                        -> Result<Stats, Error> {
    let mut stats = Arc::new(Mutex::new(Stats::new(param)));

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
