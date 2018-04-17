use sort_core::Param;
use std::str::FromStr;

use sort_core;
use file_utils;
use misc_utils;

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("sort")
        .about("Sort SBX blocks in container, can also readjust burst error resistance level")
        .arg(in_file_arg()
             .help("SBX container to sort"))
        .arg(out_file_arg()
             .required(true)
             .help("Sorted SBX container"))
        .arg(Arg::with_name("force")
             .short("f")
             .long("force")
             .help("Force overwrite even if OUT exists"))
        .arg(pr_verbosity_level_arg())
        .arg(burst_arg()
             .help("Burst error resistance level to use for the output container.
Defaults to guessing the level(guesses up to 1000) used by the
original container and uses the result."))
        .arg(verbose_arg()
             .help("Show reference block info"))
}

pub fn sort<'a>(matches : &ArgMatches<'a>) -> i32 {
    let json_enabled = get_json_enabled!(matches);

    let in_file = get_in_file!(matches, json_enabled);
    let out_file = {
        let out_file = matches.value_of("out_file").unwrap();

        if file_utils::check_if_file_is_dir(out_file) {
            misc_utils::make_path(&[out_file, in_file])
        } else {
            String::from(out_file)
        }
    };

    let burst = get_burst_opt!(matches, json_enabled);

    exit_if_file!(exists &out_file
                  => matches.is_present("force")
                  => json_enabled
                  => "File \"{}\" already exists", out_file);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_enabled);

    let param = Param::new(get_ref_block_choice!(matches),
                           json_enabled,
                           in_file,
                           &out_file,
                           matches.is_present("verbose"),
                           pr_verbosity_level,
                           burst);
    match sort_core::sort_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok json_enabled => "{}", s),
        Ok(None)    => exit_with_msg!(ok json_enabled => ""),
        Err(e)      => exit_with_msg!(op json_enabled => "{}", e),
    }
}
