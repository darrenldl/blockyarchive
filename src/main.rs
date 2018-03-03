#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate nom;

extern crate clap;
use clap::*;

extern crate chrono;

extern crate pond;

extern crate futures;

extern crate smallvec;

extern crate ctrlc;

//#[macro_use]
extern crate reed_solomon_erasure;

#[macro_use]
mod worker_macros;

macro_rules! smallvec {
    [
        $arr:ty => $val:expr; $len:expr
    ] => {{
        let mut v : SmallVec<$arr> = SmallVec::with_capacity($len);
        for _ in 0..$len {
            v.push($val);
        }
        v
    }};
    [
        $val:expr; $len:expr
    ] => {{
        let mut v = SmallVec::with_capacity($len);
        for _ in 0..$len {
            v.push($val);
        }
        v
    }}
}

mod file_error;

mod general_error;
use general_error::Error;
use general_error::ErrorKind;

mod multihash;
mod multihash_test;
mod misc_utils;
mod misc_utils_test;
mod file_utils;
mod rand_utils;
mod time_utils;
mod integer_utils;
mod block_utils;

mod sbx_block;
mod sbx_specs;

mod log;

mod rs_codec;

mod encode_core;
mod decode_core;
mod rescue_core;
mod repair_core;
mod show_core;
mod sort_core;
mod check_core;

mod progress_report;

mod file_reader;
use file_reader::ReadResult;
mod file_writer;

macro_rules! exit_with_msg {
    (
        ok => $($x:expr),*
    ) => {{
        print!($($x),*);
        return 0;
    }};
    (
        usr => $($x:expr),*
    ) => {{
        println!($($x),*);
        return 1;
    }};
    (
        op => $($x:expr),*
    ) => {{
        print!($($x),*);
        return 2;
    }}
}

macro_rules! exit_if_file {
    (
        exists $file:expr => $($x:expr),*
    ) => {{
        if file_utils::check_if_file_exists($file) {
            exit_with_msg!(usr => $($x),*);
        }
    }};
    (
        not_exists $file:expr => $($x:expr),*
    ) => {{
        if !file_utils::check_if_file_exists($file) {
            exit_with_msg!(usr => $($x),*);
        }
    }}
}

macro_rules! get_silence_level {
    (
        $matches:expr
    ) => {{
        match $matches.value_of("silence_level") {
            None    => progress_report::SilenceLevel::L0,
            Some(x) => match progress_report::string_to_silence_level(x) {
                Ok(x)  => x,
                Err(_) => exit_with_msg!(usr => "Invalid silence level")
            }
        }
    }}
}

macro_rules! parse_uid {
    (
        $buf:expr, $uid:expr
    ) => {{
        use misc_utils::HexError::*;
        match misc_utils::hex_string_to_bytes($uid) {
            Ok(x) => {
                if x.len() != SBX_FILE_UID_LEN {
                    exit_with_msg!(usr => "UID must be {} bytes({} hex characters) in length",
                                   SBX_FILE_UID_LEN,
                                   SBX_FILE_UID_LEN * 2);
                }

                $buf.copy_from_slice(&x);
            },
            Err(InvalidHexString) => {
                exit_with_msg!(usr => "UID provided is not a valid hex string");
            },
            Err(InvalidLen) => {
                exit_with_msg!(usr => "UID provided does not have the correct number of hex digits, provided : {}, need : {}",
                               $uid.len(),
                               SBX_FILE_UID_LEN * 2);
            }
        }
    }}
}

mod cli_encode;
mod cli_decode;
mod cli_rescue;
mod cli_show;
mod cli_repair;
mod cli_check;
mod cli_sort;

fn silence_level_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("silence_level")
    .value_name("LEVEL")
    .short("s")
    .long("silent")
    .takes_value(true)
    .help("One of :
    0 (show everything)
    1 (only show progress stats when done)
    2 (show nothing)
This only affects progress text printing.")
}

fn force_misalign_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("force_misalign")
        .long("force-misalign")
        .help("Disable automatic rounding down of FROM-BYTE. This is not normally
used and is only intended for data recovery or related purposes.")
}

fn from_byte_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("from_pos")
        .value_name("FROM-BYTE")
        .long("from")
        .visible_alias("skip-to")
        .takes_value(true)
        .help("Start from byte FROM-BYTE. The position is automatically rounded
down to the closest multiple of 128 bytes, after adding the bytes
processed field from the log file(if specified). If this option is
not specified, defaults to the start of file. Negative values are
treated as 0. If FROM-BYTE exceeds the largest possible
position(file size - 1), then it will be treated as (file size - 1).
The rounding procedure is applied after all auto-adjustments.")
}

fn to_byte_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("to_pos")
        .value_name("TO-BYTE")
        .long("to")
        .takes_value(true)
        .help("Last position to try to decode a block. If not specified, defaults
to the end of file. Negative values are treated as 0. If TO-BYTE is
smaller than FROM-BYTE, then it will be treated as FROM-BYTE.")
}

fn no_meta_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("no_meta")
        .long("no-meta")
        .help("Use first whatever valid block as reference block. Use this when
the container does not have metadata block or when you are okay
with using a data block as reference block.")
}

fn report_ref_block_info(ref_block_pos : u64,
                         ref_block     : &sbx_block::Block) {
    println!();
    println!("Using {} block as reference block, located at byte {} (0x{:X})",
             if ref_block.is_meta() { "metadata" }
             else                   { "data"     },
             ref_block_pos,
             ref_block_pos);
    println!();
}

fn real_main () -> i32 {
    use std::str::FromStr;
    use std::path::Path;

    let matches = App::new("rsbx")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Darren Ldl <darrenldldev@gmail.com>")
        .about("Rust implementation of SeqBox")
        .subcommand(cli_encode::sub_command())
        .subcommand(cli_decode::sub_command())
        .subcommand(cli_rescue::sub_command())
        .subcommand(cli_show::sub_command())
        .subcommand(cli_repair::sub_command())
        .subcommand(cli_check::sub_command())
        .get_matches();

    if      let Some(matches) = matches.subcommand_matches("encode") {
        cli_encode::encode(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("decode") {
        cli_decode::decode(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("rescue") {
        cli_rescue::rescue(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("show") {
        cli_show::show(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("repair") {
        return 0;
    }
    else if let Some(matches) = matches.subcommand_matches("check") {
        cli_check::check(matches)
    }
    else {
        exit_with_msg!(ok => "Invoke with -h or --help for help message");
    }
}

fn main() {
    std::process::exit(real_main())
}
