use show_core::Param;
use show_core;
use std::str::FromStr;

use json_printer::BracketType;

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("show")
        .about("Search for and print metadata in file")
        .arg(in_file_arg()
             .help("SBX container to search for metadata"))
        .arg(Arg::with_name("show_all")
             .long("show-all")
             .help("Show all metadata (by default only shows the first one)"))
        .arg(force_misalign_arg())
        .arg(pr_verbosity_level_arg())
        .arg(from_byte_arg()
             .help("Start from byte FROM-BYTE. The position is automatically rounded
down to the closest multiple of 128 bytes. If this option is not
specified, defaults to the start of file. Negative values are rejected.
If FROM-BYTE exceeds the largest possible position (file size - 1),
then it will be treated as (file size - 1). The rounding procedure
is applied after all auto-adjustments."))
        .arg(to_byte_arg())
        .arg(guess_burst_arg())
        .arg(json_arg())
}

pub fn show<'a>(matches : &ArgMatches<'a>) -> i32 {
    let json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(None, BracketType::Curly);

    let in_file = get_in_file!(matches, json_printer);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let from_pos = get_from_pos!(matches, json_printer);
    let to_pos   = get_to_pos!(matches, json_printer);

    let param = Param::new(matches.is_present("show_all"),
                           matches.is_present("guess_burst"),
                           matches.is_present("force_misalign"),
                           &json_printer,
                           from_pos,
                           to_pos,
                           in_file,
                           pr_verbosity_level);
    match show_core::show_file(&param) {
        Ok(s)  => exit_with_msg!(ok json_printer => "{}", s),
        Err(e) => exit_with_msg!(op json_printer => "{}", e)
    }
}
