use crate::sbx_specs::SBX_FILE_UID_LEN;
use crate::show_core;
use crate::show_core::Param;

use crate::json_printer::BracketType;

use crate::cli_utils::*;
use clap::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("show")
        .about("Search for and print metadata in file")
        .arg(in_file_arg().help("SBX container to search for metadata"))
        .arg(
            Arg::with_name("show_all")
                .long("show-all")
                .help("Show all metadata (by default only shows the first one)"),
        )
        .arg(only_pick_uid_arg())
        .arg(force_misalign_arg())
        .arg(pr_verbosity_level_arg())
        .arg(from_byte_arg().help(FROM_BYTE_ARG_HELP_MSG_SCAN))
        .arg(to_byte_inc_arg())
        .arg(to_byte_exc_arg())
        .arg(guess_burst_arg())
        .arg(json_arg())
}

pub fn show<'a>(matches: &ArgMatches<'a>) -> i32 {
    let json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(None, BracketType::Curly);

    let in_file = get_in_file!(matches, json_printer);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let from_pos = get_from_pos!(matches, json_printer);
    let to_pos = get_to_pos!(matches, json_printer);

    let mut temp_uid = [0; SBX_FILE_UID_LEN];
    let uid: Option<&[u8; SBX_FILE_UID_LEN]> = get_uid!(matches, temp_uid, json_printer);

    let param = Param::new(
        matches.is_present("show_all"),
        matches.is_present("guess_burst"),
        matches.is_present("force_misalign"),
        &json_printer,
        from_pos,
        to_pos,
        in_file,
        uid,
        pr_verbosity_level,
    );
    match show_core::show_file(&param) {
        Ok(s) => exit_with_msg!(ok json_printer => "{}", s),
        Err(e) => exit_with_msg!(op json_printer => "{}", e),
    }
}
