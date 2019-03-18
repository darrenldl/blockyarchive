use crate::encode_core;
use crate::encode_core::Param;
use crate::sbx_specs::{
    ver_to_block_size, ver_to_data_size, ver_to_usize, ver_uses_rs, Version, SBX_FILE_UID_LEN,
};
use std::str::FromStr;
use std::time::UNIX_EPOCH;
use std::sync::Arc;
use crate::output_channel::OutputChannel;

use crate::json_printer::BracketType;

use crate::multihash;

use crate::file_utils;
use crate::misc_utils;
use crate::rand_utils;
use crate::time_utils;

use crate::cli_utils::*;
use clap::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("encode")
        .about("Encode file")
        .arg(
            in_file_arg()
                .help("File to encode. Supply - to use stdin as input. Use ./- for files named -."),
        )
        .arg(out_arg().help(
            "SBX container name (defaults to INFILE.sbx). If OUT is a directory, then the
container is stored as OUT/INFILE.sbx (only the file part of INFILE is used).",
        ))
        .arg(force_arg().help("Force overwrite even if OUT exists"))
        .arg(
            Arg::with_name("hash_type")
                .value_name("HASH-TYPE")
                .long("hash")
                .takes_value(true)
                .help(
                    "Hash function to use, one of (case-insensitive) :
          sha1
(default) sha256
          sha512
          blake2b-512",
                ),
        )
        .arg(Arg::with_name("no_meta").long("no-meta").help(
            "Skip metadata block in the SBX container. Metadata block is
never skipped for version 17, 18, 19.
This means this option has no effect for version 17, 18, 19.",
        ))
        .arg(pr_verbosity_level_arg())
        .arg(sbx_version_arg())
        .arg(only_pick_uid_arg().long("uid").help(
            "Alternative file UID in hex (by default UID is randomly generated).
UID must be exactly 6 bytes (12 hex digits) in length.",
        ))
        .arg(rs_data_arg())
        .arg(rs_parity_arg())
        .arg(from_byte_arg().help(FROM_BYTE_ARG_HELP_MSG_RAW_UNALIGNED))
        .arg(to_byte_inc_arg())
        .arg(to_byte_exc_arg())
        .arg(burst_arg().help(
            "Burst error resistance level. Note that blkar only guesses up to
1000 in repair, show, and sort mode. If you use level above 1000,
then blkar will make an incorrect guess, and you will need to
specify it explicitly in repair and sort mode. Show mode does
not rely on burst level, but provides an option for enabling
automatic guessing.",
        ))
        .arg(
            Arg::with_name("info_only")
                .long("info-only")
                .help("Only display information about encoding then exit"),
        )
        .arg(json_arg())
}

