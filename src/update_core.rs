use std::fmt;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use crate::progress_report::*;

use smallvec::SmallVec;

use crate::sbx_specs::{ver_to_block_size, ver_to_usize, Version, SBX_LARGEST_BLOCK_SIZE};

use crate::cli_utils::setup_ctrlc_handler;

use std::io::SeekFrom;

use crate::sbx_block;
use crate::sbx_block::Block;
use crate::sbx_block::{Metadata, MetadataID};

use crate::json_printer::{BracketType, JSONPrinter};

use crate::file_reader::{FileReader, FileReaderParam};

use crate::general_error::Error;

use crate::block_utils::RefBlockChoice;
use crate::sbx_block::BlockType;

use crate::multihash;

use crate::time_utils;

pub struct Param {
    in_file: String,
    dry_run: bool,
    metas_to_update: SmallVec<[Metadata; 8]>,
    metas_to_remove: SmallVec<[MetadataID; 8]>,
    json_printer: Arc<JSONPrinter>,
    hash_type: Option<multihash::HashType>,
    verbose: bool,
    pr_verbosity_level: PRVerbosityLevel,
    burst: Option<usize>,
}

impl Param {
    pub fn new(
        in_file: &str,
        dry_run: bool,
        metas_to_update: SmallVec<[Metadata; 8]>,
        metas_to_remove: SmallVec<[MetadataID; 8]>,
        json_printer: &Arc<JSONPrinter>,
        hash_type: Option<multihash::HashType>,
        verbose: bool,
        pr_verbosity_level: PRVerbosityLevel,
        burst: Option<usize>,
    ) -> Param {
        Param {
            in_file: String::from(in_file),
            dry_run,
            metas_to_update,
            metas_to_remove,
            json_printer: Arc::clone(json_printer),
            hash_type,
            verbose,
            pr_verbosity_level,
            burst,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    version: Version,
    pub meta_blocks_updated: u64,
    pub meta_blocks_decode_failed: u64,
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
            meta_blocks_decode_failed: 0,
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
        self.meta_blocks_updated + self.meta_blocks_decode_failed
    }

    fn total_units(&self) -> Option<u64> {
        Some(self.total_meta_blocks)
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
        write_maybe_json!(
            f,
            json_printer,
            "Block size used in updating                : {}",
            block_size
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of metadata blocks processed        : {}",
            self.units_so_far()
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of metadata blocks updated          : {}",
            self.meta_blocks_updated
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of metadata blocks failed to decode : {}",
            self.meta_blocks_decode_failed
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Time elapsed                             : {:02}:{:02}:{:02}",
            hour,
            minute,
            second
        )?;
        json_printer.print_close_bracket();

        Ok(())
    }
}

fn update_metas(block: &mut Block, metas: &[Metadata]) {
    block.update_metas(metas).unwrap();
}

fn remove_metas(block: &mut Block, ids: &[MetadataID]) {
    block.remove_metas(ids).unwrap();
}

fn print_block_info_and_meta_changes(
    param: &Param,
    meta_block_count: u64,
    pos: u64,
    old_meta: &[Metadata],
) {
    let json_printer = &param.json_printer;

    json_printer.print_open_bracket(None, BracketType::Curly);

    if meta_block_count > 0 {
        print_if!(not_json => json_printer => "";);
    }
    print_maybe_json!(json_printer, "Metadata block number : {}", meta_block_count);
    print_if!(not_json => json_printer => "========================================";);

    print_maybe_json!(json_printer, "Found at byte : {}", pos);
    json_printer.print_open_bracket(Some("changes"), BracketType::Square);

    print_if!(not_json => json_printer => "";);

    let mut change_count = 0;

    for m in param.metas_to_update.iter() {
        let id = sbx_block::meta_to_meta_id(m);
        let old = sbx_block::get_meta_ref_by_meta_id(old_meta, id);
        let changed = match old {
            None => true,
            Some(old) => old != m,
        };
        if changed {
            if change_count > 0 {
                print_if!(not_json => json_printer => "";);
            }

            let field_str = format!("Field         : {}", sbx_block::meta_id_to_str(id));
            let from_str = match old {
                None => format!("From          : {}", null_if_json_else_NA!(json_printer)),
                Some(old) => format!("From          : {}", old),
            };
            let to_str = format!("To            : {}", m);

            json_printer.print_open_bracket(None, BracketType::Curly);
            print_maybe_json!(json_printer, "{}", field_str);
            print_maybe_json!(json_printer, "{}", from_str);
            print_maybe_json!(json_printer, "{}", to_str);
            json_printer.print_close_bracket();

            change_count += 1;
        }
    }
    for &id in param.metas_to_remove.iter() {
        let old = sbx_block::get_meta_ref_by_meta_id(old_meta, id);
        if let Some(old) = old {
            if change_count > 0 {
                print_if!(not_json => json_printer => "";);
            }

            json_printer.print_open_bracket(None, BracketType::Curly);
            print_maybe_json!(
                json_printer,
                "Field         : {}",
                sbx_block::meta_id_to_str(id)
            );
            print_maybe_json!(json_printer, "From          : {}", old);
            print_maybe_json!(
                json_printer,
                "To            : {}",
                null_if_json_else_NA!(json_printer)
            );
            json_printer.print_close_bracket();

            change_count += 1;
        }
    }
    json_printer.print_close_bracket();

    json_printer.print_close_bracket();
}

fn update_metadata_blocks(
    ctrlc_stop_flag: &AtomicBool,
    param: &Param,
    ref_block: &Block,
    json_printer: &Arc<JSONPrinter>,
    data_par_burst: Option<(usize, usize, usize)>,
    test_run: bool,
) -> Result<Stats, Error> {
    let version = ref_block.get_version();

    let header_pred = header_pred_same_ver_uid!(ref_block);

    let mut block = Block::dummy();
    let mut buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut meta_block_count: u64 = 0;

    let stats = Arc::new(Mutex::new(Stats::new(
        &ref_block,
        data_par_burst,
        json_printer,
    )));

    let reporter = Arc::new(ProgressReporter::new(
        &stats,
        if test_run {
            "SBX metadata block update testing progress"
        } else {
            "SBX metadata block update progress"
        },
        "blocks",
        param.pr_verbosity_level,
        param.json_printer.json_enabled(),
    ));

    let mut reader = FileReader::new(
        &param.in_file,
        FileReaderParam {
            write: !param.dry_run,
            buffered: false,
        },
    )?;

    let mut err = None;

    if param.verbose && !test_run {
        json_printer.print_open_bracket(Some("metadata changes"), BracketType::Square);
    }
    for &p in sbx_block::calc_meta_block_all_write_pos_s(version, data_par_burst).iter() {
        break_if_atomic_bool!(ctrlc_stop_flag);

        if let Some(_) = err {
            break;
        }

        reader.seek(SeekFrom::Start(p))?;
        let read_res = reader.read(sbx_block::slice_buf_mut(version, &mut buffer))?;

        break_if_eof_seen!(read_res);

        break_if_eof_seen!(read_res);

        let block_okay = match block.sync_from_buffer(&buffer, Some(&header_pred), None) {
            Ok(()) => true,
            Err(_) => false,
        } && block.is_meta();

        if block_okay {
            let old_metas = block.metas().unwrap().clone();

            update_metas(&mut block, &param.metas_to_update);
            remove_metas(&mut block, &param.metas_to_remove);

            match block.sync_to_buffer(None, &mut buffer) {
                Ok(()) => {
                    if param.verbose {
                        pause_reporter!(reporter =>
                                        print_block_info_and_meta_changes(param, meta_block_count, p, &old_metas););
                    }

                    if !param.dry_run {
                        reader.seek(SeekFrom::Start(p))?;
                        reader.write(sbx_block::slice_buf(version, &buffer))?;
                    }

                    stats.lock().unwrap().meta_blocks_updated += 1;
                }
                Err(e) => match e {
                    sbx_block::Error::TooMuchMetadata(_) => {
                        let err_msg = format!("Failed to update metadata block number {} at {} (0x{:X}) due to too much metadata",
                                              meta_block_count, p, p);
                        err = Some(Error::with_msg(&err_msg));
                    }
                    _ => unreachable!(),
                },
            }
        } else {
            stats.lock().unwrap().meta_blocks_decode_failed += 1;
        }

        meta_block_count += 1;
    }
    if param.verbose && !test_run {
        json_printer.print_close_bracket();
    }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();

    match err {
        None => Ok(stats),
        Some(e) => Err(e),
    }
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

    let data_par_burst =
        get_data_par_burst!(no_offset => param, ref_block_pos, ref_block, "update");

    match update_metadata_blocks(&ctrlc_stop_flag,
                           param,
                           &ref_block,
                           &json_printer,
                           data_par_burst,
                           false,
    ) {
        Ok(s) => Ok(Some(s)),
        Err(e) => Err(e),
    }
}
