use super::clap::*;
use super::sbx_block;

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
        if file_utils::check_if_file_exists($file)
            && !$force_write
        {
            exit_with_msg!(usr => $($x),*);
        }
    }};
    (
        not_exists $file:expr => $($x:expr),*
    ) => {{
        if !file_utils::check_if_file_exists($file) {
            exit_with_msg!(usr => $($x),*);
        }
    }}
}

macro_rules! get_pr_verbosity_level {
    (
        $matches:expr
    ) => {{
        match $matches.value_of("pr_verbosity_level") {
            None    => progress_report::PRVerbosityLevel::L2,
            Some(x) => match progress_report::string_to_verbosity_level(x) {
                Ok(x)  => x,
                Err(_) => exit_with_msg!(usr => "Invalid progress report verbosity level")
            }
        }
    }}
}

macro_rules! parse_uid {
    (
        $buf:expr, $uid:expr
    ) => {{
        use misc_utils::HexError::*;
        match misc_utils::hex_string_to_bytes($uid) {
            Ok(x) => {
                if x.len() != SBX_FILE_UID_LEN {
                    exit_with_msg!(usr => "UID must be {} bytes({} hex characters) in length",
                                   SBX_FILE_UID_LEN,
                                   SBX_FILE_UID_LEN * 2);
                }

                $buf.copy_from_slice(&x);
            },
            Err(InvalidHexString) => {
                exit_with_msg!(usr => "UID provided is not a valid hex string");
            },
            Err(InvalidLen) => {
                exit_with_msg!(usr => "UID provided does not have the correct number of hex digits, provided : {}, need : {}",
                               $uid.len(),
                               SBX_FILE_UID_LEN * 2);
            }
        }
    }}
}

pub fn pr_verbosity_level_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("pr_verbosity_level")
        .value_name("LEVEL")
        .long("pv")
        .takes_value(true)
        .help("Progress report verbosity level, one of :
          0 (show nothing)
          1 (only show progress stats when done)
(default) 2 (show both progress bar and progress stats)
This only affects progress text printing.")
}

pub fn verbose_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("verbose")
        .short("v")
        .long("verbose")
}

pub fn force_misalign_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("force_misalign")
        .long("force-misalign")
        .help("Disable automatic rounding down of FROM-BYTE. This is not normally
used and is only intended for data recovery or related purposes.")
}

pub fn from_byte_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("from_pos")
        .value_name("FROM-BYTE")
        .long("from")
        .visible_alias("skip-to")
        .takes_value(true)
        .help("Start from byte FROM-BYTE. The position is automatically rounded
down to the closest multiple of 128 bytes, after adding the bytes
processed field from the log file(if specified). If this option is
not specified, defaults to the start of file. Negative values are
treated as 0. If FROM-BYTE exceeds the largest possible
position(file size - 1), then it will be treated as (file size - 1).
The rounding procedure is applied after all auto-adjustments.")
}

pub fn to_byte_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("to_pos")
        .value_name("TO-BYTE")
        .long("to")
        .takes_value(true)
        .help("Last position to try to decode a block. If not specified, defaults
to the end of file. Negative values are treated as 0. If TO-BYTE is
smaller than FROM-BYTE, then it will be treated as FROM-BYTE.")
}

pub fn no_meta_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("no_meta")
        .long("no-meta")
        .help("Use first whatever valid block as reference block. Use this when
the container does not have metadata block or when you are okay
with using a data block as reference block.")
}

pub fn burst_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("burst")
        .value_name("LEVEL")
        .long("burst")
        .takes_value(true)
        .help("Burst error resistance level")
}

pub fn report_ref_block_info(ref_block_pos : u64,
                             ref_block     : &sbx_block::Block) {
    println!();
    println!("Using {} block as reference block, located at byte {} (0x{:X})",
             if ref_block.is_meta() { "metadata" }
             else                   { "data"     },
             ref_block_pos,
             ref_block_pos);
    println!();
}
