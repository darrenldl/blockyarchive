use super::clap::ArgMatches;
use super::decode_core::Param;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       Version,
                       string_to_ver,
                       ver_supports_rs,
                       ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use super::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("decode")
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
}

pub fn decode<'a>(matches : &ArgMatches<'a>) -> i32 {
    let param = Param::new(false,
                           "test.sbx",
                           "test2",
                           progress_report::SilenceLevel::L0);
    match decode_core::decode_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
