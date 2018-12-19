extern crate clap;
use clap::*;

extern crate blkar_lib;
use blkar_lib::*;

fn real_main () -> i32 {
    let matches = App::new("blkar")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Darren Ldl <darrenldldev@gmail.com>")
        .about("Archive with forward error correction and sector level recoverability")
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
        cli_encode::encode(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("decode") {
        cli_decode::decode(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("rescue") {
        cli_rescue::rescue(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("show") {
        cli_show::show(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("repair") {
        cli_repair::repair(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("check") {
        cli_check::check(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("sort") {
        cli_sort::sort(matches)
    }
    else if let Some(matches) = matches.subcommand_matches("calc") {
        cli_calc::calc(matches)
    }
    else {
        exit_with_msg!(ok json_printer::JSONPrinter::new(false, output_channel::OutputChannel::Stdout)
                       => "Invoke with -h or --help for help message\n");
    }
}

fn main() {
    std::process::exit(real_main())
}
