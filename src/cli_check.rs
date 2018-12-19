use check_core;
use check_core::Param;

use clap::*;
use cli_utils::*;

use json_printer::BracketType;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("check")
        .about("Check integrity of SBX blocks in container

===== IMPORTANT =====
Please note that this is the last version of this software to be released under the name rsbx,
future releases will be published under the name blkar. See project repo for details.
=====================")
        .arg(in_file_arg()
             .help("SBX container to check"))
        .arg(no_meta_arg())
        .arg(pr_verbosity_level_arg())
        .arg(Arg::with_name("report_blank")
             .long("report-blank")
             .help("Completely blank blocks are ignored by default.
Specify this if you want rsbx to report blank blocks as well."))
        .arg(verbose_arg()
             .help("Show reference block info, show individual check results"))
        .arg(json_arg())
}

pub fn check<'a>(matches : &ArgMatches<'a>) -> i32 {
    let json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(None, BracketType::Curly);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let in_file  = get_in_file!(matches, json_printer);
    let param = Param::new(get_ref_block_choice!(matches),
                           matches.is_present("report_blank"),
                           &json_printer,
                           in_file,
                           matches.is_present("verbose"),
                           pr_verbosity_level);
    match check_core::check_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok json_printer => "{}", s),
        Ok(None)    => exit_with_msg!(ok json_printer => ""),
        Err(e)      => exit_with_msg!(op json_printer => "{}", e),
    }
}
