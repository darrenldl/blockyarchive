#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
extern crate proptest;

#[macro_use]
extern crate nom;

extern crate clap;

extern crate rand;

extern crate chrono;

extern crate ctrlc;

#[macro_use]
extern crate smallvec;

extern crate reed_solomon_erasure;

extern crate blake2;
extern crate sha1;
extern crate sha2;

extern crate num_cpus;

mod crc_ccitt;

macro_rules! break_if_eof_seen {
    (
        $read_res:expr
    ) => {
        if $read_res.eof_seen {
            break;
        }
    };
}

mod file_error;
mod stdin_error;
mod stdout_error;

mod general_error;
use crate::general_error::Error;
use crate::general_error::ErrorKind;

#[macro_use]
mod json_macros;

#[macro_use]
mod misc_macros;

#[macro_use]
mod cli_macros;

#[macro_use]
mod header_preds;

#[macro_use]
mod block_preds;

mod block_utils;
mod file_utils;
mod file_utils_tests;
mod integer_utils;
mod integer_utils_tests;
pub mod json_printer;
mod json_utils;
mod misc_utils;
mod misc_utils_tests;
mod multihash;
mod multihash_tests;
pub mod output_channel;
mod rand_utils;
mod time_utils;
mod time_utils_tests;

pub mod sbx_block;
mod sbx_container_content;
pub mod sbx_specs;
mod sbx_specs_tests;

mod encode_defaults;

mod log;

mod rs_codec;

mod data_block_buffer;

mod check_core;
mod decode_core;
mod encode_core;
mod repair_core;
mod rescue_core;
mod show_core;
mod sort_core;
mod update_core;

mod progress_report;

mod hash_stats;

mod file_reader;
mod file_writer;
mod reader;
mod writer;

mod cli_utils;

pub mod cli_calc;
pub mod cli_check;
pub mod cli_decode;
pub mod cli_encode;
pub mod cli_repair;
pub mod cli_rescue;
pub mod cli_show;
pub mod cli_sort;
pub mod cli_update;
