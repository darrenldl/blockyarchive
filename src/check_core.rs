use crate::file_utils;
use crate::misc_utils;
use crate::progress_report::*;
use std::fmt;
use std::io::SeekFrom;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::misc_utils::RequiredLenAndSeekTo;

use crate::json_printer::{BracketType, JSONPrinter};

use crate::cli_utils::setup_ctrlc_handler;

use crate::file_reader::{FileReader, FileReaderParam};
use crate::sbx_block::{Block, BlockType};

use crate::general_error::Error;
use crate::sbx_specs::Version;

use crate::sbx_block;
use crate::sbx_specs::{ver_to_block_size, ver_to_usize, SBX_LARGEST_BLOCK_SIZE};

use crate::time_utils;

use crate::block_utils::RefBlockChoice;
use crate::misc_utils::{PositionOrLength, RangeEnd};

pub enum HashAction {
    NoHash,
    HashAfterCheck,
    HashOnly,
}

pub struct Param {
    ref_block_choice: RefBlockChoice,
    ref_block_from_pos: Option<u64>,
    ref_block_to_pos: Option<RangeEnd<u64>>,
    report_blank: bool,
    json_printer: Arc<JSONPrinter>,
    from_pos: Option<u64>,
    to_pos: Option<RangeEnd<u64>>,
    force_misalign: bool,
    hash_action: HashAction,
    in_file: String,
    verbose: bool,
    pr_verbosity_level: PRVerbosityLevel,
}

