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
        .arg(Arg::with_name("in_file")
             .value_name("INFILE")
             .required(true)
             .index(1)
             .help("SBX container to repair"))
        .arg(pr_verbosity_level_arg())
        .arg(burst_arg())
        .arg(verbose_arg()
             .help("Show info(successes and failures) about all needed repairs"))
}

pub fn repair<'a>(matches : &ArgMatches<'a>) -> i32 {
    let in_file = matches.value_of("in_file").unwrap();
    exit_if_file!(not_exists in_file => "File \"{}\" does not exist", in_file);

    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let burst =
        match matches.value_of("burst") {
            None    => None,
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => Some(x),
                    Err(_) => {
                        exit_with_msg!(usr => "Failed to parse burst error resistance level");
                    }
                }
            }
        };

    let param = Param::new(in_file,
                           matches.is_present("verbose"),
                           pr_verbosity_level,
                           burst);
    match repair_core::repair_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
