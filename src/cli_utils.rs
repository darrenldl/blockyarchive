use clap::*;
use sbx_block;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ctrlc;

pub fn in_file_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("in_file")
        .value_name("INFILE")
        .required(true)
        .index(1)
}

pub fn out_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("out")
        .value_name("OUT")
        .index(2)
}

pub fn out_file_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("out_file")
        .value_name("OUTFILE")
        .index(2)
}

pub fn out_dir_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("out_dir")
        .value_name("OUTDIR")
        .index(2)
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
rejected. If FROM-BYTE exceeds the largest possible
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
    println!("Using {} block as reference block, located at byte {} (0x{:X})",
             if ref_block.is_meta() { "metadata" }
             else                   { "data"     },
             ref_block_pos,
             ref_block_pos);
}

pub fn guess_burst_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("guess_burst")
        .long("guess-burst")
        .help("Guess burst error resistance level at start.
Note that this requires scanning for a reference block, and may
go through the entire file as a result.
This operation does not respect the misalignment and range requirements.")
}

pub fn setup_ctrlc_handler() -> Arc<AtomicBool> {
    let stop_flag         = Arc::new(AtomicBool::new(false));
    let handler_stop_flag = Arc::clone(&stop_flag);

    ctrlc::set_handler(move || {
        handler_stop_flag.store(true, Ordering::SeqCst);
        println!("Interrupted");
    }).expect("Failed to set Ctrl-C handler");

    println!("Press Ctrl-C to interrupt");

    stop_flag
}