impl Param {
    pub fn new(
        ref_block_choice: RefBlockChoice,
        ref_block_from_pos: Option<u64>,
        ref_block_to_pos: Option<RangeEnd<u64>>,
        report_blank: bool,
        json_printer: &Arc<JSONPrinter>,
        from_pos: Option<u64>,
        to_pos: Option<RangeEnd<u64>>,
        force_misalign: bool,
        hash_action: HashAction,
        in_file: &str,
        verbose: bool,
        pr_verbosity_level: PRVerbosityLevel,
    ) -> Param {
        Param {
            ref_block_choice,
            ref_block_from_pos,
            ref_block_to_pos,
            report_blank,
            json_printer: Arc::clone(json_printer),
            from_pos,
            to_pos,
            force_misalign,
            hash_action,
            in_file: String::from(in_file),
            verbose,
            pr_verbosity_level,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    version: Version,
    block_size: u64,
    pub meta_or_par_blocks_decoded: u64,
    pub data_or_par_blocks_decoded: u64,
    pub blocks_decode_failed: u64,
    pub okay_blank_blocks: u64,
    total_blocks: u64,
    start_time: f64,
    end_time: f64,
    json_printer: Arc<JSONPrinter>,
}

impl Stats {
    pub fn new(ref_block: &Block, required_len: u64, json_printer: &Arc<JSONPrinter>) -> Stats {
        use crate::file_utils::from_container_size::calc_total_block_count;
        let total_blocks = calc_total_block_count(ref_block.get_version(), required_len);
        let version = ref_block.get_version();
        Stats {
            version,
            block_size: ver_to_block_size(version) as u64,
            meta_or_par_blocks_decoded: 0,
            data_or_par_blocks_decoded: 0,
            blocks_decode_failed: 0,
            okay_blank_blocks: 0,
            total_blocks,
            start_time: 0.,
            end_time: 0.,
            json_printer: Arc::clone(json_printer),
        }
    }

    fn blocks_so_far(&self) -> u64 {
        self.meta_or_par_blocks_decoded
            + self.data_or_par_blocks_decoded
            + self.blocks_decode_failed
            + self.okay_blank_blocks
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
        self.blocks_so_far() * self.block_size
    }

    fn total_units(&self) -> Option<u64> {
        Some(self.total_blocks * self.block_size)
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
        write_maybe_json!(f, json_printer, "Block size used in checking              : {}", block_size                         => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks processed               : {}", self.blocks_so_far()                => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks passed check (metadata) : {}", self.meta_or_par_blocks_decoded    => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks passed check (data)     : {}", self.data_or_par_blocks_decoded    => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks failed check            : {}", self.blocks_decode_failed          => skip_quotes)?;
        write_maybe_json!(
            f,
            json_printer,
            "Time elapsed                             : {:02}:{:02}:{:02}",
            hour,
            minute,
            second
        )?;

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

fn check_blocks(
    param: &Param,
    ctrlc_stop_flag: &Arc<AtomicBool>,
    required_len: u64,
    seek_to: u64,
    ref_block: &Block,
    stats: &Arc<Mutex<Stats>>,
) -> Result<(), Error> {
    let json_printer = &param.json_printer;

    let version = ref_block.get_version();

    let mut buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut reader = FileReader::new(
        &param.in_file,
        FileReaderParam {
            write: false,
            buffered: true,
        },
    )?;

    let mut block = Block::dummy();

    let reporter = Arc::new(ProgressReporter::new(
        &stats,
        "SBX block checking progress",
        "bytes",
        param.pr_verbosity_level,
        param.json_printer.json_enabled(),
    ));

    let ver_usize = ver_to_usize(version);

    let block_size = ver_to_block_size(version);

    let mut block_pos: u64;
    let mut bytes_processed: u64 = 0;

    let header_pred = header_pred_same_ver_uid!(ref_block);

    reporter.start();

    // seek to calculated position
    reader.seek(SeekFrom::Start(seek_to))?;

    if param.verbose {
        json_printer.print_open_bracket(Some("blocks failed"), BracketType::Square);
    }

    loop {
        let mut stats = stats.lock().unwrap();

        break_if_atomic_bool!(ctrlc_stop_flag);

        break_if_reached_required_len!(bytes_processed, required_len);

        let read_res = reader.read(sbx_block::slice_buf_mut(version, &mut buffer))?;

        block_pos = bytes_processed;
        bytes_processed += read_res.len_read as u64;

        break_if_eof_seen!(read_res);

        match block.sync_from_buffer(&buffer, Some(&header_pred), None) {
            Ok(_) => match block.block_type() {
                BlockType::Meta => {
                    stats.meta_or_par_blocks_decoded += 1;
                }
                BlockType::Data => {
                    stats.data_or_par_blocks_decoded += 1;
                }
            },
            Err(_) => {
                // only report error if the buffer is not completely blank
                // unless report blank is true
                if misc_utils::buffer_is_blank(sbx_block::slice_buf(version, &buffer)) {
                    if param.report_blank {
                        if json_printer.json_enabled() {
                            if param.verbose {
                                json_printer.print_open_bracket(None, BracketType::Curly);

                                print_maybe_json!(json_printer, "pos : {}", block_pos);

                                json_printer.print_close_bracket();
                            }
                        } else {
                            print_if!(verbose => param, reporter =>
                                      "Block failed check, version : {}, block size : {}, at byte {} (0x{:X})",
                                      ver_usize,
                                      block_size,
                                      block_pos,
                                      block_pos;);
                        }

                        stats.blocks_decode_failed += 1;
                    } else {
                        stats.okay_blank_blocks += 1;
                    }
                } else {
                    stats.blocks_decode_failed += 1;
                }
            }
        }
    }

    if param.verbose {
        json_printer.print_close_bracket();
    }

    if stats.lock().unwrap().blocks_decode_failed > 0 {
        print_if!(verbose not_json => param, reporter, json_printer => "";);
    }

    reporter.stop();

    Ok(())
}

// fn check_hash(
//     param: &Param,
//     ctrlc_stop_flag: &Arc<AtomicBool>,
//     required_len: u64,
//     seek_to: u64,
//     ref_block: &Block,
//     stats: &Arc<Mutex<HashStats>>,
// ) -> Result<(), Error> {
//     Ok(())
// }

pub fn check_file(param: &Param) -> Result<Option<Stats>, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let (_, ref_block) = get_ref_block!(param, &param.json_printer, ctrlc_stop_flag);

    let file_size = file_utils::get_file_size(&param.in_file)?;

    // calulate length to read and position to seek to
    let RequiredLenAndSeekTo {
        required_len,
        seek_to,
    } = misc_utils::calc_required_len_and_seek_to_from_byte_range(
        param.from_pos,
        param.to_pos,
        param.force_misalign,
        0,
        PositionOrLength::Len(file_size),
        Some(ver_to_block_size(ref_block.get_version()) as u64),
    );

    let stats = Arc::new(Mutex::new(Stats::new(
        &ref_block,
        required_len,
        &param.json_printer,
    )));

    check_blocks(
        param,
        &ctrlc_stop_flag,
        required_len,
        seek_to,
        &ref_block,
        &stats,
    )?;

    let stats = stats.lock().unwrap().clone();

    Ok(Some(stats))
}
