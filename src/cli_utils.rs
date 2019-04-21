#![allow(dead_code)]
use crate::sbx_block;
use clap::*;
use ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::sbx_specs::ver_to_usize;

use crate::json_printer::{BracketType, JSONPrinter};

pub fn in_file_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("in_file")
        .value_name("INFILE")
        .required(true)
        .index(1)
}

pub fn out_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("out").value_name("OUT").index(2)
}

pub fn out_file_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("out_file").value_name("OUTFILE").index(2)
}

pub fn out_dir_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("out_dir").value_name("OUTDIR").index(2)
}

pub fn pr_verbosity_level_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("pr_verbosity_level")
        .value_name("LEVEL")
        .long("pv")
        .takes_value(true)
        .help(
            "Progress report verbosity level, one of :
(default in JSON mode)     0 (show nothing)
                           1 (only show progress stats when done)
(default in non-JSON mode) 2 (show both progress bar and progress stats)
This only affects progress text printing",
        )
}

pub fn verbose_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("verbose").short("v").long("verbose")
}

pub fn force_misalign_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("force_misalign")
        .long("force-misalign")
        .help(
            "Disable automatic rounding down of FROM-BYTE. This is not normally
used and is only intended for data recovery or related purposes.",
        )
}

pub fn force_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("force").short("f").long("force")
}

pub fn multi_pass_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("multi_pass").long("multi-pass").help(
        "Disable truncation of OUT, and skip writing if a good block already
exists at the location. This allows writing to OUT multiple times
to update it gradually.",
    )
}

pub fn multi_pass_no_skip_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("multi_pass_no_skip")
        .long("multi-pass-no-skip")
        .help(
            "Disable truncation of OUT, write even if a good block exists at
the location. This allows writing to OUT multiple times to update
it gradually.",
        )
}

pub fn only_pick_uid_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("uid")
        .value_name("UID-HEX")
        .long("only-pick-uid")
        .takes_value(true)
        .help(
            "Only pick blocks with UID-HEX as UID. UID must be exactly 6
bytes (12 hex digits) in length.",
        )
}

pub fn from_byte_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("from_pos")
        .value_name("FROM-BYTE")
        .long("from")
        .visible_alias("skip-to")
        .takes_value(true)
}

pub const FROM_BYTE_ARG_HELP_MSG_SCAN: &str =
    "Start from byte FROM-BYTE. The position is automatically rounded
down to the closest multiple of 128 bytes, after adding the bytes
processed field from the log file (if specified). If this option is
not specified, defaults to the start of file. Negative values are
rejected. If FROM-BYTE exceeds the largest possible position
(file size - 1), then it will be treated as (file size - 1).
The rounding procedure is applied after all auto-adjustments.";

pub const FROM_BYTE_ARG_HELP_MSG_REF_BLOCK: &str =
    "Start from byte FROM-BYTE. The position is automatically rounded
down to the closest multiple of (ref block size) bytes. If this
option is not specified, defaults to the start of file. Negative
values are rejected. If FROM-BYTE exceeds the largest possible
position (file size - 1), then it will be treated as (file size - 1).
The rounding procedure is applied after all auto-adjustments.";

pub const FROM_BYTE_ARG_HELP_MSG_RAW_UNALIGNED: &str =
    "Start from byte FROM-BYTE. If this option is not specified,
defaults to the start of the file. Negative values are rejected. If
FROM-BYTE exceeds the largest possible position (file size - 1). then
it will be treated as (file size - 1).";

pub fn to_byte_inc_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("to_pos_inc")
        .value_name("TO-BYTE-INC")
        .long("to-inc")
        .takes_value(true)
        .help(
            "Last position (inclusive) to try to decode a block. If not specified,
defaults to the end of file. Negative values are rejected. If TO-BYTE
is smaller than FROM-BYTE, then it will be treated as
FROM-BYTE.",
        )
}

pub fn to_byte_exc_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("to_pos_exc")
        .value_name("TO-BYTE-EXC")
        .long("to-exc")
        .takes_value(true)
        .help(
            "Last position (exclusive) to try to decode a block. If not specified,
defaults to the end of file. Negative values are rejected. If TO-BYTE
is smaller than FROM-BYTE, then it will be treated as
FROM-BYTE.",
        )
}

pub fn guess_burst_from_byte_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("guess_burst_from_pos")
        .value_name("FROM-BYTE")
        .long("guess-burst-from")
        .takes_value(true)
        .help(
            "Position to start guessing burst error resistance level. The position
is automatically rounded down to the closest multiple of (ref block size) bytes.
If this option is not specified, defaults to the start of file.
Negative values are rejected. If FROM-BYTE exceeds the largest
possible position (file size - 1), then it will be treated as
(file size - 1). The rounding procedure is applied after all
auto-adjustments.",
        )
}

