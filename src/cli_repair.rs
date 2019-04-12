use crate::repair_core;
use crate::repair_core::Param;

use crate::cli_utils::*;
use clap::*;

use crate::json_printer::BracketType;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("repair")
        .about("Repair SBX container")
        .arg(in_file_arg().help("SBX container to repair"))
        .arg(pr_verbosity_level_arg())
        .arg(burst_arg().help(
            "Burst error resistance level used by the container.
Use this if the level used by the container is above 1000,
as blkar will only guess up to 1000. Or use this when blkar
fails to guess correctly.",
        ))
        .arg(
            verbose_arg()
                .help("Show reference block info, successes and failures of all required repairs"),
        )
        .arg(
            Arg::with_name("skip_warning")
                .short("y")
                .long("skip-warning")
                .help("Skip warning about in-place automatic repairs"),
        )
        .arg(dry_run_arg().help("Only do repairs in memory. The container will not be modified."))
        .arg(json_arg().help(
            "Output information in JSON format. Note that blkar does not
guarantee the JSON data to be well-formed if blkar is interrupted.
This also implies --skip-warning, and changes progress report text
(if any) to be in JSON.",
        ))
}

pub fn repair<'a>(matches: &ArgMatches<'a>) -> i32 {
    let json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(None, BracketType::Curly);

    let in_file = get_in_file!(matches, json_printer);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let burst = get_burst_opt!(matches, json_printer);

    if matches.is_present("dry_run") && !json_printer.json_enabled() {
        print_block!(
            "Note : This is a dry run only, the container is not modified";
            "";
        );
    }

    if !matches.is_present("skip_warning")
        && !matches.is_present("dry_run")
        && !json_printer.json_enabled()
    {
        print_block!(
            "Warning :";
            "";
            "    Repair mode modifies the SBX container in-place.";
            "";
            "    This may cause further damage to the container and prohibit further";
            "    data recovery if incorrect automatic repairs are made.";
            "";
            "    It is advisable to make a copy of the container and work on the copy";
            "    rather than repairing the original container directly.";
            "";
        );

        ask_if_wish_to_continue!();
    }

    let param = Param::new(
        in_file,
        matches.is_present("dry_run"),
        &json_printer,
        matches.is_present("verbose"),
        pr_verbosity_level,
        burst,
    );
    match repair_core::repair_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok json_printer => "{}", s),
        Ok(None) => exit_with_msg!(ok json_printer => ""),
        Err(e) => exit_with_msg!(op json_printer => "{}", e),
    }
}
