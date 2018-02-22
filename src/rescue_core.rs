use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use std::io::SeekFrom;

use super::progress_report::*;

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
    version                          : Version,
    pub meta_or_par_blocks_processed : u64,
    pub data_or_par_blocks_processed : u64,
    start_time                       : f64,
    end_time                         : f64,
}

pub struct Param {
    in_file : String,
    out_dir : String,
    silence_level : SilenceLevel,
}

impl Param {
    pub fn new(in_file : &str,
               out_dir : &str,
               silence_level : SilenceLevel) -> Param {
        Param {
            in_file : String::from(in_file),
            out_dir : String::from(out_file),
            silence_level,
        }
    }
}
