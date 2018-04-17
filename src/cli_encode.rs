use std::time::UNIX_EPOCH;
use encode_core::Param;
use encode_core;
use sbx_specs::{SBX_FILE_UID_LEN,
                Version,
                ver_to_usize,
                ver_to_block_size,
                ver_to_data_size,
                ver_uses_rs};
use std::str::FromStr;

use multihash;

use file_utils;
use misc_utils;
use rand_utils;
use time_utils;

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("encode")
        .about("Encode file")
        .arg(in_file_arg()
             .help("File to encode"))
        .arg(out_arg()
             .help("SBX container name(defaults to INFILE.sbx). If OUT is a
directory, then the container is stored as OUT/INFILE.sbx."))
        .arg(Arg::with_name("force")
             .short("f")
             .long("force")
             .help("Force overwrite even if OUT exists"))
        .arg(Arg::with_name("hash_type")
             .value_name("HASH-TYPE")
             .long("hash")
             .takes_value(true)
             .help("Hash function to use, one of (case-insensitive) :
          sha1
(default) sha256
          sha512
          blake2b-512"))
        .arg(Arg::with_name("no_meta")
             .long("no-meta")
             .help("Skip metadata block in the SBX container. Metadata block is
never skipped for version 17, 18, 19.
This means this option has no effect for version 17, 18, 19."))
        .arg(pr_verbosity_level_arg())
        .arg(sbx_version_arg())
        .arg(Arg::with_name("uid")
             .value_name("UID-HEX")
             .long("uid")
             .takes_value(true)
             .help("Alternative file uid in hex(by default uid is randomly generated).
Uid must be exactly 6 bytes(12 hex digits) in length."))
        .arg(rs_data_arg())
        .arg(rs_parity_arg())
        .arg(burst_arg()
            .help("Burst error resistance level. Note that rsbx only guesses up to
1000 in repair, show, and sort mode. If you use level above 1000,
then rsbx will make an incorrect guess, and you will need to
specify it explicitly in repair and sort mode. Show mode does
not rely on burst level, but provides an option for enabling
automatic guessing."))
        .arg(Arg::with_name("info_only")
             .long("info-only")
             .help("Only display information about encoding then exit"))
        .arg(json_arg())
}

pub fn encode<'a>(matches : &ArgMatches<'a>) -> i32 {
    let json_enabled = get_json_enabled!(matches);

    print_maybe_json_open_bracket!(json_enabled);

    // compute uid
    let mut uid : [u8; SBX_FILE_UID_LEN] = [0; SBX_FILE_UID_LEN];
    {
        match matches.value_of("uid") {
            None    => { rand_utils::fill_random_bytes(&mut uid); },
            Some(x) => { parse_uid!(uid, x, json_enabled); }
        }
    }

    let version   = get_version!(matches, json_enabled);

    let data_par_burst =
        if ver_uses_rs(version) {
            // deal with RS related options
            let data_shards   = get_data_shards!(matches, version, json_enabled);
            let parity_shards = get_parity_shards!(matches, version, json_enabled);

            check_data_parity_shards!(data_shards, parity_shards, json_enabled);

            let burst = get_burst_or_zero!(matches, json_enabled);

            Some((data_shards, parity_shards, burst))
        } else {
            None
        };

    let in_file = get_in_file!(matches, json_enabled);
    let out = match matches.value_of("out") {
        None    => format!("{}.sbx", in_file),
        Some(x) => {
            if file_utils::check_if_file_is_dir(x) {
                let in_file = file_utils::get_file_name_part_of_path(in_file);
                misc_utils::make_path(&[x,
                                        &format!("{}.sbx", in_file)])
            } else {
                String::from(x)
            }
        }
    };

    let hash_type = match matches.value_of("hash_type") {
        None    => multihash::HashType::SHA256,
        Some(x) => match multihash::string_to_hash_type(x) {
            Ok(x)  => x,
            Err(_) => exit_with_msg!(usr json_enabled => "Invalid hash type")
        }
    };

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_enabled);

    let meta_enabled = get_meta_enabled!(matches);

    if matches.is_present("info_only") {
        let in_file_meta  = match file_utils::get_file_metadata(in_file) {
            Ok(x)  => x,
            Err(_) => exit_with_msg!(usr json_enabled => "Failed to get metadata of \"{}\"",
                                     in_file)
        };

        let in_file_size = match file_utils::get_file_size(in_file) {
            Ok(x)  => x,
            Err(_) => exit_with_msg!(usr json_enabled => "Failed to get file size of \"{}\"",
                                     in_file)
        };

        let in_file_mod_time = match in_file_meta.modified() {
            Ok(t)  => match t.duration_since(UNIX_EPOCH) {
                Ok(t)  => Some(t.as_secs() as i64),
                Err(_) => None,
            },
            Err(_) => None
        };

        let in_file_mod_time_str = match in_file_mod_time {
            None    => "N/A".to_string(),
            Some(x) => match (time_utils::i64_secs_to_date_time_string(x, time_utils::TimeMode::UTC),
                              time_utils::i64_secs_to_date_time_string(x, time_utils::TimeMode::Local)) {
                (Some(u), Some(l)) => format!("{} (UTC)  {} (Local)", u, l),
                _                  => "Invalid file modification time".to_string(),
            }
        };

        let out_file_size =
            file_utils::from_orig_file_size::calc_container_size(version,
                                                                 Some(meta_enabled),
                                                                 data_par_burst,
                                                                 in_file_size);

        if ver_uses_rs(version) {
            print_maybe_json!(json_enabled, "File name                    : {}", in_file                    => no_comma);
            print_maybe_json!(json_enabled, "SBX container name           : {}", out);
            print_maybe_json!(json_enabled, "SBX container version        : {}", ver_to_usize(version));
            print_maybe_json!(json_enabled, "SBX container block size     : {}", ver_to_block_size(version) => skip_quotes);
            print_maybe_json!(json_enabled, "SBX container data  size     : {}", ver_to_data_size(version)  => skip_quotes);
            print_maybe_json!(json_enabled, "RS data   shard count        : {}", data_par_burst.unwrap().0  => skip_quotes);
            print_maybe_json!(json_enabled, "RS parity shard count        : {}", data_par_burst.unwrap().1  => skip_quotes);
            print_maybe_json!(json_enabled, "Burst error resistance level : {}", data_par_burst.unwrap().2  => skip_quotes);
            print_maybe_json!(json_enabled, "File size                    : {}", in_file_size               => skip_quotes);
            print_maybe_json!(json_enabled, "SBX container size           : {}", out_file_size              => skip_quotes);
            print_maybe_json!(json_enabled, "File modification time       : {}", in_file_mod_time_str);
        } else {
            print_maybe_json!(json_enabled, "File name                : {}", in_file                    => no_comma);
            print_maybe_json!(json_enabled, "SBX container name       : {}", out);
            print_maybe_json!(json_enabled, "SBX container version    : {}", ver_to_usize(version));
            print_maybe_json!(json_enabled, "SBX container block size : {}", ver_to_block_size(version) => skip_quotes);
            print_maybe_json!(json_enabled, "SBX container data  size : {}", ver_to_data_size(version)  => skip_quotes);
            print_maybe_json!(json_enabled, "File size                : {}", in_file_size               => skip_quotes);
            print_maybe_json!(json_enabled, "SBX container size       : {}", out_file_size              => skip_quotes);
            print_maybe_json!(json_enabled, "File modification time   : {}", in_file_mod_time_str);
        }

        exit_with_msg!(ok json_enabled => "")
    } else {
        exit_if_file!(exists &out
                      => matches.is_present("force")
                      => json_enabled
                      => "File \"{}\" already exists", out);

        let param = Param::new(version,
                               &uid,
                               data_par_burst,
                               meta_enabled,
                               json_enabled,
                               hash_type,
                               in_file,
                               &out,
                               pr_verbosity_level);
        match encode_core::encode_file(&param) {
            Ok(s)  => exit_with_msg!(ok json_enabled => "{}", s),
            Err(e) => exit_with_msg!(op json_enabled => "{}", e)
        }
    }
}
