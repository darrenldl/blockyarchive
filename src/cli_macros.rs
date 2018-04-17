macro_rules! exit_with_msg {
    (
        ok => $($x:expr),*
    ) => {{
        print!($($x),*);
        return 0;
    }};
    (
        usr => $($x:expr),*
    ) => {{
        println!($($x),*);
        return 1;
    }};
    (
        op => $($x:expr),*
    ) => {{
        print!($($x),*);
        return 2;
    }}
}

macro_rules! exit_if_file {
    (
        exists $file:expr => $force_write:expr => $($x:expr),*
    ) => {{
        use file_utils;
        if file_utils::check_if_file_exists($file)
            && !$force_write
        {
            exit_with_msg!(usr => $($x),*);
        }
    }};
    (
        not_exists $file:expr => $($x:expr),*
    ) => {{
        use file_utils;
        if !file_utils::check_if_file_exists($file) {
            exit_with_msg!(usr => $($x),*);
        }
    }}
}

macro_rules! get_pr_verbosity_level {
    (
        $matches:expr
    ) => {{
        use progress_report;
        match $matches.value_of("pr_verbosity_level") {
            None    => progress_report::PRVerbosityLevel::L2,
            Some(x) => match progress_report::string_to_verbosity_level(x) {
                Ok(x)  => x,
                Err(_) => exit_with_msg!(usr => "Invalid progress report verbosity level")
            }
        }
    }}
}

macro_rules! get_from_pos {
    (
        $matches:expr
    ) => {{
        match $matches.value_of("from_pos") {
            None    => None,
            Some(x) => match u64::from_str(x) {
                Ok(x)  => Some(x),
                Err(_) => exit_with_msg!(usr => "Invalid from position")
            }
        }
    }}
}

macro_rules! get_to_pos {
    (
        $matches:expr
    ) => {{
        match $matches.value_of("to_pos") {
            None    => None,
            Some(x) => match u64::from_str(x) {
                Ok(x)  => Some(x),
                Err(_) => exit_with_msg!(usr => "Invalid to position")
            }
        }
    }}
}

macro_rules! get_in_file {
    (
        $matches:expr
    ) => {{
        let in_file  = $matches.value_of("in_file").unwrap();
        exit_if_file!(not_exists in_file => "File \"{}\" does not exist", in_file);
        in_file
    }}
}

macro_rules! get_version {
    (
        $matches:expr
    ) => {{
        use sbx_specs::string_to_ver;
        match $matches.value_of("sbx_version") {
            None    => Version::V1,
            Some(x) => match string_to_ver(&x) {
                Ok(v)   => v,
                Err(()) => {
                    exit_with_msg!(usr => "Invalid SBX version");
                }
            }
        }
    }}
}

macro_rules! get_data_shards {
    (
        $matches:expr, $version:expr
    ) => {{
        use sbx_specs::ver_to_usize;

        let ver_usize = ver_to_usize($version);

        match $matches.value_of("rs_data") {
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
        }
    }}
}

macro_rules! get_parity_shards {
    (
        $matches:expr, $version:expr
    ) => {{
        use sbx_specs::ver_to_usize;

        let ver_usize = ver_to_usize($version);

        match $matches.value_of("rs_parity") {
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
        }
    }}
}

macro_rules! check_data_parity_shards {
    (
        $data_shards:expr, $parity_shards:expr
    ) => {{
        use reed_solomon_erasure::ReedSolomon;
        use reed_solomon_erasure::Error;

        match ReedSolomon::new($data_shards, $parity_shards) {
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
    }}
}

macro_rules! get_burst_or_zero {
    (
        $matches:expr
    ) => {{
        match $matches.value_of("burst") {
            None    => 0,
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => x,
                    Err(_) => {
                        exit_with_msg!(usr => "Failed to parse burst error resistance level");
                    }
                }
            }
        }
    }}
}

macro_rules! ask_if_wish_to_continue {
    (
    ) => {{
        use std::io::{stdin, stdout, Read, Write};

        print!("Do you wish to continue? [y/N] ");

        stdout().flush().unwrap();

        let mut ans : [u8; 1] = [0; 1];

        let _ = stdin().read(&mut ans).unwrap();

        if ans != *b"y"  {
            return 0;
        }
    }}
}

macro_rules! get_burst_opt {
    (
        $matches:expr
    ) => {{
        match $matches.value_of("burst") {
            None    => None,
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => Some(x),
                    Err(_) => {
                        exit_with_msg!(usr => "Failed to parse burst error resistance level");
                    }
                }
            }
        }
    }}
}

macro_rules! parse_uid {
    (
        $buf:expr, $uid:expr
    ) => {{
        use misc_utils::HexError;
        use misc_utils;
        match misc_utils::hex_string_to_bytes($uid) {
            Ok(x) => {
                if x.len() != SBX_FILE_UID_LEN {
                    exit_with_msg!(usr => "UID must be {} bytes({} hex characters) in length",
                                   SBX_FILE_UID_LEN,
                                   SBX_FILE_UID_LEN * 2);
                }

                $buf.copy_from_slice(&x);
            },
            Err(HexError::InvalidHexString) => {
                exit_with_msg!(usr => "UID provided is not a valid hex string");
            },
            Err(HexError::InvalidLen) => {
                exit_with_msg!(usr => "UID provided does not have the correct number of hex digits, provided : {}, need : {}",
                               $uid.len(),
                               SBX_FILE_UID_LEN * 2);
            }
        }
    }}
}

macro_rules! get_ref_block_choice {
    (
        $matches:expr
    ) => {{
        use block_utils::RefBlockChoice::*;
        use sbx_block::BlockType;

        if $matches.is_present("no_meta") {
            Any
        } else {
            Prefer(BlockType::Meta)
        }
    }}
}

macro_rules! get_meta_enabled {
    (
        $matches:expr
    ) => {{
        !$matches.is_present("no_meta")
    }}
}

macro_rules! get_json_enabled {
    (
        $matches:expr
    ) => {{
        $matches.is_present("json")
    }}
}

macro_rules! print_json_field {
    (
        $key:expr, $value:expr
    ) => {{
        println!("\"{}\": {}", $key, $value)
    }}
}
