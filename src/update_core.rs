use std::fmt;
use std::sync::{Arc, Mutex};

use crate::progress_report::*;

use smallvec::SmallVec;

use crate::sbx_specs::{
    ver_to_block_size, ver_to_usize, ver_uses_rs, Version, SBX_LARGEST_BLOCK_SIZE,
};

use crate::cli_utils::setup_ctrlc_handler;

use std::io::SeekFrom;

use crate::sbx_block;
use crate::sbx_block::Block;
use crate::sbx_block::Metadata;

use crate::json_printer::{BracketType, JSONPrinter};

use crate::file_reader::{FileReader, FileReaderParam};

use crate::general_error::Error;

use crate::block_utils::RefBlockChoice;
use crate::sbx_block::BlockType;

use crate::time_utils;

pub struct Param {
    in_file: String,
    dry_run: bool,
    metas_to_update: SmallVec<[Metadata; 8]>,
    json_printer: Arc<JSONPrinter>,
    verbose: bool,
    pr_verbosity_level: PRVerbosityLevel,
    burst: Option<usize>,
}

impl Param {
    pub fn new(
        in_file: &str,
        dry_run: bool,
        metas_to_update: SmallVec<[Metadata; 8]>,
        json_printer: &Arc<JSONPrinter>,
        verbose: bool,
        pr_verbosity_level: PRVerbosityLevel,
        burst: Option<usize>,
    ) -> Param {
        Param {
            in_file: String::from(in_file),
            dry_run,
            metas_to_update,
            json_printer: Arc::clone(json_printer),
            verbose,
            pr_verbosity_level,
            burst,
        }
    }
}

#[derive(Clone)]
pub struct Stats {
    version: Version,
    pub meta_blocks_updated: u64,
    pub meta_blocks_failed: u64,
    total_meta_blocks: u64,
    start_time: f64,
    end_time: f64,
    json_printer: Arc<JSONPrinter>,
}

