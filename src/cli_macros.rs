#[macro_export]
macro_rules! exit_with_msg {
    (
        ok $json_printer:expr => $($x:expr),*
    ) => {{
        print_at_output_channel!($json_printer.output_channel() => $($x),*);
        print_field_if_json!($json_printer, "error : null");
        $json_printer.print_close_bracket();
        return 0;
    }};
    (
        usr $json_printer:expr => $($x:expr),*
    ) => {{
        if $json_printer.json_enabled() {
            print_json_field!($json_printer.output_channel() => "error", format!($($x),*), false, $json_printer.first_item());
        } else {
            println_at_output_channel!($json_printer.output_channel() => $($x),*);
        }
        $json_printer.print_close_bracket();
        return 1;
    }};
    (
        op $json_printer:expr => $($x:expr),*
    ) => {{
        if $json_printer.json_enabled() {
            print_json_field!($json_printer.output_channel() => "error", format!($($x),*), false, $json_printer.first_item());
        } else {
            println_at_output_channel!($json_printer.output_channel() => $($x),*);
        }
        $json_printer.print_close_bracket();
        return 2;
    }}
}

macro_rules! exit_if_file {
    (
        exists $file:expr => $force_write:expr => $json_printer:expr => $($x:expr),*
    ) => {{
        use crate::file_utils;
        if file_utils::check_if_file_exists($file)
            && !$force_write
        {
            exit_with_msg!(usr $json_printer => $($x),*);
        }
    }};
    (
        not_exists $file:expr => $json_printer:expr => $($x:expr),*
    ) => {{
        use crate::file_utils;
        if !file_utils::check_if_file_exists($file) {
            exit_with_msg!(usr $json_printer => $($x),*);
        }
    }}
}

macro_rules! get_pr_verbosity_level {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        use crate::progress_report;
        use crate::progress_report::PRVerbosityLevel;

        match $matches.value_of("pr_verbosity_level") {
            None    => if get_json_enabled!($matches) { PRVerbosityLevel::L0 } else { PRVerbosityLevel::L2 },
            Some(x) => match progress_report::string_to_verbosity_level(x) {
                Ok(x)  => x,
                Err(_) => exit_with_msg!(usr $json_printer => "Invalid progress report verbosity level")
            }
        }
    }}
}

macro_rules! get_from_pos {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        use std::str::FromStr;

        match $matches.value_of("from_pos") {
            None    => None,
            Some(x) => match u64::from_str(x) {
                Ok(x)  => Some(x),
                Err(_) => exit_with_msg!(usr $json_printer => "Invalid from position")
            }
        }
    }}
}

macro_rules! get_to_pos {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        use std::str::FromStr;
        use crate::misc_utils::RangeEnd;

        match ($matches.value_of("to_pos_inc"), $matches.value_of("to_pos_exc")) {
            (None,    None   ) => None,
            (Some(x), None   ) =>
                match u64::from_str(x) {
                    Ok(x)  => Some(RangeEnd::Inc(x)),
                    Err(_) => exit_with_msg!(usr $json_printer => "Invalid to inc position")
                },
            (None,    Some(x)) =>
                match u64::from_str(x) {
                    Ok(x)  => Some(RangeEnd::Exc(x)),
                    Err(_) => exit_with_msg!(usr $json_printer => "Invalid to exc position")
                },
            (Some(_), Some(_)) =>
                unreachable!(),
        }
    }}
}

macro_rules! get_guess_burst_from_pos {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        use std::str::FromStr;

        match $matches.value_of("guess_burst_from_pos") {
            None    => None,
            Some(x) => match u64::from_str(x) {
                Ok(x)  => Some(x),
                Err(_) => exit_with_msg!(usr $json_printer => "Invalid guess burst from position")
            }
        }
    }}
}

macro_rules! get_ref_from_pos {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        use std::str::FromStr;

        match $matches.value_of("ref_from_pos") {
            None    => None,
            Some(x) => match u64::from_str(x) {
                Ok(x)  => Some(x),
                Err(_) => exit_with_msg!(usr $json_printer => "Invalid ref from position")
            }
        }
    }}
}

