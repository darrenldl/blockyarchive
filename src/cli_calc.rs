use sbx_specs::{Version,
                ver_to_usize,
                ver_to_block_size,
                ver_to_data_size,
                ver_uses_rs};

use std::str::FromStr;

use file_utils;

use json_printer::{JSONPrinter,
                   BracketType};

use clap::*;
use cli_utils::*;
pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("calc")
        .about("Calculate and display detailed information given an encoding configuration")
        .arg(Arg::with_name("in_file_size")
             .value_name("INFILE-SIZE")
             .required(true)
             .index(1)
             .help("Input file size"))
        .arg(sbx_version_arg())
        .arg(Arg::with_name("no_meta")
             .long("no-meta")
             .help("Skip metadata block in the calculations. Metadata block is
never skipped for version 17, 18, 19.
This means this option has no effect for version 17, 18, 19."))
        .arg(rs_data_arg())
        .arg(rs_parity_arg())
        .arg(burst_arg())
        .arg(json_arg())
}

pub fn calc<'a>(matches : &ArgMatches<'a>) -> i32 {
    let mut json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(BracketType::Curly);

    let version = get_version!(matches, json_printer);

    let meta_enabled = Some(get_meta_enabled!(matches));

    let in_file_size =
        match u64::from_str(matches.value_of("in_file_size").unwrap()) {
            Ok(x)  => x,
            Err(_) => exit_with_msg!(usr json_printer => "Invalid file size")
        };

    let data_par_burst =
        if ver_uses_rs(version) {
            // deal with RS related options
            let data_shards   = get_data_shards!(matches, version, json_printer);
            let parity_shards = get_parity_shards!(matches, version, json_printer);

            check_data_parity_shards!(data_shards, parity_shards, json_printer);

            let burst = get_burst_or_zero!(matches, json_printer);

            Some((data_shards, parity_shards, burst))
        } else {
            None
        };

    let out_file_size =
        file_utils::from_orig_file_size::calc_container_size(version,
                                                             meta_enabled,
                                                             data_par_burst,
                                                             in_file_size);

    let total_block_count =
        file_utils::from_orig_file_size::calc_total_block_count_exc_burst_gaps(version,
                                                                               meta_enabled,
                                                                               data_par_burst,
                                                                               in_file_size);

    let meta_block_count =
        file_utils::calc_meta_block_count_exc_burst_gaps(version,
                                                         meta_enabled,
                                                         data_par_burst);

    let (data_only_block_count, parity_block_count) =
        file_utils::from_orig_file_size::calc_data_only_and_parity_block_count_exc_burst_gaps(version,
                                                                                              data_par_burst,
                                                                                              in_file_size);

    print_if_not_json!(json_printer,    "SBX container general info");
    print_if_not_json!(json_printer,    "========================================");
    if ver_uses_rs(version) {
        print_maybe_json!(json_printer, "    SBX container version        : {}",
                          ver_to_usize(version));
    } else {
        print_maybe_json!(json_printer, "    SBX container version        : {}", ver_to_usize(version));
    }
    print_maybe_json!(json_printer,     "    SBX container block size     : {}", ver_to_block_size(version) => skip_quotes);
    print_maybe_json!(json_printer,     "    SBX container data  size     : {}", ver_to_data_size(version)  => skip_quotes);

    print_if_not_json!(json_printer, "");

    print_if_not_json!(json_printer,    "SBX block distribution");
    print_if_not_json!(json_printer,    "========================================");
    if ver_uses_rs(version) {
        print_maybe_json!(json_printer, "    Metadata    block count      : {}", meta_block_count      => skip_quotes);
        print_maybe_json!(json_printer, "    Data only   block count      : {}", data_only_block_count => skip_quotes);
        print_maybe_json!(json_printer, "    Data parity block count      : {}", parity_block_count    => skip_quotes);
        print_maybe_json!(json_printer, "    Total       block count      : {}", total_block_count     => skip_quotes);
    } else {
        print_maybe_json!(json_printer, "    Metadata block count         : {}", meta_block_count      => skip_quotes);
        print_maybe_json!(json_printer, "    Data     block count         : {}", data_only_block_count => skip_quotes);
        print_maybe_json!(json_printer, "    Total    block count         : {}", total_block_count     => skip_quotes);
    }

    print_if_not_json!(json_printer, "");

    print_if_not_json!(json_printer,      "Error correction info");
    print_if_not_json!(json_printer,      "========================================");
    if ver_uses_rs(version) {
        print_maybe_json!(json_printer,   "    RS data   shard count        : {}", data_par_burst.unwrap().0  => skip_quotes);
        print_maybe_json!(json_printer,   "    RS parity shard count        : {}", data_par_burst.unwrap().1  => skip_quotes);
        print_maybe_json!(json_printer,   "    Burst error resistance level : {}", data_par_burst.unwrap().2  => skip_quotes);
    } else {
        print_maybe_json!(json_printer,   "    RS data   shard count        : {}",
                          null_if_json_else!(json_printer, "version does not use RS")                         => skip_quotes);
        print_maybe_json!(json_printer,   "    RS parity shard count        : {}",
                          null_if_json_else!(json_printer, "version does not use RS")                         => skip_quotes);
        print_maybe_json!(json_printer,   "    Burst error resistance level : {}",
                          null_if_json_else!(json_printer, "version does not support burst error resistance") => skip_quotes);
    }

    print_if_not_json!(json_printer, "");

    if ver_uses_rs(version) && !json_printer.json_enabled() {
        let (data, par, burst) = data_par_burst.unwrap();

        let block_size = ver_to_block_size(version);

        print!("Error correction parameters interpretation");
        println!("========================================");
        if burst == 0 {
            print_block!(
                "    The container can tolerate {} SBX block corruptions", par;
                "    in any block set.";
                "";
                "    A block set consists of {} blocks({} bytes).", (data + par), (data + par) * block_size;
                "";
                "    In total, {} blocks({} bytes) may be corrupted in", par, par * block_size;
                "    any block set.";
            );
        } else {
            if burst == 1 {
                print_block!(
                    "    Warning :";
                    "";
                    "        Burst error resistance level of {} may not provide", burst;
                    "        meaningful resistance.";
                    "";
                );
            }

            let block_set_size       = data + par;
            let super_block_set_size = (data + par) * burst;

            print_block!("    The container can tolerate {} burst SBX block corruptions in", par;
                         "    any super block set({} interleaved block sets).", burst;
                         "";
                         "    A block set consists of {} blocks({} bytes).", block_set_size, block_set_size * block_size;
                         "";
                         "    A super block set consists of {} blocks({} bytes).", super_block_set_size, super_block_set_size * block_size;
                         "";
                         "    Each burst error may be up to {} blocks({} bytes) in size.", burst, burst * block_size;
                         "";
                         "    In total, {} sets of {} consecutive blocks({} bytes) may be", par, burst, burst * block_size;
                         "    corrupted in any super block set.";
                         "";
                         "    The sets of corrupted blocks may be connected as well, so the";
                         "    largest single burst error that can be tolerated in any super";
                         "    block set is {} blocks({} bytes) in size.", par * burst, par * burst * block_size;
                         "";
                         "    Note that the actual tolerance depends on the behaviour of";
                         "    the file system.";
            );
        }

        println!();
    }

    print_if_not_json!(json_printer, "File and container size");
    print_if_not_json!(json_printer, "========================================");
    print_maybe_json!(json_printer,  "    File size                    : {}", in_file_size  => skip_quotes);
    print_maybe_json!(json_printer,  "    SBX container size           : {}", out_file_size => skip_quotes);

    exit_with_msg!(ok json_printer => "")
}
