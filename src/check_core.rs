use std::sync::{Arc, Mutex};
use std::fmt;
use file_utils;
use misc_utils;
use progress_report::*;

use json_printer::{JSONPrinter,
                   BracketType};

use cli_utils::setup_ctrlc_handler;

use file_reader::{FileReader,
                  FileReaderParam};
use sbx_block::{Block,
                BlockType};

use general_error::Error;
use sbx_specs::Version;

use sbx_block;
use sbx_specs::{SBX_LARGEST_BLOCK_SIZE,
                ver_to_block_size,
                ver_to_usize};

use time_utils;
use block_utils;

use block_utils::RefBlockChoice;

use cli_utils::report_ref_block_info;

pub struct Param {
    ref_block_choice   : RefBlockChoice,
    report_blank       : bool,
    json_printer       : Arc<JSONPrinter>,
    in_file            : String,
    verbose            : bool,
    pr_verbosity_level : PRVerbosityLevel,
}

impl Param {
    pub fn new(ref_block_choice   : RefBlockChoice,
               report_blank       : bool,
               json_printer       : &Arc<JSONPrinter>,
               in_file            : &str,
               verbose            : bool,
               pr_verbosity_level : PRVerbosityLevel) -> Param {
        Param {
            ref_block_choice,
            report_blank,
            json_printer : Arc::clone(json_printer),
            in_file  : String::from(in_file),
            verbose,
            pr_verbosity_level,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    version                        : Version,
    pub meta_or_par_blocks_decoded : u64,
    pub data_or_par_blocks_decoded : u64,
    pub blocks_decode_failed       : u64,
    total_blocks                   : u64,
    start_time                     : f64,
    end_time                       : f64,
    json_printer                   : Arc<JSONPrinter>,
}

impl Stats {
    pub fn new(ref_block    : &Block,
               file_size    : u64,
               json_printer : &Arc<JSONPrinter>) -> Stats {
        use file_utils::from_container_size::calc_total_block_count;
        let total_blocks =
            calc_total_block_count(ref_block.get_version(),
                                   file_size);
        Stats {
            version                    : ref_block.get_version(),
            blocks_decode_failed       : 0,
            meta_or_par_blocks_decoded : 0,
            data_or_par_blocks_decoded : 0,
            total_blocks,
            start_time                 : 0.,
            end_time                   : 0.,
            json_printer               : Arc::clone(json_printer),
        }
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      {
        (self.meta_or_par_blocks_decoded
         + self.data_or_par_blocks_decoded
         + self.blocks_decode_failed) as u64
    }

    fn total_units(&self)        -> u64      { self.total_blocks as u64 }
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let block_size             = ver_to_block_size(self.version);
        let time_elapsed           = (self.end_time - self.start_time) as i64;
        let (hour, minute, second) = time_utils::seconds_to_hms(time_elapsed);

        let json_printer = &self.json_printer;

        json_printer.write_open_bracket(f, Some("stats"), BracketType::Curly)?;

        write_maybe_json!(f, json_printer, "SBX version                              : {}", ver_to_usize(self.version))?;
        write_maybe_json!(f, json_printer, "Block size used in checking              : {}", block_size                         => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks processed               : {}", self.units_so_far()                => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks passed check (metadata) : {}", self.meta_or_par_blocks_decoded    => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks passed check (data)     : {}", self.data_or_par_blocks_decoded    => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks failed check            : {}", self.blocks_decode_failed          => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Time elapsed                             : {:02}:{:02}:{:02}", hour, minute, second)?;

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

pub fn check_file(param : &Param)
                  -> Result<Option<Stats>, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let json_printer = &param.json_printer;

    let (_, ref_block) = get_ref_block!(param, json_printer, ctrlc_stop_flag);

    let file_size = file_utils::get_file_size(&param.in_file)?;
    let stats = Arc::new(Mutex::new(Stats::new(&ref_block,
                                               file_size,
                                               &param.json_printer)));

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    let mut block = Block::dummy();

    let reporter = Arc::new(ProgressReporter::new(&stats,
                                                  "SBX block checking progress",
                                                  "bytes",
                                                  param.pr_verbosity_level,
                                                  param.json_printer.json_enabled()));

    let ver_usize = ver_to_usize(ref_block.get_version());

    let block_size = ver_to_block_size(ref_block.get_version());

    let mut block_pos       : u64;
    let mut bytes_processed : u64 = 0;

    reporter.start();

    if param.verbose {
        json_printer.print_open_bracket(Some("blocks failed"), BracketType::Square);
    }

    loop {
        break_if_atomic_bool!(ctrlc_stop_flag);

        let read_res = reader.read(sbx_block::slice_buf_mut(ref_block.get_version(),
                                                            &mut buffer))?;

        block_pos        = bytes_processed;
        bytes_processed += read_res.len_read as u64;

        break_if_eof_seen!(read_res);

        match block.sync_from_buffer(&buffer, None) {
            Ok(_)  => match block.block_type() {
                BlockType::Meta => {
                    stats.lock().unwrap().meta_or_par_blocks_decoded += 1;
                },
                BlockType::Data => {
                    stats.lock().unwrap().data_or_par_blocks_decoded += 1;
                }
            },
            Err(_) => {
                // only report error if the buffer is not completely blank
                // unless report blank is true
                if param.report_blank
                    || !misc_utils::buffer_is_blank(
                        sbx_block::slice_buf(ref_block.get_version(),
                                             &buffer))
                {
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

                    stats.lock().unwrap().blocks_decode_failed += 1;
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

    let stats = stats.lock().unwrap().clone();

    Ok(Some(stats))
}
