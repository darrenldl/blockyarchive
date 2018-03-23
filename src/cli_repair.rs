use repair_core::Param;
use repair_core;
use std::str::FromStr;

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("repair")
        .about("Repair SBX container")
        .arg(in_file_arg()
             .help("SBX container to repair"))
        .arg(pr_verbosity_level_arg())
        .arg(burst_arg())
        .arg(verbose_arg()
             .help("Show reference block info, successes and failures of all required repairs"))
        .arg(Arg::with_name("skip_warning")
             .short("y")
             .long("skip-warning")
             .help("Skip warning about in-place automatic repairs"))
        .arg(Arg::with_name("dry_run")
             .long("dry-run")
             .help("Only do repairs in memory, does not modify anything"))
}

pub fn repair<'a>(matches : &ArgMatches<'a>) -> i32 {
    let in_file = get_in_file!(matches);

    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let burst = get_burst_opt!(matches);

    if matches.is_present("dry_run") {
        print_block!(
            "Note : This is a dry run only, the container is not modified.";
            "";
        );
    }

    if !matches.is_present("skip_warning")
        && !matches.is_present("dry_run")
    {
        print_block!(
            "Warning :";
            "";
            "   Repair mode modifies the SBX container in-place.";
            "";
            "   This may cause further damage to the container and prohibit further";
            "   data recovery if incorrect automatic repairs are made.";
            "";
            "   It is advisable to make a copy of the container and work on the copy";
            "   rather than repairing the original container directly.";
            "";
        );

        ask_if_wish_to_continue!();
    }

    let param = Param::new(in_file,
                           matches.is_present("dry_run"),
                           matches.is_present("verbose"),
                           pr_verbosity_level,
                           burst);
    match repair_core::repair_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok => "{}", s),
        Ok(None)    => exit_with_msg!(ok => ""),
        Err(e)      => exit_with_msg!(op => "{}", e),
    }
}
