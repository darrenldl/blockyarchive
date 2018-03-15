use decode_core::Param;
use decode_core;
use sbx_specs::{SBX_FILE_UID_LEN,
                Version,
                string_to_ver,
                ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("decode")
        .about("Decode SBX container")
        .arg(in_file_arg()
             .help("SBX container to decode"))
        .arg(out_arg()
             .help("Decoded file name. If OUT is not provided, then name stored in sbx
container is used if present. If OUT is provided and is a
directory(DIR) then output file is stored as DIR/STORED_NAME. If
OUT is provided and is not a directory, then it is used directly."))
        .arg(Arg::with_name("force")
             .short("f")
             .long("force")
             .help("Force overwrite even if OUT exists"))
        .arg(no_meta_arg())
        .arg(pr_verbosity_level_arg())
        .arg(verbose_arg()
             .help("Show reference block info"))
}

pub fn decode<'a>(matches : &ArgMatches<'a>) -> i32 {
    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let in_file = get_in_file!(matches);
    let out     = matches.value_of("out");

    let param = Param::new(matches.is_present("no_meta"),
                           matches.is_present("force"),
                           in_file,
                           out,
                           matches.is_present("verbose"),
                           pr_verbosity_level);
    match decode_core::decode_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok => "{}", s),
        Ok(None)    => exit_with_msg!(ok => ""),
        Err(e)      => exit_with_msg!(op => "{}", e),
    }
}
