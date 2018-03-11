use super::check_core::Param;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       Version,
                       string_to_ver,
                       ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use super::*;
use super::cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("check")
        .about("Check integrity of SBX blocks in file")
        .arg(Arg::with_name("in_file")
             .value_name("INFILE")
             .required(true)
             .index(1)
             .help("SBX container to check"))
        .arg(no_meta_arg())
        .arg(pr_verbosity_level_arg())
}

pub fn check<'a>(matches : &ArgMatches<'a>) -> i32 {
    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let in_file  = get_in_file!(matches);
    let param = Param::new(matches.is_present("no_meta"),
                           in_file,
                           pr_verbosity_level);
    match check_core::check_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
