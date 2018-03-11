use super::rescue_core::Param;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       Version,
                       string_to_ver,
                       ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use super::*;
use super::cli_utils::*;

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
        .arg(force_misalign_arg())
        .arg(pr_verbosity_level_arg())
        .arg(from_byte_arg())
        .arg(to_byte_arg())
}

pub fn rescue<'a>(matches : &ArgMatches<'a>) -> i32 {
    use sbx_block::BlockType;
    let mut temp_uid = [0; SBX_FILE_UID_LEN];
    let uid : Option<&[u8; SBX_FILE_UID_LEN]> = {
        match matches.value_of("uid") {
            None    => None ,
            Some(x) => { parse_uid!(temp_uid, x); Some(&temp_uid) }
        }
    };

    let block_type = match matches.value_of("block_type") {
        None    => None,
        Some(x) => {
            match x {
                "any"  => None,
                "meta" => Some(BlockType::Meta),
                "data" => Some(BlockType::Data),
                _      => exit_with_msg!(usr => "Invalid block type")
            }
        }
    };

    let from_pos = match matches.value_of("from_pos") {
        None    => None,
        Some(x) => match u64::from_str(x) {
            Ok(x)  => Some(x),
            Err(_) => exit_with_msg!(usr => "Invalid from position")
        }
    };

    let to_pos = match matches.value_of("to_pos") {
        None    => None,
        Some(x) => match u64::from_str(x) {
            Ok(x)  => Some(x),
            Err(_) => exit_with_msg!(usr => "Invalid to position")
        }
    };

    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let in_file  = matches.value_of("in_file").unwrap();
    exit_if_file!(not_exists in_file => "File \"{}\" does not exist", in_file);
    let out_dir = matches.value_of("out_dir").unwrap();

    if !file_utils::check_if_file_exists(out_dir) {
        exit_with_msg!(usr => "Directory \"{}\" does not exist", out_dir);
    }
    if !file_utils::check_if_file_is_dir(out_dir) {
        exit_with_msg!(usr => "\"{}\" is not a directory", out_dir);
    }

    let log_file = matches.value_of("log_file");

    let param = Param::new(in_file,
                           out_dir,
                           log_file,
                           from_pos,
                           to_pos,
                           matches.is_present("force_misalign"),
                           block_type,
                           uid,
                           pr_verbosity_level);
    match rescue_core::rescue_from_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
