use super::repair_core::Param;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       Version,
                       string_to_ver,
                       ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use super::*;
use super::cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("repair")
        .about("Repair SBX container")
        .arg(in_file_arg()
             .help("SBX container to repair"))
        .arg(pr_verbosity_level_arg())
        .arg(burst_arg())
        .arg(verbose_arg()
             .help("Show reference block info, successes and failures of all needed repairs"))
        .arg(Arg::with_name("skip_warning")
             .short("y")
             .long("skip-warning")
             .help("Skip warning about in-place editing that occurs during repairing"))
}

pub fn repair<'a>(matches : &ArgMatches<'a>) -> i32 {
    let in_file = get_in_file!(matches);

    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let burst = get_burst_opt!(matches);

    let param = Param::new(in_file,
                           matches.is_present("verbose"),
                           pr_verbosity_level,
                           burst);
    match repair_core::repair_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
