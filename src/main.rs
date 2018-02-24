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
use file_reader::FileReader;
mod file_writer;
use file_writer::FileWriter;

mod worker;

const rsbx_ver_str : &str = "1.0";

fn main () {
    let matches = App::new("rsbx")
        .version(rsbx_ver_str)
        .author("Darren Ldl <darrenldldev@gmail.com>")
        .about("Rust implementation of SeqBox")
        .subcommand(SubCommand::with_name("encode")
                    .about("Encode file")
                    .arg(Arg::with_name("INFILE")
                         .required(true)
                         .index(1)
                         .help("File to encode"))
                    .arg(Arg::with_name("OUT")
                         .index(2)
                         .help("Sbx container name (defaults to INFILE.sbx). If OUT is a
directory(DIR), then the final file will be stored as
DIR/INFILE.sbx."))
                    .arg(Arg::with_name("force")
                         .short("f")
                         .long("force")
                         .help("Force overwrite even if OUT exists"))
                    .arg(Arg::with_name("hash")
                         .long("hash")
                         .takes_value(true)
                         .help("Hash function to use, one of (case-insensitive) :
    sha1
    sha256 (default)
    sha512
    blake2b-512"))
                    .arg(Arg::with_name("no-meta")
                         .long("no-meta")
                         .help("Skip metadata block in the sbx container. Metadata block is
never skipped for version 11, 12, 13."))
                    .arg(Arg::with_name("silent")
                         .short("s")
                         .long("silent")
                         .takes_value(true)
                         .help("One of :
    0 (show everything)
    1 (only show progress stats when done)
    2 (show nothing)
This only affects progress text printing."))
                    .arg(Arg::with_name("SBX-VERSION")
                         .long("sbx-version")
                         .takes_value(true)
                         .help("Sbx container version, one of :
    1  (bs=512  bytes)
    2  (bs=128  bytes)
    3  (bs=4096 bytes)
    11 (bs=512  bytes, Reed-Solomon enabled)
    12 (bs=128  bytes, Reed-Solomon enabled)
    13 (bs=4096 bytes, Reed-Solomon enabled)
where bs=sbx block size."))
                    .arg(Arg::with_name("uid")
                         .long("uid")
                         .takes_value(true)
                         .help("Alternative file uid in hex (by default uid is randomly generated).
Uid must be exactly 6 bytes(12 hex digits) in length."))
        )
        .subcommand(SubCommand::with_name("decode")
                    .about("Decode file")
                    .arg(Arg::with_name("INFILE")
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
                    .arg(Arg::with_name("no-meta")
                         .long("no-meta")
                         .help("Use first whatever valid block as reference block. Use this when
the container does not have metadata block or when you are okay
with using a data block as reference block."))
                    .arg(Arg::with_name("silent")
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
        )
        .subcommand(SubCommand::with_name("repair")
                    .about("Repair sbx container")
        )
        .subcommand(SubCommand::with_name("verify")
                    .about("Repair sbx container")
        )
        .get_matches();

    /*use encode_core::Param;
    let param = Param::new(sbx_specs::Version::V11,
                           &[0, 1, 2, 3, 4, 5],
                           10,
                           2,
                           true,
                           multihash::HashType::SHA256,
                           "test",
                           "test.sbx",
                           progress_report::SilenceLevel::L0);
    match encode_core::encode_file(&param) {
        Ok(s)  => print!("{}", s),
        Err(e) => print!("{}", e)
    }*/
    /*use decode_core::Param;
    let param = Param::new(false,
                           "test.sbx",
                           "test2",
                           progress_report::SilenceLevel::L0);
    match decode_core::decode_file(&param) {
        Ok(s)  => print!("{}", s),
        Err(e) => print!("{}", e)
    }*/
}
