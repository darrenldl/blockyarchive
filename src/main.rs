#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[macro_use]
extern crate nom;

extern crate clap;
use clap::*;

extern crate rand;

extern crate chrono;

extern crate pond;

extern crate smallvec;
extern crate ctrlc;

extern crate reed_solomon_erasure;

extern crate ring;
extern crate blake2_c;

mod crc_ccitt;

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

#[macro_use]
mod misc_macros;

#[macro_use]
mod block_preds;

mod multihash;
mod multihash_tests;
mod misc_utils;
mod misc_utils_tests;
mod file_utils;
mod file_utils_tests;
mod rand_utils;
mod time_utils;
mod time_utils_tests;
mod integer_utils;
mod integer_utils_tests;
mod block_utils;

mod sbx_block;
mod sbx_specs;
mod sbx_specs_tests;

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
mod cli_calc;

fn real_main () -> i32 {
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
        .subcommand(cli_calc::sub_command())
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
    else if let Some(matches) = matches.subcommand_matches("calc") {
        cli_calc::calc(matches)
    }
    else {
        exit_with_msg!(ok false => "Invoke with -h or --help for help message\n");
    }
}

fn main() {
    std::process::exit(real_main())
}
