use super::clap::ArgMatches;
use super::encode_core::Param;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       Version,
                       string_to_ver,
                       ver_supports_rs,
                       ver_to_usize};
use std::str::FromStr;
use std::path::Path;

use super::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("encode")
        .about("Encode file")
        .arg(Arg::with_name("in_file")
             .value_name("INFILE")
             .required(true)
             .index(1)
             .help("File to encode"))
        .arg(Arg::with_name("out_file")
             .value_name("OUT")
             .index(2)
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
             .default_value("sha256")
             .help("Hash function to use, one of (case-insensitive) :
    sha1
    sha256 (default)
    sha512
    blake2b-512"))
        .arg(Arg::with_name("no_meta")
             .long("no-meta")
             .help("Skip metadata block in the SBX container. Metadata block is
never skipped for version 11, 12, 13.
This means this option does nothing for version 11, 12, 13."))
        .arg(Arg::with_name("silence_level")
             .value_name("LEVEL")
             .short("s")
             .long("silent")
             .takes_value(true)
             .help("One of :
    0 (show everything)
    1 (only show progress stats when done)
    2 (show nothing)
This only affects progress text printing."))
        .arg(Arg::with_name("sbx_version")
             .value_name("SBX-VERSION")
             .long("sbx-version")
             .takes_value(true)
             .default_value("1")
             .help("Sbx container version, one of :
    1  (bs=512  bytes)
    2  (bs=128  bytes)
    3  (bs=4096 bytes)
    11 (bs=512  bytes, Reed-Solomon enabled)
    12 (bs=128  bytes, Reed-Solomon enabled)
    13 (bs=4096 bytes, Reed-Solomon enabled)
where bs=sbx block size."))
        .arg(Arg::with_name("uid")
             .value_name("UID-HEX")
             .long("uid")
             .takes_value(true)
             .help("Alternative file uid in hex (by default uid is randomly generated).
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
}

pub fn encode<'a>(matches : &ArgMatches<'a>) -> i32 {
    // compute uid
    let mut uid : [u8; SBX_FILE_UID_LEN] = [0; SBX_FILE_UID_LEN];
    {
        match matches.value_of("uid") {
            None    => { rand_utils::fill_random_bytes(&mut uid); },
            Some(x) => {
                use misc_utils::HexError::*;
                match misc_utils::hex_string_to_bytes(x) {
                    Ok(x) => {
                        if x.len() != SBX_FILE_UID_LEN {
                            exit_with_msg!(usr => "UID must be {} bytes({} hex characters) in length",
                                           SBX_FILE_UID_LEN,
                                           SBX_FILE_UID_LEN * 2);
                        }

                        uid.copy_from_slice(&x);
                    },
                    Err(InvalidHexString) => {
                        exit_with_msg!(usr => "UID provided is not a valid hex string");
                    },
                    Err(InvalidLen) => {
                        exit_with_msg!(usr => "UID provided does not have the correct number of hex digits, provided : {}, need : {}",
                                       x.len(),
                                       SBX_FILE_UID_LEN * 2);
                    }
                }
            }
        }
    }

    // compute version
    let mut version : Version =
        match matches.value_of("sbx_version") {
            None    => Version::V1,
            Some(x) => match string_to_ver(&x) {
                Ok(v)   => v,
                Err(()) => {
                    exit_with_msg!(usr => "Invalid SBX version");
                }
            }
        };

    let ver_usize = ver_to_usize(version);

    let (rs_data, rs_parity) =
        if ver_supports_rs(version) {
            use reed_solomon_erasure::ReedSolomon;
            use reed_solomon_erasure::Error;
            // deal with RS related options
            let data_shards = match matches.value_of("rs_data") {
                None    => {
                    exit_with_msg!(usr => "Reed-Solomon erasure code data shard count must be specified for version {}", ver_usize);
                },
                Some(x) => {
                    match usize::from_str(&x) {
                        Ok(x)  => x,
                        Err(_) => {
                            exit_with_msg!(usr => "Failed to parse Reed-Solomon erasure code data shard count");
                        }
                    }
                }
            };
            let parity_shards = match matches.value_of("rs_parity") {
                None    => {
                    exit_with_msg!(usr => "Reed-Solomon erasure code parity shard count must be specified for version {}", ver_usize);
                },
                Some(x) => {
                    match usize::from_str(&x) {
                        Ok(x)  => x,
                        Err(_) => {
                            exit_with_msg!(usr => "Failed to parse Reed-Solomon erasure code parity shard count");
                        }
                    }
                }
            };
            match ReedSolomon::new(data_shards, parity_shards) {
                Ok(_)                          => {},
                Err(Error::TooFewDataShards)   => {
                    exit_with_msg!(usr => "Too few data shards for Reed-Solomon erasure code");
                },
                Err(Error::TooFewParityShards) => {
                    exit_with_msg!(usr => "Too few parity shards for Reed-Solomon erasure code");
                },
                Err(Error::TooManyShards)      => {
                    exit_with_msg!(usr => "Too many shards for Reed-Solomon erasure code");
                },
                Err(_)                         => { panic!(); }
            }
            (data_shards, parity_shards)
        } else {
            (1, 1) // use dummy values
        };

    let force_write = matches.is_present("force");

    let in_file  = matches.value_of("in_file").unwrap();
    exit_if_file!(not_exists in_file => "File \"{}\" does not exist", in_file);
    let out_file = match matches.value_of("out_file") {
        None    => format!("{}.sbx", in_file),
        Some(x) => {
            let x =
                if file_utils::check_if_file_is_dir(x) {
                    misc_utils::make_path(&[x, in_file])
                } else {
                    String::from(x)
                };
            if !force_write {
                exit_if_file!(exists &x => "File \"{}\" already exists", x);
            }
            x
        }
    };

    let hash_type = match matches.value_of("hash_type") {
        None    => multihash::HashType::SHA256,
        Some(x) => match multihash::string_to_hash_type(x) {
            Ok(x)  => x,
            Err(_) => exit_with_msg!(usr => "Invalid hash type")
        }
    };

    let silence_level = match matches.value_of("silence_level") {
        None    => progress_report::SilenceLevel::L0,
        Some(x) => match progress_report::string_to_silence_level(x) {
            Ok(x)  => x,
            Err(_) => exit_with_msg!(usr => "Invalid silence level")
        }
    };

    let param = Param::new(version,
                           &uid,
                           rs_data,
                           rs_parity,
                           matches.is_present("no_meta"),
                           hash_type,
                           in_file,
                           &out_file,
                           silence_level);
    match encode_core::encode_file(&param) {
        Ok(s)  => exit_with_msg!(ok => "{}", s),
        Err(e) => exit_with_msg!(op => "{}", e)
    }
}
