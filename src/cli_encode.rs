use encode_core::Param;
use encode_core;
use sbx_specs::{SBX_FILE_UID_LEN,
                Version,
                string_to_ver,
                ver_uses_rs,
                ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use multihash;

use file_utils;
use misc_utils;
use rand_utils;

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("encode")
        .about("Encode file")
        .arg(in_file_arg()
             .help("File to encode"))
        .arg(out_arg()
             .help("SBX container name (defaults to INFILE.sbx). If OUT is a
directory(DIR), then the final file will be stored as
DIR/INFILE.sbx."))
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
        .arg(Arg::with_name("sbx_version")
             .value_name("SBX-VERSION")
             .long("sbx-version")
             .takes_value(true)
             .help("SBX container version, one of :
                    | SBX block size | Reed-Solomon | Burst error resistance |
(default)  1        |      512 bytes |  not enabled |          not supported |
           2        |      128 bytes |  not enabled |          not supported |
           3        |     4096 bytes |  not enabled |          not supported |
          17 (0x11) |      512 bytes |      enabled |              supported |
          18 (0x12) |      128 bytes |      enabled |              supported |
          19 (0x13) |     4096 bytes |      enabled |              supported |"))
        .arg(Arg::with_name("uid")
             .value_name("UID-HEX")
             .long("uid")
             .takes_value(true)
             .help("Alternative file uid in hex(by default uid is randomly generated).
Uid must be exactly 6 bytes(12 hex digits) in length."))
        .arg(Arg::with_name("rs_data")
             .value_name("SHARD")
             .long("rs-data")
             .takes_value(true)
             .help("Reed-Solomon data shard count"))
        .arg(Arg::with_name("rs_parity")
             .value_name("SHARD")
             .long("rs-parity")
             .takes_value(true)
             .help("Reed-Solomon parity shard count"))
        .arg(burst_arg())
}

pub fn encode<'a>(matches : &ArgMatches<'a>) -> i32 {
    // compute uid
    let mut uid : [u8; SBX_FILE_UID_LEN] = [0; SBX_FILE_UID_LEN];
    {
        match matches.value_of("uid") {
            None    => { rand_utils::fill_random_bytes(&mut uid); },
            Some(x) => { parse_uid!(uid, x); }
        }
    }

    let version   = get_version!(matches);

    let data_par_burst =
        if ver_uses_rs(version) {
            // deal with RS related options
            let data_shards   = get_data_shards!(matches, version);
            let parity_shards = get_parity_shards!(matches, version);

            check_data_parity_shards!(data_shards, parity_shards);

            let burst = get_burst_or_zero!(matches);

            Some((data_shards, parity_shards, burst))
        } else {
            None
        };

    let in_file = get_in_file!(matches);
    let out = match matches.value_of("out") {
        None    => format!("{}.sbx", in_file),
        Some(x) => {
            if file_utils::check_if_file_is_dir(x) {
                misc_utils::make_path(&[x, in_file])
            } else {
                String::from(x)
            }
        }
    };

    exit_if_file!(exists &out
                  => matches.is_present("force")
                  => "File \"{}\" already exists", out);

    let hash_type = match matches.value_of("hash_type") {
        None    => multihash::HashType::SHA256,
        Some(x) => match multihash::string_to_hash_type(x) {
            Ok(x)  => x,
            Err(_) => exit_with_msg!(usr => "Invalid hash type")
        }
    };

    let pr_verbosity_level = get_pr_verbosity_level!(matches);

    let param = Param::new(version,
                           &uid,
                           data_par_burst,
                           matches.is_present("no_meta"),
                           hash_type,
                           in_file,
                           &out,
                           pr_verbosity_level);
    match encode_core::encode_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
