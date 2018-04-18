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
}

pub fn sbx_version_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("sbx_version")
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
          19 (0x13) |     4096 bytes |      enabled |              supported |")
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

pub fn report_ref_block_info(json_enabled  : bool,
                             no_comma      : Option<bool>,
                             ref_block_pos : u64,
                             ref_block     : &sbx_block::Block) {
    if json_enabled {
        print_json_field!("reference block type",
                          if ref_block.is_meta() { "metadata" }
                          else                   { "data"     }, false, no_comma.unwrap_or(true));
        print_maybe_json!(json_enabled, "reference block location : {}", ref_block_pos);
    } else {
        println!("Using {} block as reference block, located at byte {} (0x{:X})",
                 if ref_block.is_meta() { "metadata" }
                 else                   { "data"     },
                 ref_block_pos,
                 ref_block_pos);
    }
}

pub fn guess_burst_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("guess_burst")
        .long("guess-burst")
        .help("Guess burst error resistance level(guesses up to 1000) at start.
Note that this requires scanning for a reference block, and may
go through the entire file as a result, thus may cause major delay
before scanning for metadata blocks.
This operation does not respect the misalignment and range requirements.")
}

pub fn json_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("json")
        .long("json")
        .help("Output information in JSON format. Note that rsbx does not
guarantee the JSON data to be well form if rsbx is interrupted.")
}

pub fn setup_ctrlc_handler(json_enabled : bool) -> Arc<AtomicBool> {
    let stop_flag         = Arc::new(AtomicBool::new(false));
    let handler_stop_flag = Arc::clone(&stop_flag);

    ctrlc::set_handler(move || {
        handler_stop_flag.store(true, Ordering::SeqCst);
        if !json_enabled {
            println!("Interrupted");
        }
    }).expect("Failed to set Ctrl-C handler");

    if !json_enabled {
        println!("Press Ctrl-C to interrupt");
    }

    stop_flag
}
