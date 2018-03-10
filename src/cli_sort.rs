use super::clap::ArgMatches;
use super::sort_core::Param;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       Version,
                       string_to_ver,
                       ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use super::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("sort")
        .about("Sort sbx blocks")
        .arg(Arg::with_name("in_file")
             .value_name("INFILE")
             .required(true)
             .index(1)
             .help("File to sort"))
        .arg(Arg::with_name("in_file")
             .value_name("OUTFILE")
             .required(true)
             .index(1)
             .help("File for storing the sorted blocks"))
        .arg(no_meta_arg())
        .arg(silence_level_arg())
        .arg(from_byte_arg())
        .arg(to_byte_arg())
        .arg(burst_arg())
}

pub fn sort<'a>(matches : &ArgMatches<'a>) -> i32 {
    let in_file = matches.value_of("in_file").unwrap();
    exit_if_file!(not_exists in_file => "File \"{}\" does not exist", in_file);

    let out_file = {
        let out_file = matches.value_of("out_file").unwrap();

        if file_utils::check_if_file_is_dir(out_file) {
            misc_utils::make_path(&[out_file, in_file])
        } else {
            String::from(out_file)
        }
    };

    let burst =
        match matches.value_of("burst") {
            None    => None,
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => Some(x),
                    Err(_) => {
                        exit_with_msg!(usr => "Failed to parse burst error resistance level");
                    }
                }
            }
        };

    let force_write = matches.is_present("force");

    if !force_write {
        exit_if_file!(exists &out_file => "File \"{}\" already exists", out_file);
    }

    let silence_level = get_silence_level!(matches);

    let param = Param::new(matches.is_present("no-meta"),
                           in_file,
                           &out_file,
                           silence_level,
                           burst);
    match sort_core::sort_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
