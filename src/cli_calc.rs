use sbx_specs::{SBX_FILE_UID_LEN,
                Version,
                ver_to_usize,
                ver_to_block_size,
                ver_to_data_size,
                ver_uses_rs};

use clap::*;
use cli_utils::*;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("calc")
        .about("Compute and display detailed information given a configuration")
        .arg(sbx_version_arg())
        .arg(Arg::with_name("no_meta")
             .long("no-meta")
             .help("Skip metadata block in the calculations. Metadata block is
never skipped for version 17, 18, 19.
This means this option has no effect for version 17, 18, 19."))
        .arg(rs_data_arg())
        .arg(rs_parity_arg())
        .arg(burst_arg())
}

pub fn calc<'a>(matches : &ArgMatches<'a>) -> i32 {
}
