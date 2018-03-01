use super::clap::ArgMatches;
use super::rescue_core::Param;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       Version,
                       string_to_ver,
                       ver_supports_rs,
                       ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use super::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("rescue")
        .about("Rescue SBX blocks from file/block device")
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
}

pub fn rescue<'a>(matches : &ArgMatches<'a>) -> i32 {
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
