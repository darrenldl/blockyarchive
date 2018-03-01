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
        .about("Decode SBX container")
        .arg(Arg::with_name("in_file")
             .value_name("INFILE")
             .required(true)
             .index(1)
             .help("SBX container to decode"))
        .arg(Arg::with_name("out_file")
             .value_name("OUT")
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
        .arg(silence_level_arg())
}

pub fn decode<'a>(matches : &ArgMatches<'a>) -> i32 {
    let no_meta = matches.is_present("no_meta");

    let force_write = matches.is_present("force");

    let silence_level = get_silence_level!(matches);

    let in_file  = matches.value_of("in_file").unwrap();
    exit_if_file!(not_exists in_file => "File \"{}\" does not exist", in_file);
    let out_file = matches.value_of("out_file");

    let param = Param::new(matches.is_present("no_meta"),
                           matches.is_present("force"),
                           in_file,
                           out_file,
                           silence_level);
    match decode_core::decode_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