pub fn ref_from_byte_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("ref_from_pos")
        .value_name("FROM-BYTE")
        .long("ref-from")
        .takes_value(true)
        .help(
            "First position to try to search for a reference block. The position
is automatically rounded down to the closest multiple of 128 bytes.
If this option is not specified, defaults to the start of file.
Negative values are rejected. If FROM-BYTE exceeds the largest
possible position (file size - 1), then it will be treated as
(file size - 1). The rounding procedure is applied after all
auto-adjustments.",
        )
}

pub fn ref_to_byte_inc_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("ref_to_pos_inc")
        .value_name("TO-BYTE-INC")
        .long("ref-to-inc")
        .takes_value(true)
        .help(
            "Last position (inclusive) to try to search for a reference block.
If not specified, defaults to the end of file. Negative values are
rejected.",
        )
}

pub fn ref_to_byte_exc_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("ref_to_pos_exc")
        .value_name("TO-BYTE-EXC")
        .long("ref-to-exc")
        .takes_value(true)
        .help(
            "Last position (exclusive) to try to search for a reference block.
If not specified, defaults to the end of file. Negative values are
rejected.",
        )
}

pub fn no_meta_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("no_meta").long("no-meta").help(
        "Use first whatever valid block as reference block. Use this when
the container does not have metadata block or when you are okay
with using a data block as reference block.",
    )
}

pub fn burst_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("burst")
        .value_name("LEVEL")
        .long("burst")
        .takes_value(true)
}

pub fn dry_run_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("dry_run").long("dry-run")
}

pub fn sbx_version_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("sbx_version")
        .value_name("SBX-VERSION")
        .long("sbx-version")
        .takes_value(true)
        .help(
            "SBX container version, one of :
                    | SBX block size | FEC enabled | Burst error resistance |
           1        |      512 bytes |          no |          not supported |
           2        |      128 bytes |          no |          not supported |
           3        |     4096 bytes |          no |          not supported |
(default) 17 (0x11) |      512 bytes |         yes |              supported |
          18 (0x12) |      128 bytes |         yes |              supported |
          19 (0x13) |     4096 bytes |         yes |              supported |

                    | File extension |
           1        |           .sbx |
           2        |           .sbx |
           3        |           .sbx |
(default) 17 (0x11) |         .ecsbx |
          18 (0x12) |         .ecsbx |
          19 (0x13) |         .ecsbx |

Details of default option : sbx-version=17, rs-data=10, rs-parity=2, burst=10

Note : blkar will function correctly regardless of the file extension you pick,
       the ones listed above are just the defaults",
        )
}

pub fn rs_data_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("rs_data")
        .value_name("SHARD")
        .long("rs-data")
        .takes_value(true)
        .help("Reed-Solomon data shard count")
}

pub fn rs_parity_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("rs_parity")
        .value_name("SHARD")
        .long("rs-parity")
        .takes_value(true)
        .help("Reed-Solomon parity shard count")
}

pub fn report_ref_block_info(
    json_printer: &JSONPrinter,
    ref_block_pos: u64,
    ref_block: &sbx_block::Block,
) {
    if json_printer.json_enabled() {
        json_printer.print_open_bracket(Some("ref block info"), BracketType::Curly);

        print_maybe_json!(
            json_printer,
            "type : {}",
            if ref_block.is_meta() {
                "metadata"
            } else {
                "data"
            }
        );
        print_maybe_json!(
            json_printer,
            "ver : {}",
            ver_to_usize(ref_block.get_version())
        );
        print_maybe_json!(json_printer, "pos : {}", ref_block_pos);

        json_printer.print_close_bracket();
    } else {
        println!(
            "Using {} block as reference block, SBX version {}, located at byte {} (0x{:X})",
            if ref_block.is_meta() {
                "metadata"
            } else {
                "data"
            },
            ver_to_usize(ref_block.get_version()),
            ref_block_pos,
            ref_block_pos
        );
    }
}

pub fn guess_burst_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("guess_burst").long("guess-burst").help(
        "Guess burst error resistance level (guesses up to 1000) at start.
Note that this requires scanning for a reference block, and may
go through the entire file as a result, thus may cause major delay
before scanning for metadata blocks.
This operation does not respect the misalignment and range requirements.",
    )
}

pub fn json_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("json").long("json").help(
        "Output information in JSON format. Note that blkar does not
guarantee the JSON data to be well-formed if blkar is interrupted.
This also changes the progress report text (if any) to be in JSON.",
    )
}

pub fn setup_ctrlc_handler(json_enabled: bool) -> Arc<AtomicBool> {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let handler_stop_flag = Arc::clone(&stop_flag);

    ctrlc::set_handler(move || {
        handler_stop_flag.store(true, Ordering::SeqCst);
        if !json_enabled {
            eprintln!("Interrupted");
        }
    })
    .expect("Failed to set Ctrl-C handler");

    if !json_enabled {
        eprintln!("Press Ctrl-C to interrupt");
    }

    stop_flag
}
