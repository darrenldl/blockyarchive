use std::sync::{Arc, Mutex};
use std::fmt;
use file_utils;
use std::io::SeekFrom;
use std::cmp::Ordering;

use progress_report::*;

use json_printer::{JSONPrinter,
                   BracketType};

use file_reader::{FileReader,
                  FileReaderParam};
use file_writer::{FileWriter,
                  FileWriterParam};

use general_error::Error;
use sbx_specs::Version;

use sbx_block::Block;
use sbx_block;
use sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use sbx_specs::{ver_to_block_size,
                ver_uses_rs,
                ver_to_usize};

use cli_utils::report_ref_block_info;
use cli_utils::setup_ctrlc_handler;

use time_utils;
use block_utils;

use block_utils::RefBlockChoice;

pub struct Param {
    ref_block_choice   : RefBlockChoice,
    multi_pass         : bool,
    json_printer       : Arc<JSONPrinter>,
    in_file            : String,
    out_file           : String,
    verbose            : bool,
    pr_verbosity_level : PRVerbosityLevel,
    burst              : Option<usize>,
}

impl Param {
    pub fn new(ref_block_choice   : RefBlockChoice,
               multi_pass         : bool,
               json_printer       : &Arc<JSONPrinter>,
               in_file            : &str,
               out_file           : &str,
               verbose            : bool,
               pr_verbosity_level : PRVerbosityLevel,
               burst              : Option<usize>) -> Param {
        Param {
            ref_block_choice,
            multi_pass,
            json_printer       : Arc::clone(json_printer),
            in_file            : String::from(in_file),
            out_file           : String::from(out_file),
            verbose,
            pr_verbosity_level,
            burst,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    version                        : Version,
    pub meta_blocks_decoded        : u64,
    pub data_or_par_blocks_decoded : u64,
    pub blocks_decode_failed       : u64,
    pub meta_blocks_same_order     : u64,
    pub meta_blocks_diff_order     : u64,
    pub data_blocks_same_order     : u64,
    pub data_blocks_diff_order     : u64,
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
            meta_blocks_decoded        : 0,
            data_or_par_blocks_decoded : 0,
            total_blocks,
            meta_blocks_same_order     : 0,
            meta_blocks_diff_order     : 0,
            data_blocks_same_order     : 0,
            data_blocks_diff_order     : 0,
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
        (self.meta_blocks_decoded
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

        write_maybe_json!(f, json_printer, "SBX version                               : {}", ver_to_usize(self.version))?;
        write_maybe_json!(f, json_printer, "Block size used in checking               : {}", block_size                      => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks processed                : {}", self.units_so_far()             => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks sorted (metadata)        : {}", self.meta_blocks_decoded        => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks sorted (data)            : {}", self.data_or_par_blocks_decoded => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks in same order (metadata) : {}", self.meta_blocks_same_order     => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks in diff order (metadata) : {}", self.meta_blocks_diff_order     => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks in same order (data)     : {}", self.data_blocks_same_order     => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks in diff order (data)     : {}", self.data_blocks_diff_order     => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks sorted (data)            : {}", self.data_or_par_blocks_decoded => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks failed to sort           : {}", self.blocks_decode_failed       => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Time elapsed                              : {:02}:{:02}:{:02}", hour, minute, second)?;

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

pub fn sort_file(param : &Param)
                 -> Result<Option<Stats>, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let json_printer = &param.json_printer;

    let (ref_block_pos, ref_block) = get_ref_block!(param,
                                                    json_printer,
                                                    ctrlc_stop_flag);

    let file_size = file_utils::get_file_size(&param.in_file)?;
    let stats = Arc::new(Mutex::new(Stats::new(&ref_block, file_size, json_printer)));

    let version   = ref_block.get_version();
    let rs_enabled = ver_uses_rs(version);

    let burst = get_burst_or_guess!(param,
                                    ref_block_pos,
                                    ref_block);

    let data_par_burst =
        if rs_enabled {
            Some((get_RSD_from_ref_block!(ref_block_pos, ref_block, "sort"),
                  get_RSP_from_ref_block!(ref_block_pos, ref_block, "sort"),
                  burst))
        } else {
            None
        };

    let mut buffer       : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];
    let mut check_buffer : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    let mut writer = FileWriter::new(&param.out_file,
                                     FileWriterParam { read     : false,
                                                       append   : false,
                                                       truncate : !param.multi_pass,
                                                       buffered : true   })?;

    let mut block = Block::dummy();

    let reporter = Arc::new(ProgressReporter::new(&stats,
                                                  "SBX block sorting progress",
                                                  "blocks",
                                                  param.pr_verbosity_level,
                                                  param.json_printer.json_enabled()));

    let mut meta_written = false;

    let pred = block_pred_same_ver_uid!(ref_block);

    reporter.start();

    loop {
        break_if_atomic_bool!(ctrlc_stop_flag);

        let read_res = reader.read(sbx_block::slice_buf_mut(version,
                                                            &mut buffer))?;

        break_if_eof_seen!(read_res);

        if let Err(_) = block.sync_from_buffer(&buffer, Some(&pred)) {
            stats.lock().unwrap().blocks_decode_failed += 1;
            continue;
        }

        if block.is_meta() {
            if !meta_written {
                let write_pos_s =
                    sbx_block::calc_meta_block_all_write_pos_s(version,
                                                               data_par_burst);

                // copy the value of current position in original container
                let reader_cur_pos = reader.cur_pos()?;

                for &p in write_pos_s.iter() {
                    // write metadata blocks
                    writer.seek(SeekFrom::Start(p))?;
                    writer.write(sbx_block::slice_buf(version,
                                                      &buffer))?;

                    // read block in original container
                    reader.seek(SeekFrom::Start(p))?;
                    reader.read(sbx_block::slice_buf_mut(version,
                                                         &mut check_buffer))?;

                    match buffer.cmp(&check_buffer) {
                        Ordering::Equal => stats.lock().unwrap().meta_blocks_same_order += 1,
                        _               => stats.lock().unwrap().meta_blocks_diff_order += 1,
                    }
                }

                // restore the position of reader
                reader.seek(SeekFrom::Start(reader_cur_pos))?;

                meta_written = true;
            }
        } else {
            let write_pos =
                sbx_block::calc_data_block_write_pos(version,
                                                     block.get_seq_num(),
                                                     None,
                                                     data_par_burst);

            // copy the value of current position in original container
            let reader_cur_pos = reader.cur_pos()?;

            writer.seek(SeekFrom::Start(write_pos))?;
            writer.write(sbx_block::slice_buf(version,
                                              &buffer))?;

            // read block in original container
            reader.seek(SeekFrom::Start(write_pos))?;
            reader.read(sbx_block::slice_buf_mut(version,
                                                 &mut check_buffer))?;

            match buffer.cmp(&check_buffer) {
                Ordering::Equal => stats.lock().unwrap().data_blocks_same_order += 1,
                _               => stats.lock().unwrap().data_blocks_diff_order += 1,
            }

            // restore the position of reader
            reader.seek(SeekFrom::Start(reader_cur_pos))?;
        }

        if block.is_meta() {
            stats.lock().unwrap().meta_blocks_decoded += 1;
        } else {
            stats.lock().unwrap().data_or_par_blocks_decoded += 1;
        }
    }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();

    Ok(Some(stats))
}