pub fn encode<'a>(matches: &ArgMatches<'a>) -> i32 {
    let mut json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(None, BracketType::Curly);

    // compute uid
    let mut uid: [u8; SBX_FILE_UID_LEN] = [0; SBX_FILE_UID_LEN];
    {
        match matches.value_of("uid") {
            None => {
                rand_utils::fill_random_bytes(&mut uid);
            }
            Some(x) => {
                parse_uid!(uid, x, json_printer);
            }
        }
    }

    let version = get_version!(matches, json_printer);

    let data_par_burst = if ver_uses_rs(version) {
        // deal with RS related options
        let data_shards = get_data_shards!(matches, version, json_printer);
        let parity_shards = get_parity_shards!(matches, version, json_printer);

        check_data_parity_shards!(data_shards, parity_shards, json_printer);

        let burst = get_burst_or_zero!(matches, json_printer);

        Some((data_shards, parity_shards, burst))
    } else {
        None
    };

    let in_file = get_in_file!(accept_stdin matches, json_printer);
    let out = match matches.value_of("out") {
        None => {
            if file_utils::check_if_file_is_stdin(in_file) {
                exit_with_msg!(usr json_printer => "Explicit output file name is required when input is stdin");
            } else {
                Some(format!("{}.sbx", in_file))
            }
        }
        Some(x) => {
            if file_utils::check_if_file_is_dir(x) {
                if file_utils::check_if_file_is_stdin(in_file) {
                    exit_with_msg!(usr json_printer => "Explicit output file name is required when input is stdin");
                }

                let in_file = file_utils::get_file_name_part_of_path(in_file);
                Some(misc_utils::make_path(&[x, &format!("{}.sbx", in_file)]))
            } else if file_utils::check_if_file_is_stdout(x) {
                Arc::get_mut(&mut json_printer)
                    .unwrap()
                    .set_output_channel(OutputChannel::Stderr);
                None
            } else {
                Some(String::from(x))
            }
        }
    };

    let hash_type = match matches.value_of("hash_type") {
        None => multihash::HashType::SHA256,
        Some(x) => match multihash::string_to_hash_type(x) {
            Ok(x) => x,
            Err(_) => exit_with_msg!(usr json_printer => "Invalid hash type"),
        },
    };

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let meta_enabled = get_meta_enabled!(matches);

    let from_pos = get_from_pos!(matches, json_printer);
    let to_pos = get_to_pos!(matches, json_printer);

    if matches.is_present("info_only") {
        json_printer.print_open_bracket(Some("stats"), BracketType::Curly);

        if file_utils::check_if_file_is_stdin(in_file) {
            exit_with_msg!(usr json_printer => "No information is available for stdin input");
        }

        let in_file_meta = match file_utils::get_file_metadata(in_file) {
            Ok(x) => x,
            Err(_) => exit_with_msg!(usr json_printer => "Failed to get metadata of \"{}\"",
                                     in_file),
        };

        let in_file_size = match file_utils::get_file_size(in_file) {
            Ok(x) => x,
            Err(_) => exit_with_msg!(usr json_printer => "Failed to get file size of \"{}\"",
                                     in_file),
        };

        let in_file_mod_time = match in_file_meta.modified() {
            Ok(t) => match t.duration_since(UNIX_EPOCH) {
                Ok(t) => Some(t.as_secs() as i64),
                Err(_) => None,
            },
            Err(_) => None,
        };

        let in_file_mod_time_str = match in_file_mod_time {
            None => "N/A".to_string(),
            Some(x) => match (
                time_utils::i64_secs_to_date_time_string(x, time_utils::TimeMode::UTC),
                time_utils::i64_secs_to_date_time_string(x, time_utils::TimeMode::Local),
            ) {
                (Some(u), Some(l)) => format!("{} (UTC)  {} (Local)", u, l),
                _ => "Invalid file modification time".to_string(),
            },
        };

        let out_name = match out {
            None => "N/A",
            Some(ref f) => f,
        };

        let out_file_size = file_utils::from_orig_file_size::calc_container_size(
            version,
            Some(meta_enabled),
            data_par_burst,
            in_file_size,
        );

        if ver_uses_rs(version) {
            print_maybe_json!(json_printer, "File name                    : {}", in_file);
            print_maybe_json!(json_printer, "SBX container name           : {}", out_name);
            print_maybe_json!(
                json_printer,
                "SBX container version        : {}",
                ver_to_usize(version)
            );
            print_maybe_json!(json_printer, "SBX container block size     : {}", ver_to_block_size(version) => skip_quotes);
            print_maybe_json!(json_printer, "SBX container data  size     : {}", ver_to_data_size(version)  => skip_quotes);
            print_maybe_json!(json_printer, "RS data   shard count        : {}", data_par_burst.unwrap().0  => skip_quotes);
            print_maybe_json!(json_printer, "RS parity shard count        : {}", data_par_burst.unwrap().1  => skip_quotes);
            print_maybe_json!(json_printer, "Burst error resistance level : {}", data_par_burst.unwrap().2  => skip_quotes);
            print_maybe_json!(json_printer, "File size                    : {}", in_file_size               => skip_quotes);
            print_maybe_json!(json_printer, "SBX container size           : {}", out_file_size              => skip_quotes);
            print_maybe_json!(
                json_printer,
                "File modification time       : {}",
                in_file_mod_time_str
            );
        } else {
            print_maybe_json!(json_printer, "File name                : {}", in_file);
            print_maybe_json!(json_printer, "SBX container name       : {}", out_name);
            print_maybe_json!(
                json_printer,
                "SBX container version    : {}",
                ver_to_usize(version)
            );
            print_maybe_json!(json_printer, "SBX container block size : {}", ver_to_block_size(version) => skip_quotes);
            print_maybe_json!(json_printer, "SBX container data  size : {}", ver_to_data_size(version)  => skip_quotes);
            print_maybe_json!(json_printer, "File size                : {}", in_file_size               => skip_quotes);
            print_maybe_json!(json_printer, "SBX container size       : {}", out_file_size              => skip_quotes);
            print_maybe_json!(
                json_printer,
                "File modification time   : {}",
                in_file_mod_time_str
            );
        }

        json_printer.print_close_bracket();

        exit_with_msg!(ok json_printer => "")
    } else {
        if let Some(ref f) = out {
            exit_if_file!(exists f
                          => matches.is_present("force")
                          => json_printer
                          => "File \"{}\" already exists", f);
        }

        let in_file = if file_utils::check_if_file_is_stdin(in_file) {
            None
        } else {
            Some(in_file)
        };

        let out: Option<&str> = match out {
            None => None,
            Some(ref f) => Some(f),
        };

        let param = Param::new(
            version,
            &uid,
            data_par_burst,
            meta_enabled,
            &json_printer,
            hash_type,
            from_pos,
            to_pos,
            in_file,
            out,
            pr_verbosity_level,
        );
        match encode_core::encode_file(&param) {
            Ok(s) => exit_with_msg!(ok json_printer => "{}", s),
            Err(e) => exit_with_msg!(op json_printer => "{}", e),
        }
    }
}
