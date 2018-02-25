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
use super::sbx_specs::SBX_FILE_UID_LEN;
use super::sbx_specs::{ver_to_block_size,
                       ver_to_data_size,
                       ver_first_data_seq_num,
                       ver_supports_rs};
use nom::digit;
use std::num::ParseIntError;
use super::integer_utils::IntegerUtils;

pub struct Param {
    in_file         : String,
    out_dir         : String,
    log_file        : Option<String>,
    from_pos        : Option<u64>,
    to_pos          : Option<u64>,
    force_misalign  : bool,
    only_pick_block : Option<BlockType>,
    only_pick_uid   : [u8; SBX_FILE_UID_LEN],
    silence_level   : SilenceLevel,
}

impl Param {
    pub fn new(in_file         : &str,
               out_dir         : &str,
               log_file        : Option<&str>,
               from_pos        : Option<u64>,
               to_pos          : Option<u64>,
               force_misalign  : bool,
               only_pick_block : Option<BlockType>,
               only_pick_uid   : &[u8; SBX_FILE_UID_LEN],
               silence_level   : SilenceLevel) -> Param {
        Param {
            in_file  : String::from(in_file),
            out_dir  : String::from(out_dir),
            log_file : match log_file {
                None    => None,
                Some(x) => Some(String::from(x)),
            },
            from_pos,
            to_pos,
            force_misalign,
            only_pick_block,
            only_pick_uid : only_pick_uid.clone(),
            silence_level,
        }
    }
}

#[derive(Clone)]
pub struct Stats {
    pub meta_or_par_blocks_processed : u64,
    pub data_or_par_blocks_processed : u64,
    pub bytes_processed              : u64,
    total_bytes                      : u64,
    start_time                       : f64,
    end_time                         : f64,
}

impl Stats {
    pub fn new(param         : &Param,
               file_metadata : &fs::Metadata)
               -> Result<Stats, Error> {
        let mut stats = Stats {
            meta_or_par_blocks_processed : 0,
            data_or_par_blocks_processed : 0,
            bytes_processed              : 0,
            total_bytes                  : file_metadata.len(),
            start_time                   : 0.,
            end_time                     : 0.,
        };
        match param.log_file {
            None        => {},
            Some(ref x) => stats.read_from(x)?,
        }
        Ok(stats)
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      { self.bytes_processed }

    fn total_units(&self)        -> u64      { self.total_bytes }
}

impl Log for Stats {
    fn serialize(&self) -> String {
        let mut string = String::with_capacity(200);
        string.push_str(&format!("bytes_processed={}\n",
                                 self.bytes_processed));
        string.push_str(&format!("blocks_processed={}\n",
                                 self.meta_or_par_blocks_processed
                                 + self.data_or_par_blocks_processed));
        string.push_str(&format!("meta_blocks_processed={}\n",
                                 self.meta_or_par_blocks_processed));
        string.push_str(&format!("data_blocks_processed={}\n",
                                 self.data_or_par_blocks_processed));

        string
    }

    fn deserialize(&mut self, input : &[u8]) -> Result<(), ()> {
        use nom::IResult;

        fn stats_p_helper(bytes  : &[u8],
                          blocks : &[u8],
                          meta   : &[u8],
                          data   : &[u8])
                          -> Result<(u64, u64, u64, u64), ParseIntError> {
            use std::str::from_utf8;

            let bytes  = from_utf8(bytes).unwrap();
            let blocks = from_utf8(blocks).unwrap();
            let meta   = from_utf8(meta).unwrap();
            let data   = from_utf8(data).unwrap();

            Ok((bytes.parse::<u64>()?,
                blocks.parse::<u64>()?,
                meta.parse::<u64>()?,
                data.parse::<u64>()?))
        }

        named!(stats_p <Result<(u64, u64, u64, u64), ParseIntError>>,
               do_parse!(
                   _id1 : tag!("bytes_processed=") >>
                       bytes  : digit >>
                       _id2   : tag!("blocks_processed=") >>
                       blocks : digit >>
                       _id3   : tag!("meta_blocks_processed=") >>
                       meta   : digit >>
                       _id4   : tag!("data_blocks_processed=") >>
                       data   : digit >>
                       (stats_p_helper(bytes, blocks, meta, data))
               )
        );

        match stats_p(input) {
            IResult::Done(_, Ok((bytes, _, meta, data))) => {
                self.bytes_processed              = bytes;
                self.meta_or_par_blocks_processed = meta;
                self.data_or_par_blocks_processed = data;
                Ok(())
            },
            IResult::Done(_, Err(_))                     => Err(()),
            _                                            => Err(())
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Number of bytes processed             : {}", self.bytes_processed)?;
        writeln!(f, "Number of blocks processed            : {}",
                 self.meta_or_par_blocks_processed
                 + self.data_or_par_blocks_processed)?;
        writeln!(f, "Number of blocks processed (metadata) : {}", self.meta_or_par_blocks_processed)?;
        writeln!(f, "Number of blocks processed (data)     : {}", self.data_or_par_blocks_processed)
    }
}

pub fn rescue_from_file(param : &Param)
                        -> Result<Stats, Error> {
    let metadata = file_utils::get_file_metadata(&param.in_file)?;
    let mut stats = Arc::new(Mutex::new(Stats::new(param, &metadata)?));

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
