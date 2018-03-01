#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate nom;

extern crate clap;
use clap::*;

extern crate time;

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

mod progress_report;

mod file_reader;
mod file_writer;

mod worker;

const RSBX_VER_STR : &str = "1.0";

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

mod cli_encode;
mod cli_decode;
mod cli_rescue;
mod cli_show;
mod cli_repair;
mod cli_check;

fn real_main () -> i32 {
    use std::str::FromStr;
    use std::path::Path;

    let matches = App::new("rsbx")
        .version(RSBX_VER_STR)
        .author("Darren Ldl <darrenldldev@gmail.com>")
        .about("Rust implementation of SeqBox")
        .subcommand(cli_encode::sub_command())
        .subcommand(cli_decode::sub_command())
        .subcommand(cli_rescue::sub_command())
        .subcommand(SubCommand::with_name("show")
                    .about("Search for and print metadata in file")
        )
        .subcommand(SubCommand::with_name("repair")
                    .about("Repair sbx container")
        )
        .subcommand(SubCommand::with_name("check")
                    .about("Repair sbx container")
        )
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
        return 0;
    }
    else if let Some(matches) = matches.subcommand_matches("repair") {
        return 0;
    }
    else if let Some(matches) = matches.subcommand_matches("check") {
        return 0;
    }
    else {
        return 0;
    }
}

fn main() {
    std::process::exit(real_main())
}
