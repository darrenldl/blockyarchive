extern crate clap;
use clap::*;

extern crate rsbx_lib;
use rsbx_lib::*;

fn real_main () -> i32 {
    let matches = App::new("rsbx")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Darren Ldl <darrenldldev@gmail.com>")
        .about("Archive with forward error correction and sector level recoverability
IMPORTANT : Please note that this is the last version of this software to be released under the name rsbx,
future releases will be published under the name blkar. See project repo for details.")
        .subcommand(cli_encode::sub_command())
        .subcommand(cli_decode::sub_command())
        .subcommand(cli_rescue::sub_command())
        .subcommand(cli_show::sub_command())
        .subcommand(cli_repair::sub_command())
        .subcommand(cli_check::sub_command())
        .subcommand(cli_sort::sub_command())
        .subcommand(cli_calc::sub_command())
        .get_matches();

    if      let Some(matches) = matches.subcommand_matches("encode") {
        rsbx_lib::cli_encode::encode(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("decode") {
        rsbx_lib::cli_decode::decode(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("rescue") {
        rsbx_lib::cli_rescue::rescue(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("show") {
        rsbx_lib::cli_show::show(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("repair") {
        rsbx_lib::cli_repair::repair(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("check") {
        rsbx_lib::cli_check::check(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("sort") {
        rsbx_lib::cli_sort::sort(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("calc") {
        rsbx_lib::cli_calc::calc(matches)
    }
    else {
        exit_with_msg!(ok rsbx_lib::json_printer::JSONPrinter::new(false, rsbx_lib::output_channel::OutputChannel::Stdout)
                       => "Invoke with -h or --help for help message\n");
    }
}

fn main() {
    std::process::exit(real_main())
}
