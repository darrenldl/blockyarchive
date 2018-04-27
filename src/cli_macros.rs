macro_rules! exit_with_msg {
    (
        ok $json_printer:expr => $($x:expr),*
    ) => {{
        print!($($x),*);
        print_field_if_json!($json_printer, "error : null");
        $json_printer.print_close_bracket();
        return 0;
    }};
    (
        usr $json_printer:expr => $($x:expr),*
    ) => {{
        if $json_printer.json_enabled() {
            print_json_field!("error", format!($($x),*), false, $json_printer.first_item());
        } else {
            println!($($x),*);
        }
        $json_printer.print_close_bracket();
        return 1;
    }};
    (
        op $json_printer:expr => $($x:expr),*
    ) => {{
        if $json_printer.json_enabled() {
            print_json_field!("error", format!($($x),*), false, $json_printer.first_item());
        } else {
            println!($($x),*);
        }
        $json_printer.print_close_bracket();
        return 2;
    }}
}

macro_rules! exit_if_file {
    (
        exists $file:expr => $force_write:expr => $json_printer:expr => $($x:expr),*
    ) => {{
        use file_utils;
        if file_utils::check_if_file_exists($file)
            && !$force_write
        {
            exit_with_msg!(usr $json_printer => $($x),*);
        }
    }};
    (
        not_exists $file:expr => $json_printer:expr => $($x:expr),*
    ) => {{
        use file_utils;
        if !file_utils::check_if_file_exists($file) {
            exit_with_msg!(usr $json_printer => $($x),*);
        }
    }}
}

macro_rules! get_pr_verbosity_level {
    (
        $matches:expr, $json_enabled:expr
    ) => {{
        use progress_report;
        use progress_report::PRVerbosityLevel;
        match $matches.value_of("pr_verbosity_level") {
            None    => if get_json_enabled!($matches) { PRVerbosityLevel::L0 } else { PRVerbosityLevel::L2 },
            Some(x) => match progress_report::string_to_verbosity_level(x) {
                Ok(x)  => x,
                Err(_) => exit_with_msg!(usr $json_enabled => "Invalid progress report verbosity level")
            }
        }
    }}
}

macro_rules! get_from_pos {
    (
        $matches:expr, $json_enabled:expr
    ) => {{
        match $matches.value_of("from_pos") {
            None    => None,
            Some(x) => match u64::from_str(x) {
                Ok(x)  => Some(x),
                Err(_) => exit_with_msg!(usr $json_enabled => "Invalid from position")
            }
        }
    }}
}

macro_rules! get_to_pos {
    (
        $matches:expr, $json_enabled:expr
    ) => {{
        match $matches.value_of("to_pos") {
            None    => None,
            Some(x) => match u64::from_str(x) {
                Ok(x)  => Some(x),
                Err(_) => exit_with_msg!(usr $json_enabled => "Invalid to position")
            }
        }
    }}
}

macro_rules! get_in_file {
    (
        $matches:expr, $json_enabled:expr
    ) => {{
        let in_file  = $matches.value_of("in_file").unwrap();
        exit_if_file!(not_exists in_file
                      => $json_enabled
                      => "File \"{}\" does not exist", in_file);
        in_file
    }}
}

macro_rules! get_version {
    (
        $matches:expr, $json_enabled:expr
    ) => {{
        use sbx_specs::string_to_ver;
        match $matches.value_of("sbx_version") {
            None    => Version::V1,
            Some(x) => match string_to_ver(&x) {
                Ok(v)   => v,
                Err(()) => {
                    exit_with_msg!(usr $json_enabled => "Invalid SBX version");
                }
            }
        }
    }}
}

macro_rules! get_data_shards {
    (
        $matches:expr, $version:expr, $json_enabled:expr
    ) => {{
        use sbx_specs::ver_to_usize;

        let ver_usize = ver_to_usize($version);

        match $matches.value_of("rs_data") {
            None    => {
                exit_with_msg!(usr $json_enabled => "Reed-Solomon erasure code data shard count must be specified for version {}", ver_usize);
            },
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => x,
                    Err(_) => {
                        exit_with_msg!(usr $json_enabled => "Failed to parse Reed-Solomon erasure code data shard count");
                    }
                }
            }
        }
    }}
}

macro_rules! get_parity_shards {
    (
        $matches:expr, $version:expr, $json_enabled:expr
    ) => {{
        use sbx_specs::ver_to_usize;

        let ver_usize = ver_to_usize($version);

        match $matches.value_of("rs_parity") {
            None    => {
                exit_with_msg!(usr $json_enabled => "Reed-Solomon erasure code parity shard count must be specified for version {}", ver_usize);
            },
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => x,
                    Err(_) => {
                        exit_with_msg!(usr $json_enabled => "Failed to parse Reed-Solomon erasure code parity shard count");
                    }
                }
            }
        }
    }}
}

macro_rules! check_data_parity_shards {
    (
        $data_shards:expr, $parity_shards:expr, $json_enabled:expr
    ) => {{
        use reed_solomon_erasure::ReedSolomon;
        use reed_solomon_erasure::Error;

        match ReedSolomon::new($data_shards, $parity_shards) {
            Ok(_)                          => {},
            Err(Error::TooFewDataShards)   => {
                exit_with_msg!(usr $json_enabled => "Too few data shards for Reed-Solomon erasure code");
            },
            Err(Error::TooFewParityShards) => {
                exit_with_msg!(usr $json_enabled => "Too few parity shards for Reed-Solomon erasure code");
            },
            Err(Error::TooManyShards)      => {
                exit_with_msg!(usr $json_enabled => "Too many shards for Reed-Solomon erasure code");
            },
            Err(_)                         => { panic!(); }
        }
    }}
}

macro_rules! get_burst_or_zero {
    (
        $matches:expr, $json_enabled:expr
    ) => {{
        match $matches.value_of("burst") {
            None    => 0,
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => x,
                    Err(_) => {
                        exit_with_msg!(usr $json_enabled => "Failed to parse burst error resistance level");
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
        $matches:expr, $json_enabled:expr
    ) => {{
        match $matches.value_of("burst") {
            None    => None,
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => Some(x),
                    Err(_) => {
                        exit_with_msg!(usr $json_enabled => "Failed to parse burst error resistance level");
                    }
                }
            }
        }
    }}
}

macro_rules! parse_uid {
    (
        $buf:expr, $uid:expr, $json_printer:expr
    ) => {{
        use misc_utils::HexError;
        use misc_utils;
        match misc_utils::hex_string_to_bytes($uid) {
            Ok(x) => {
                if x.len() != SBX_FILE_UID_LEN {
                    exit_with_msg!(usr $json_printer => "UID provided does not have the correct number of hex digits, provided : {}, need : {}",
                                   $uid.len(),
                                   SBX_FILE_UID_LEN * 2);
                }

                $buf.copy_from_slice(&x);
            },
            Err(HexError::InvalidHexString) => {
                exit_with_msg!(usr $json_printer => "UID provided is not a valid hex string");
            },
            Err(HexError::InvalidLen) => {
                exit_with_msg!(usr $json_printer => "UID provided does not have the correct number of hex digits, provided : {}, need : {}",
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

macro_rules! get_json_printer {
    (
        $matches:expr
    ) => {{
        use json_printer::JSONPrinter;
        use std::sync::Arc;

        Arc::new(JSONPrinter::new($matches.is_present("json")))
    }}
}
