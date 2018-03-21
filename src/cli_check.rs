use check_core;
use check_core::Param;

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("check")
        .about("Check integrity of SBX blocks in container")
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
}

pub fn check<'a>(matches : &ArgMatches<'a>) -> i32 {
    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let in_file  = get_in_file!(matches);
    let param = Param::new(get_ref_block_choice!(matches),
                           matches.is_present("report_blank"),
                           in_file,
                           matches.is_present("verbose"),
                           pr_verbosity_level);
    match check_core::check_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok => "{}", s),
        Ok(None)    => exit_with_msg!(ok => ""),
        Err(e)      => exit_with_msg!(op => "{}", e),
    }
}
