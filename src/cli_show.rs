use super::show_core::Param;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       Version,
                       string_to_ver,
                       ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use super::*;
use super::cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("show")
        .about("Search for and print metadata in file")
        .arg(Arg::with_name("in_file")
             .value_name("INFILE")
             .required(true)
             .index(1)
             .help("SBX container to search for metadata"))
        .arg(Arg::with_name("show_all")
             .long("show-all")
             .help("Show all metadata (by default only shows the first one)"))
        .arg(force_misalign_arg())
        .arg(pr_verbosity_level_arg())
        .arg(from_byte_arg())
        .arg(to_byte_arg())
        .arg(guess_burst_arg())
}

pub fn show<'a>(matches : &ArgMatches<'a>) -> i32 {
    let in_file = matches.value_of("in_file").unwrap();
    exit_if_file!(not_exists in_file => "File \"{}\" does not exist", in_file);

    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let from_pos = get_from_pos!(matches);
    let to_pos   = get_to_pos!(matches);

    let param = Param::new(matches.is_present("show_all"),
                           matches.is_present("guess_burst"),
                           matches.is_present("force_misalign"),
                           from_pos,
                           to_pos,
                           in_file,
                           pr_verbosity_level);
    match show_core::show_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
