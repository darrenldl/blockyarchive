use crate::check_core;
use crate::check_core::HashAction;
use crate::check_core::Param;

use crate::cli_utils::*;
use clap::*;

use crate::json_printer::BracketType;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("check")
        .about("Check integrity of SBX blocks in container")
        .arg(in_file_arg().help("SBX container to check"))
        .arg(no_meta_arg())
        .arg(pr_verbosity_level_arg())
        .arg(Arg::with_name("report_blank").long("report-blank").help(
            "Completely blank blocks are ignored by default.
Specify this if you want blkar to report blank blocks as well.",
        ))
        .arg(verbose_arg().help("Show reference block info, show individual check results"))
        .arg(from_byte_arg().help(FROM_BYTE_ARG_HELP_MSG_REF_BLOCK))
        .arg(to_byte_inc_arg())
        .arg(to_byte_exc_arg())
        .arg(force_misalign_arg())
        .arg(ref_from_byte_arg())
        .arg(ref_to_byte_inc_arg())
        .arg(ref_to_byte_exc_arg())
        .arg(guess_burst_from_byte_arg())
        .arg(
            Arg::with_name("hash")
                .long("hash")
                .help(
                    "Hash stored data after individual block checking. This is done
only if the reference block is a metadata block and has the hash
field.",
                )
                .conflicts_with("from_pos")
                .conflicts_with("to_pos_inc")
                .conflicts_with("to_pos_exc"),
        )
        .arg(
            Arg::with_name("hash_only")
                .long("hash-only")
                .help(
                    "Hash stored data and skip individual block checking. This is
done only if the reference block is a metadata block and has
the hash field.",
                )
                .conflicts_with("from_pos")
                .conflicts_with("to_pos_inc")
                .conflicts_with("to_pos_exc")
                .conflicts_with("hash"),
        )
        .arg(json_arg())
}

pub fn check<'a>(matches: &ArgMatches<'a>) -> i32 {
    let json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(None, BracketType::Curly);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let in_file = get_in_file!(matches, json_printer);

    let from_pos = get_from_pos!(matches, json_printer);
    let to_pos = get_to_pos!(matches, json_printer);

    let ref_from_pos = get_ref_from_pos!(matches, json_printer);
    let ref_to_pos = get_ref_to_pos!(matches, json_printer);

    let guess_burst_from_pos = get_guess_burst_from_pos!(matches, json_printer);

    let hash_action = if matches.is_present("hash") {
        HashAction::HashAfterCheck
    } else if matches.is_present("hash_only") {
        HashAction::HashOnly
    } else {
        HashAction::NoHash
    };

    let burst = get_burst_opt!(matches, json_printer);

    let param = Param::new(
        get_ref_block_choice!(matches),
        ref_from_pos,
        ref_to_pos,
        guess_burst_from_pos,
        matches.is_present("report_blank"),
        &json_printer,
        from_pos,
        to_pos,
        matches.is_present("force_misalign"),
        hash_action,
        burst,
        in_file,
        matches.is_present("verbose"),
        pr_verbosity_level,
    );
    match check_core::check_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok json_printer => "{}", s),
        Ok(None) => exit_with_msg!(ok json_printer => ""),
        Err(e) => exit_with_msg!(op json_printer => "{}", e),
    }
}
