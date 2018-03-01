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
        .subcommand(SubCommand::with_name("encode")
                    .about("Encode file")
                    .arg(Arg::with_name("in_file")
                         .value_name("INFILE")
                         .required(true)
                         .index(1)
                         .help("File to encode"))
                    .arg(Arg::with_name("out_file")
                         .value_name("OUT")
                         .index(2)
                         .help("Sbx container name (defaults to INFILE.sbx). If OUT is a
directory(DIR), then the final file will be stored as
DIR/INFILE.sbx."))
                    .arg(Arg::with_name("force")
                         .short("f")
                         .long("force")
                         .help("Force overwrite even if OUT exists"))
                    .arg(Arg::with_name("hash_type")
                         .value_name("HASH-TYPE")
                         .long("hash")
                         .takes_value(true)
                         .default_value("sha256")
                         .help("Hash function to use, one of (case-insensitive) :
    sha1
    sha256 (default)
    sha512
    blake2b-512"))
                    .arg(Arg::with_name("no_meta")
                         .long("no-meta")
                         .help("Skip metadata block in the sbx container. Metadata block is
never skipped for version 11, 12, 13.
This means this option does nothing for version 11, 12, 13."))
                    .arg(Arg::with_name("silence_level")
                         .value_name("LEVEL")
                         .short("s")
                         .long("silent")
                         .takes_value(true)
                         .help("One of :
    0 (show everything)
    1 (only show progress stats when done)
    2 (show nothing)
This only affects progress text printing."))
                    .arg(Arg::with_name("sbx_version")
                         .value_name("SBX-VERSION")
                         .long("sbx-version")
                         .takes_value(true)
                         .default_value("1")
                         .help("Sbx container version, one of :
    1  (bs=512  bytes)
    2  (bs=128  bytes)
    3  (bs=4096 bytes)
    11 (bs=512  bytes, Reed-Solomon enabled)
    12 (bs=128  bytes, Reed-Solomon enabled)
    13 (bs=4096 bytes, Reed-Solomon enabled)
where bs=sbx block size."))
                    .arg(Arg::with_name("uid")
                         .value_name("UID-HEX")
                         .long("uid")
                         .takes_value(true)
                         .help("Alternative file uid in hex (by default uid is randomly generated).
Uid must be exactly 6 bytes(12 hex digits) in length."))
                    .arg(Arg::with_name("rs_data")
                         .value_name("SHARD")
                         .long("rs-data")
                         .takes_value(true)
                         .help("Reed-Solomon data shard count"))
        )
        .subcommand(SubCommand::with_name("decode")
                    .about("Decode file")
                    .arg(Arg::with_name("in_file")
                         .value_name("INFILE")
                         .required(true)
                         .index(1)
                         .help("Sbx container to decode"))
                    .arg(Arg::with_name("OUT")
                         .index(2)
                         .help("Decoded file name. If OUT is not provided, then name stored in sbx
container is used if present. If OUT is provided and is a
directory(DIR) then output file is stored as DIR/STORED_NAME. If
OUT is provided and is not a directory, then it is used directly."))
                    .arg(Arg::with_name("force")
                         .short("f")
                         .long("force")
                         .help("Force overwrite even if OUT exists"))
                    .arg(Arg::with_name("no_meta")
                         .long("no-meta")
                         .help("Use first whatever valid block as reference block. Use this when
the container does not have metadata block or when you are okay
with using a data block as reference block."))
                    .arg(Arg::with_name("silence_level")
                         .value_name("LEVEL")
                         .short("s")
                         .long("silent")
                         .takes_value(true)
                         .help("One of :
    0 (show everything)
    1 (only show progress stats when done)
    2 (show nothing)
This only affects progress text printing."))
        )
        .subcommand(SubCommand::with_name("rescue")
                    .about("Rescue sbx blocks from file/block device")
                    .arg(Arg::with_name("in_file")
                         .value_name("INFILE")
                         .required(true)
                         .index(1)
                         .help("File/block device to rescue sbx data from"))
                    .arg(Arg::with_name("out_dir")
                         .value_name("OUTDIR")
                         .required(true)
                         .index(2)
                         .help("Directory to store rescued data"))
                    .arg(Arg::with_name("log_file")
                         .value_name("LOGFILE")
                         .index(3)
                         .help("Log file to keep track of the progress to survive interruptions.
Note that you should use the same log file for the same file and
range specified in the initial run."))
                    .arg(Arg::with_name("force_misalign")
                         .long("force-misalign")
                         .help("Disable automatic rounding down of FROM-BYTE. This is not normally
used and is only intended for data recovery or related purposes."))
                    .arg(Arg::with_name("block_type")
                         .value_name("TYPE")
                         .long("only-pick-block")
                         .takes_value(true)
                         .help("Only pick BLOCK-TYPE of blocks, one of :
    any
    meta
    data"))
                    .arg(Arg::with_name("uid")
                         .value_name("UID-HEX")
                         .long("only-pick-uid")
                         .takes_value(true)
                         .help("Only pick blocks with UID-HEX as uid. Uid must be exactly 6
bytes(12 hex digits) in length."))
                    .arg(Arg::with_name("silence_level")
                         .value_name("LEVEL")
                         .short("s")
                         .long("silent")
                         .takes_value(true)
                         .help("One of :
    0 (show everything)
    1 (only show progress stats when done)
    2 (show nothing)
This only affects progress text printing."))
                    .arg(Arg::with_name("from_byte")
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
The rounding procedure is applied after all auto-adjustments."))
                    .arg(Arg::with_name("to_byte")
                         .value_name("TO-BYTE")
                         .long("to")
                         .takes_value(true)
                         .help("Last position to try to decode a block. If not specified, defaults
to the end of file. Negative values are treated as 0. If TO-BYTE is
smaller than FROM-BYTE, then it will be treated as FROM-BYTE."))
        )
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
        use decode_core::Param;
        let param = Param::new(false,
                               "test.sbx",
                               "test2",
                               progress_report::SilenceLevel::L0);
        match decode_core::decode_file(&param) {
            Ok(s)  => exit_with_msg!(ok => "{}", s),
            Err(e) => exit_with_msg!(op => "{}", e)
        }
    }
    else if let Some(matches) = matches.subcommand_matches("rescue") {
        use rescue_core::Param;
        let param = Param::new("test.sbx",
                               "abcd/",
                               Some("rescue_log"),
                               None,
                               None,
                               false,
                               None,
                               None,
                               progress_report::SilenceLevel::L0);
        match rescue_core::rescue_from_file(&param) {
            Ok(s)  => exit_with_msg!(ok => "{}", s),
            Err(e) => exit_with_msg!(op => "{}", e)
        }
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
