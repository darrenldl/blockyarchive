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

use super::file_reader;
use super::file_writer;

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
