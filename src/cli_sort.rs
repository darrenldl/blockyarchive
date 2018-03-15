use sort_core::Param;
use sbx_specs::{SBX_FILE_UID_LEN,
                Version,
                string_to_ver,
                ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use sort_core;
use file_utils;
use misc_utils;

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("sort")
        .about("Sort SBX blocks in container")
        .arg(in_file_arg()
             .help("SBX container to sort"))
        .arg(out_file_arg()
             .required(true)
             .help("Sorted SBX container"))
        .arg(Arg::with_name("force")
             .short("f")
             .long("force")
             .help("Force overwrite even if OUT exists"))
        .arg(no_meta_arg())
        .arg(pr_verbosity_level_arg())
        .arg(from_byte_arg())
        .arg(to_byte_arg())
        .arg(burst_arg())
        .arg(verbose_arg()
             .help("Show reference block info"))
}

pub fn sort<'a>(matches : &ArgMatches<'a>) -> i32 {
    let in_file = get_in_file!(matches);
    let out_file = {
        let out_file = matches.value_of("out_file").unwrap();

        if file_utils::check_if_file_is_dir(out_file) {
            misc_utils::make_path(&[out_file, in_file])
        } else {
            String::from(out_file)
        }
    };

    let burst = get_burst_opt!(matches);

    exit_if_file!(exists &out_file
                  => matches.is_present("force")
                  => "File \"{}\" already exists", out_file);

    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let param = Param::new(matches.is_present("no-meta"),
                           in_file,
                           &out_file,
                           matches.is_present("verbose"),
                           pr_verbosity_level,
                           burst);
    match sort_core::sort_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok => "{}", s),
        Ok(None)    => exit_with_msg!(ok => ""),
        Err(e)      => exit_with_msg!(op => "{}", e),
    }
}