impl Stats {
    pub fn new(
        ref_block: &Block,
        data_par_burst: Option<(usize, usize, usize)>,
        json_printer: &Arc<JSONPrinter>,
    ) -> Stats {
        let total_meta_blocks =
            sbx_block::calc_meta_block_all_write_pos_s(ref_block.get_version(), data_par_burst)
                .len() as u64;

        Stats {
            version: ref_block.get_version(),
            meta_blocks_updated: 0,
            meta_blocks_failed: 0,
            total_meta_blocks,
            start_time: 0.,
            end_time: 0.,
            json_printer: Arc::clone(json_printer),
        }
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 {
        &mut self.start_time
    }

    fn end_time_mut(&mut self) -> &mut f64 {
        &mut self.end_time
    }

    fn units_so_far(&self) -> u64 {
        self.meta_blocks_updated + self.meta_blocks_failed
    }

    fn total_units(&self) -> u64 {
        self.total_meta_blocks
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let block_size = ver_to_block_size(self.version);
        let time_elapsed = (self.end_time - self.start_time) as i64;
        let (hour, minute, second) = time_utils::seconds_to_hms(time_elapsed);

        let json_printer = &self.json_printer;

        json_printer.write_open_bracket(f, Some("stats"), BracketType::Curly)?;

        write_maybe_json!(
            f,
            json_printer,
            "SBX version                              : {}",
            ver_to_usize(self.version)
        )?;
        write_maybe_json!(f, json_printer, "Block size used in updating              : {}", block_size                            => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of metadata blocks processed      : {}", self.units_so_far()                   => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of metadata blocks updated        : {}", self.meta_blocks_updated              => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of metadata blocks update failed  : {}", self.meta_blocks_failed               => skip_quotes)?;
        write_maybe_json!(
            f,
            json_printer,
            "Time elapsed                             : {:02}:{:02}:{:02}",
            hour,
            minute,
            second
        )?;

        Ok(())
    }
}

fn update_metas(block: &mut Block, metas: &[Metadata]) {
    assert!(block.is_meta());
    block.update_metas(metas).unwrap();
}

fn print_block_info_and_meta_changes(param: &Param, pos: u64, old_meta: &[Metadata]) {
    let json_printer = &param.json_printer;

    json_printer.print_open_bracket(None, BracketType::Curly);

    print_maybe_json!(json_printer, "pos : {}", pos);
    json_printer.print_open_bracket(Some("changes"), BracketType::Square);
    for m in param.metas_to_update.iter() {
        let id = sbx_block::meta_to_meta_id(m);
        let old = sbx_block::get_meta_ref_by_meta_id(old_meta, id);
        if json_printer.json_enabled() {
            json_printer
                .print_open_bracket(Some(sbx_block::meta_id_to_str(id)), BracketType::Curly);
            match old {
                None => print_maybe_json!(json_printer, "from : null"),
                Some(old) => print_maybe_json!(json_printer, "from : {}", old),
            };
            print_maybe_json!(json_printer, "to : {}", m);
            json_printer.print_close_bracket();
        } else {
            match old {
                None => println!("N/A => {}", m),
                Some(old) => println!("{} => {}", old, m),
            }
        }
    }
    json_printer.print_close_bracket();

    json_printer.print_close_bracket();
}

pub fn update_file(param: &Param) -> Result<Option<Stats>, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let json_printer = &param.json_printer;

    let (ref_block_pos, ref_block) = get_ref_block!(no_force_misalign =>
                                                    param,
                                                    None,
                                                    None,
                                                    json_printer,
                                                    RefBlockChoice::MustBe(BlockType::Meta),
                                                    ctrlc_stop_flag
    );

    let version = ref_block.get_version();

    let pred = block_pred_same_ver_uid!(ref_block);

    let burst = get_burst_or_guess!(no_offset => param, ref_block_pos, ref_block);

    let data_par_burst = if ver_uses_rs(version) {
        Some((
            get_RSD_from_ref_block!(ref_block_pos, ref_block, "updater"),
            get_RSP_from_ref_block!(ref_block_pos, ref_block, "updater"),
            burst,
        ))
    } else {
        None
    };

    let stats = Arc::new(Mutex::new(Stats::new(
        &ref_block,
        data_par_burst,
        json_printer,
    )));

    let mut reader = FileReader::new(
        &param.in_file,
        FileReaderParam {
            write: !param.dry_run,
            buffered: true,
        },
    )?;

    let mut block = Block::dummy();
    let mut buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let reporter = Arc::new(ProgressReporter::new(
        &stats,
        "SBX metadata block updating progress",
        "blocks",
        param.pr_verbosity_level,
        param.json_printer.json_enabled(),
    ));

    reporter.start();

    json_printer.print_open_bracket(Some("metadata updates"), BracketType::Square);
    for &p in sbx_block::calc_meta_block_all_write_pos_s(version, data_par_burst).iter() {
        break_if_atomic_bool!(ctrlc_stop_flag);

        reader.seek(SeekFrom::Start(p))?;
        let read_res = reader.read(sbx_block::slice_buf_mut(version, &mut buffer))?;

        break_if_eof_seen!(read_res);

        let block_okay = match block.sync_from_buffer(&buffer, Some(&pred)) {
            Ok(()) => true,
            Err(_) => false,
        } && block.is_meta();

        if block_okay {
            let old_metas = block.metas().unwrap().clone();

            update_metas(&mut block, &param.metas_to_update);

            if param.verbose {
                print_block_info_and_meta_changes(param, p, &old_metas);
            }

            stats.lock().unwrap().meta_blocks_updated += 1;
        } else {
            stats.lock().unwrap().meta_blocks_failed += 1;
        }
    }
    json_printer.print_close_bracket();

    reporter.stop();

    let stats = stats.lock().unwrap().clone();

    Ok(Some(stats))
}