macro_rules! get_ref_to_pos {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        use std::str::FromStr;
        use crate::misc_utils::RangeEnd;

        match ($matches.value_of("ref_to_pos_inc"), $matches.value_of("ref_to_pos_exc")) {
            (None,    None   ) => None,
            (Some(x), None   ) =>
                match u64::from_str(x) {
                    Ok(x)  => Some(RangeEnd::Inc(x)),
                    Err(_) => exit_with_msg!(usr $json_printer => "Invalid ref to inc position")
                },
            (None,    Some(x)) =>
                match u64::from_str(x) {
                    Ok(x)  => Some(RangeEnd::Exc(x)),
                    Err(_) => exit_with_msg!(usr $json_printer => "Invalid ref to exc position")
                },
            (Some(_), Some(_)) =>
                unreachable!()
        }
    }}
}

macro_rules! get_in_file {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        let in_file  = $matches.value_of("in_file").unwrap();
        exit_if_file!(not_exists in_file
                      => $json_printer
                      => "File \"{}\" does not exist", in_file);
        in_file
    }};
    (
        accept_stdin $matches:expr, $json_printer:expr
    ) => {{
        use crate::file_utils;
        let in_file  = $matches.value_of("in_file").unwrap();
        if !file_utils::check_if_file_is_stdin(in_file) {
            exit_if_file!(not_exists in_file
                          => $json_printer
                          => "File \"{}\" does not exist", in_file);
        }
        in_file
    }};
}

macro_rules! get_data_shards {
    (
        $matches:expr, $version:expr, $json_printer:expr
    ) => {{
        use crate::sbx_specs::ver_to_usize;

        let ver_usize = ver_to_usize($version);

        match $matches.value_of("rs_data") {
            None    => {
                exit_with_msg!(usr $json_printer => "Reed-Solomon erasure code data shard count must be specified for version {}", ver_usize);
            },
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => x,
                    Err(_) => {
                        exit_with_msg!(usr $json_printer => "Failed to parse Reed-Solomon erasure code data shard count");
                    }
                }
            }
        }
    }}
}

macro_rules! get_parity_shards {
    (
        $matches:expr, $version:expr, $json_printer:expr
    ) => {{
        use crate::sbx_specs::ver_to_usize;

        let ver_usize = ver_to_usize($version);

        match $matches.value_of("rs_parity") {
            None    => {
                exit_with_msg!(usr $json_printer => "Reed-Solomon erasure code parity shard count must be specified for version {}", ver_usize);
            },
            Some(x) => {
                match usize::from_str(&x) {
                    Ok(x)  => x,
                    Err(_) => {
                        exit_with_msg!(usr $json_printer => "Failed to parse Reed-Solomon erasure code parity shard count");
                    }
                }
            }
        }
    }}
}

macro_rules! get_ver_and_data_par_burst_w_defaults {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        use crate::sbx_specs::string_to_ver;
        use crate::encode_defaults;

        match $matches.value_of("sbx_version") {
            None => {
                if let Some(_) = $matches.value_of("rs_data") {
                    exit_with_msg!(usr $json_printer => "Please state the SBX version explicitly if you want to use a custom Reed-Solomon data shard count");
                }
                if let Some(_) = $matches.value_of("rs_parity") {
                    exit_with_msg!(usr $json_printer => "Please state the SBX version explicitly if you want to use a custom Reed-Solomon parity shard count");
                }
                if let Some(_) = $matches.value_of("burst") {
                    exit_with_msg!(usr $json_printer => "Please state the SBX version explicitly if you want to use a custom burst error resistance level");
                }

                (encode_defaults::VERSION, Some(encode_defaults::DATA_PAR_BURST))
            },
            Some(x) => {
                let version =
                    match string_to_ver(&x) {
                        Ok(v)   => v,
                        Err(()) => {
                            exit_with_msg!(usr $json_printer => "Invalid SBX version");
                        }
                    };

                let data_par_burst = if ver_uses_rs(version) {
                    // deal with RS related options
                    let data_shards = get_data_shards!($matches, version, $json_printer);
                    let parity_shards = get_parity_shards!($matches, version, $json_printer);

                    check_data_parity_shards!(data_shards, parity_shards, $json_printer);

                    let burst = get_burst_or_zero!($matches, $json_printer);

                    Some((data_shards, parity_shards, burst))
                } else {
                    None
                };

                (version, data_par_burst)
            }
        }
    }}
}

