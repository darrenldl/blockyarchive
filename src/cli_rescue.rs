use rescue_core::Param;
use rescue_core;
use sbx_specs::SBX_FILE_UID_LEN;
use std::str::FromStr;

use sbx_block::BlockType;
use json_printer::BracketType;

use file_utils;

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("rescue")
        .about("Rescue SBX blocks from file/block device, essentially ddrescue but for SBX blocks")
        .arg(in_file_arg()
             .help("File/block device to rescue sbx data from"))
        .arg(out_dir_arg()
             .required(true)
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
        .arg(from_byte_arg()
             .help("Start from byte FROM-BYTE. The position is automatically rounded
down to the closest multiple of 128 bytes, after adding the bytes
processed field from the log file(if specified). If this option is
not specified, defaults to the start of file. Negative values are
rejected. If FROM-BYTE exceeds the largest possible
position(file size - 1), then it will be treated as (file size - 1).
The rounding procedure is applied after all auto-adjustments."))
        .arg(to_byte_arg())
        .arg(json_arg())
}

pub fn rescue<'a>(matches : &ArgMatches<'a>) -> i32 {
    let json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(None, BracketType::Curly);

    let mut temp_uid = [0; SBX_FILE_UID_LEN];
    let uid : Option<&[u8; SBX_FILE_UID_LEN]> = {
        match matches.value_of("uid") {
            None    => None ,
            Some(x) => { parse_uid!(temp_uid, x, json_printer);
                         Some(&temp_uid) }
        }
    };

    let block_type = match matches.value_of("block_type") {
        None    => None,
        Some(x) => {
            match x {
                "any"  => None,
                "meta" => Some(BlockType::Meta),
                "data" => Some(BlockType::Data),
                _      => exit_with_msg!(usr json_printer => "Invalid block type")
            }
        }
    };

    let from_pos = get_from_pos!(matches, json_printer);
    let to_pos   = get_to_pos!(matches, json_printer);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let in_file  = matches.value_of("in_file").unwrap();
    exit_if_file!(not_exists in_file
                  => json_printer
                  => "File \"{}\" does not exist", in_file);
    let out_dir = matches.value_of("out_dir").unwrap();

    if !file_utils::check_if_file_exists(out_dir) {
        exit_with_msg!(usr json_printer => "Directory \"{}\" does not exist", out_dir);
    }
    if !file_utils::check_if_file_is_dir(out_dir) {
        exit_with_msg!(usr json_printer => "\"{}\" is not a directory", out_dir);
    }

    let log_file = matches.value_of("log_file");

    let param = Param::new(in_file,
                           out_dir,
                           log_file,
                           from_pos,
                           to_pos,
                           matches.is_present("force_misalign"),
                           &json_printer,
                           block_type,
                           uid,
                           pr_verbosity_level);
    match rescue_core::rescue_from_file(&param) {
        Ok(s)  => exit_with_msg!(ok json_printer => "{}", s),
        Err(e) => exit_with_msg!(op json_printer => "{}", e)
    }
}
