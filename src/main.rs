#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[macro_use]
extern crate nom;

extern crate clap;
use clap::*;

extern crate chrono;

extern crate pond;

extern crate futures;

extern crate smallvec;

extern crate ctrlc;

extern crate reed_solomon_erasure;

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

macro_rules! break_if_eof_seen {
    (
        $read_res:expr
    ) => {
        if $read_res.eof_seen { break; }
    }
}

mod file_error;

mod general_error;
use general_error::Error;
use general_error::ErrorKind;

mod multihash;
mod multihash_tests;
mod misc_utils;
mod misc_utils_tests;
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

#[macro_use]
mod cli_macros;

mod cli_utils;

mod cli_encode;
mod cli_decode;
mod cli_rescue;
mod cli_show;
mod cli_repair;
mod cli_check;
mod cli_sort;

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
        .subcommand(cli_sort::sub_command())
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
        cli_repair::repair(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("check") {
        cli_check::check(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("sort") {
        cli_sort::sort(matches)
    }
    else {
        exit_with_msg!(ok => "Invoke with -h or --help for help message\n");
    }
}

fn main() {
    std::process::exit(real_main())
}