macro_rules! check_data_parity_shards {
    (
        $data_shards:expr, $parity_shards:expr, $json_printer:expr
    ) => {{
        use reed_solomon_erasure::ReedSolomon;
        use reed_solomon_erasure::Error;

        match ReedSolomon::new($data_shards, $parity_shards) {
            Ok(_)                          => {},
            Err(Error::TooFewDataShards)   => {
                exit_with_msg!(usr $json_printer => "Too few data shards for Reed-Solomon erasure code");
            },
            Err(Error::TooFewParityShards) => {
                exit_with_msg!(usr $json_printer => "Too few parity shards for Reed-Solomon erasure code");
            },
            Err(Error::TooManyShards)      => {
                exit_with_msg!(usr $json_printer => "Too many shards for Reed-Solomon erasure code");
            },
            Err(_)                         => { panic!(); }
        }
    }}
}

macro_rules! get_burst_or_zero {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        match $matches.value_of("burst") {
            None    => 0,
            Some(x) => {
                match usize::from_str(x) {
                    Ok(x)  => x,
                    Err(_) => {
                        exit_with_msg!(usr $json_printer => "Failed to parse burst error resistance level");
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

        let mut ans: [u8; 1] = [0; 1];

        let _ = stdin().read(&mut ans).unwrap();

        if ans != *b"y" {
            return 0;
        }
    }};
}

macro_rules! get_burst_opt {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        use std::str::FromStr;

        match $matches.value_of("burst") {
            None    => None,
            Some(x) => {
                match usize::from_str(x) {
                    Ok(x)  => Some(x),
                    Err(_) => {
                        exit_with_msg!(usr $json_printer => "Failed to parse burst error resistance level");
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
        use crate::misc_utils::HexError;
        use crate::misc_utils;
        use crate::sbx_specs::SBX_FILE_UID_LEN;

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

macro_rules! get_uid {
    (
        $matches:expr, $buf:expr, $json_printer:expr
    ) => {{
        match $matches.value_of("uid") {
            None => None,
            Some(uid) => {
                parse_uid!($buf, uid, $json_printer);
                Some(&$buf)
            }
        }
    }};
}

macro_rules! get_ref_block_choice {
    (
        $matches:expr
    ) => {{
        use crate::block_utils::RefBlockChoice::*;
        use crate::sbx_block::BlockType;

        if $matches.is_present("no_meta") {
            Any
        } else {
            Prefer(BlockType::Meta)
        }
    }};
}

macro_rules! get_meta_enabled {
    (
        $matches:expr
    ) => {{
        !$matches.is_present("no_meta")
    }};
}

macro_rules! get_json_enabled {
    (
        $matches:expr
    ) => {{
        $matches.is_present("json")
    }};
}

macro_rules! get_json_printer {
    (
        $matches:expr
    ) => {{
        use crate::json_printer::JSONPrinter;
        use crate::output_channel::OutputChannel;
        use std::sync::Arc;

        Arc::new(JSONPrinter::new(
            $matches.is_present("json"),
            OutputChannel::Stdout,
        ))
    }};
}

#[macro_export]
macro_rules! print_at_output_channel {
    (
        $channel:expr => $($x:expr),*
    ) => {{
        use crate::output_channel::OutputChannel;

        match $channel {
            OutputChannel::Stdout => print!($($x),*),
            OutputChannel::Stderr => eprint!($($x),*),
        }
    }}
}

#[macro_export]
macro_rules! println_at_output_channel {
    (
        $channel:expr => $($x:expr),*
    ) => {{
        use crate::output_channel::OutputChannel;

        match $channel {
            OutputChannel::Stdout => println!($($x),*),
            OutputChannel::Stderr => eprintln!($($x),*),
        }
    }}
}

macro_rules! get_multi_pass {
    (
        $matches:expr, $json_printer:expr
    ) => {{
        use crate::misc_utils::MultiPassType;

        match (
            $matches.is_present("multi_pass"),
            $matches.is_present("multi_pass_no_skip"),
        ) {
            (false, false) => None,
            (true, false) => Some(MultiPassType::SkipGood),
            (false, true) => Some(MultiPassType::OverwriteAll),
            (true, true) => unreachable!(),
        }
    }};
}
