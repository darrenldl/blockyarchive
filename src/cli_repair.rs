use super::clap::ArgMatches;
//use super::repair_core::Param;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       Version,
                       string_to_ver,
                       ver_supports_rs,
                       ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use super::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("repair")
        .about("Repair SBX container")
        .arg(Arg::with_name("file")
             .value_name("FILE")
             .required(true)
             .index(1)
             .help("SBX container to repair"))
        .arg(silence_level_arg())
}
