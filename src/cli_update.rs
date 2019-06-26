use crate::update_core;
use crate::update_core::Param;

use crate::cli_utils::*;
use clap::*;

use crate::sbx_block::Metadata;
use crate::sbx_block::MetadataID;

use crate::multihash;

use crate::json_printer::BracketType;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("update")
        .about("Update SBX container metadata")
        .arg(in_file_arg().help("SBX container to update"))
        .arg(pr_verbosity_level_arg())
        .arg(burst_arg().help(
            "Burst error resistance level used by the container.
Use this if the level used by the container is above 1000,
as blkar will only guess up to 1000. Or use this when blkar
fails to guess correctly.",
        ))
        .arg(
            verbose_arg()
                .help("Show reference block info, and changes made in each metadata block"),
        )
        .arg(
            Arg::with_name("skip_warning")
                .short("y")
                .long("skip-warning")
                .help("Skip warning about in-place updates"),
        )
        .arg(dry_run_arg().help("Only do updates in memory. The container will not be modified."))
        .arg(json_arg().help(
            "Output information in JSON format. Note that blkar does not
guarantee the JSON data to be well-formed if blkar is interrupted.
This also implies --skip-warning, and changes progress report text
(if any) to be in JSON.",
        ))
        .arg(
            Arg::with_name("fnm")
                .value_name("NAME")
                .takes_value(true)
                .long("fnm")
                .help("New file name"),
        )
        .arg(
            Arg::with_name("snm")
                .value_name("NAME")
                .takes_value(true)
                .long("snm")
                .help("New SBX container name"),
        )
        .arg(
            Arg::with_name("no_fnm")
                .long("no-fnm")
                .help("Remove file name")
                .conflicts_with("fnm"),
        )
        .arg(
            Arg::with_name("no_snm")
                .long("no-snm")
                .help("Remove SBX container name")
                .conflicts_with("snm"),
        )
        .arg(
            Arg::with_name("hash_type")
                .long("hash")
                .value_name("HASH-TYPE")
                .help(
                    "Rehash the stored data. If HSH field already exists, then it
is replaced with the new hash result. Otherwise a HSH field is
added for the new hash result.
HASH-TYPE may be one of (case-insensitive) :
sha1
sha256
sha512
blake2b-256
blake2b-512
blake2s-128
blake2s-256",
                ),
        )
        .arg(
            Arg::with_name("no_hsh")
                .long("no-hsh")
                .help("Remove SBX container stored data hash")
                .conflicts_with("hash_type"),
        )
}

pub fn update<'a>(matches: &ArgMatches<'a>) -> i32 {
    let json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(None, BracketType::Curly);

    let in_file = get_in_file!(matches, json_printer);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let burst = get_burst_opt!(matches, json_printer);

    let hash_type = match matches.value_of("hash_type") {
        None => None,
        Some(x) => match multihash::string_to_hash_type(x) {
            Ok(x) => Some(x),
            Err(_) => exit_with_msg!(usr json_printer => "Invalid hash type"),
        },
    };

    let metas_to_update = {
        let mut res = smallvec![];

        if let Some(x) = matches.value_of("fnm") {
            res.push(Metadata::FNM(x.to_string()))
        }
        if let Some(x) = matches.value_of("snm") {
            res.push(Metadata::SNM(x.to_string()))
        }
        if let Some(_) = matches.value_of("hash_type") {
            let hash_type = hash_type.unwrap();
            let dummy_hash = multihash::hash::Ctx::new(hash_type)
                .unwrap()
                .finish_into_bytes();
            res.push(Metadata::HSH((hash_type, dummy_hash)))
        }

        res
    };

    let metas_to_remove = {
        let mut res = smallvec![];

        if matches.is_present("no_fnm") {
            res.push(MetadataID::FNM)
        }
        if matches.is_present("no_snm") {
            res.push(MetadataID::SNM)
        }
        if matches.is_present("no_hsh") {
            res.push(MetadataID::HSH)
        }

        res
    };

    if matches.is_present("dry_run") && !json_printer.json_enabled() {
        print_block!(
            "Note : This is a dry run only, the container is not modified.";
            "";
        );
    }

    if !matches.is_present("skip_warning")
        && !matches.is_present("dry_run")
        && !json_printer.json_enabled()
    {
        print_block!(
            "Warning :";
            "";
            "    Update mode modifies the SBX container in-place.";
            "";
            "    This may cause irreversible damage to the container and prohibit normal";
            "    functioning, depending on your workflow.";
            "";
            "    It is advisable to do a dry run first via supplying the --dry-run flag";
            "    and examine the changes before actually updating the container.";
            "";
        );

        ask_if_wish_to_continue!();
    }

    let mut param = Param::new(
        in_file,
        matches.is_present("dry_run"),
        metas_to_update,
        metas_to_remove,
        &json_printer,
        hash_type,
        matches.is_present("verbose"),
        pr_verbosity_level,
        burst,
    );
    match update_core::update_file(&mut param) {
        Ok(Some(s)) => exit_with_msg!(ok json_printer => "{}", s),
        Ok(None) => exit_with_msg!(ok json_printer => ""),
        Err(e) => exit_with_msg!(op json_printer => "{}", e),
    }
}
