use crate::file_utils;
use crate::misc_utils;
use std::cmp::Ordering;
use std::fmt;
use std::io::SeekFrom;
use std::sync::{Arc, Mutex};

use crate::misc_utils::RequiredLenAndSeekTo;

use crate::progress_report::*;

use crate::json_printer::{BracketType, JSONPrinter};

use crate::file_reader::{FileReader, FileReaderParam};
use crate::file_writer::{FileWriter, FileWriterParam};

use crate::general_error::Error;
use crate::sbx_specs::Version;

use crate::sbx_block;
use crate::sbx_block::Block;
use crate::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use crate::sbx_specs::{ver_to_block_size, ver_to_usize, ver_uses_rs};

use crate::cli_utils::report_ref_block_info;
use crate::cli_utils::setup_ctrlc_handler;

use crate::block_utils;
use crate::time_utils;

use crate::misc_utils::MultiPassType;

use crate::block_utils::RefBlockChoice;

use crate::misc_utils::{PositionOrLength, RangeEnd};

pub struct Param {
    ref_block_choice: RefBlockChoice,
    ref_block_from_pos: Option<u64>,
    ref_block_to_pos: Option<RangeEnd<u64>>,
    multi_pass: Option<MultiPassType>,
    json_printer: Arc<JSONPrinter>,
    from_pos: Option<u64>,
    to_pos: Option<RangeEnd<u64>>,
    force_misalign: bool,
    in_file: String,
    out_file: Option<String>,
    verbose: bool,
    pr_verbosity_level: PRVerbosityLevel,
    burst: Option<usize>,
}

impl Param {
    pub fn new(
        ref_block_choice: RefBlockChoice,
        ref_block_from_pos: Option<u64>,
        ref_block_to_pos: Option<RangeEnd<u64>>,
        multi_pass: Option<MultiPassType>,
        json_printer: &Arc<JSONPrinter>,
        from_pos: Option<u64>,
        to_pos: Option<RangeEnd<u64>>,
        force_misalign: bool,
        in_file: &str,
        out_file: Option<&str>,
        verbose: bool,
        pr_verbosity_level: PRVerbosityLevel,
        burst: Option<usize>,
    ) -> Param {
        Param {
            ref_block_choice,
            ref_block_from_pos,
            ref_block_to_pos,
            multi_pass,
            json_printer: Arc::clone(json_printer),
            from_pos,
            to_pos,
            force_misalign,
            in_file: String::from(in_file),
            out_file: match out_file {
                Some(x) => Some(String::from(x)),
                None => None,
            },
            verbose,
            pr_verbosity_level,
            burst,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    version: Version,
    pub meta_blocks_decoded: u64,
    pub data_blocks_decoded: u64,
    pub parity_blocks_decoded: u64,
    pub blocks_decode_failed: u64,
    pub meta_blocks_same_order: u64,
    pub meta_blocks_diff_order: u64,
    pub data_blocks_same_order: u64,
    pub data_blocks_diff_order: u64,
    pub parity_blocks_same_order: u64,
    pub parity_blocks_diff_order: u64,
    total_blocks: u64,
    start_time: f64,
    end_time: f64,
    json_printer: Arc<JSONPrinter>,
}

impl Stats {
    pub fn new(ref_block: &Block, file_size: u64, json_printer: &Arc<JSONPrinter>) -> Stats {
        use crate::file_utils::from_container_size::calc_total_block_count;
        let total_blocks = calc_total_block_count(ref_block.get_version(), file_size);
        Stats {
            version: ref_block.get_version(),
            blocks_decode_failed: 0,
            meta_blocks_decoded: 0,
            data_blocks_decoded: 0,
            parity_blocks_decoded: 0,
            total_blocks,
            meta_blocks_same_order: 0,
            meta_blocks_diff_order: 0,
            data_blocks_same_order: 0,
            data_blocks_diff_order: 0,
            parity_blocks_same_order: 0,
            parity_blocks_diff_order: 0,
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
        (self.meta_blocks_decoded
            + self.data_blocks_decoded
            + self.parity_blocks_decoded
            + self.blocks_decode_failed) as u64
    }

    fn total_units(&self) -> u64 {
        self.total_blocks as u64
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
            "SBX version                               : {}",
            ver_to_usize(self.version)
        )?;
        write_maybe_json!(f, json_printer, "Block size used in checking               : {}", block_size                      => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks processed                : {}", self.units_so_far()             => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks sorted (metadata)        : {}", self.meta_blocks_decoded        => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks sorted (data)            : {}", self.data_blocks_decoded        => skip_quotes)?;
        if ver_uses_rs(self.version) {
            write_maybe_json!(f, json_printer, "Number of blocks sorted (parity)          : {}", self.parity_blocks_decoded        => skip_quotes)?;
        }
        write_maybe_json!(f, json_printer, "Number of blocks in same order (metadata) : {}", self.meta_blocks_same_order     => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks in diff order (metadata) : {}", self.meta_blocks_diff_order     => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks in same order (data)     : {}", self.data_blocks_same_order     => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks in diff order (data)     : {}", self.data_blocks_diff_order     => skip_quotes)?;
        if ver_uses_rs(self.version) {
            write_maybe_json!(f, json_printer, "Number of blocks in same order (parity)   : {}", self.parity_blocks_same_order     => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks in diff order (parity)   : {}", self.parity_blocks_diff_order     => skip_quotes)?;
        }
        write_maybe_json!(f, json_printer, "Number of blocks failed to sort           : {}", self.blocks_decode_failed       => skip_quotes)?;
        write_maybe_json!(
            f,
            json_printer,
            "Time elapsed                              : {:02}:{:02}:{:02}",
            hour,
            minute,
            second
        )?;

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

pub fn sort_file(param: &Param) -> Result<Option<Stats>, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let json_printer = &param.json_printer;

    let (ref_block_pos, ref_block) = get_ref_block!(param, json_printer, ctrlc_stop_flag);

    let file_size = file_utils::get_file_size(&param.in_file)?;
    let stats = Arc::new(Mutex::new(Stats::new(&ref_block, file_size, json_printer)));

    let version = ref_block.get_version();
    let rs_enabled = ver_uses_rs(version);

    let burst = get_burst_or_guess!(param, ref_block_pos, ref_block);

    let data_par_burst = if rs_enabled {
        Some((
            get_RSD_from_ref_block!(ref_block_pos, ref_block, "sort"),
            get_RSP_from_ref_block!(ref_block_pos, ref_block, "sort"),
            burst,
        ))
    } else {
        None
    };

    let mut buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut reader = FileReader::new(
        &param.in_file,
        FileReaderParam {
            write: false,
            buffered: true,
        },
    )?;

    let mut writer = match param.out_file {
        Some(ref f) => Some(FileWriter::new(
            f,
            FileWriterParam {
                read: param.multi_pass == Some(MultiPassType::SkipGood),
                append: false,
                truncate: param.multi_pass == None,
                buffered: true,
            },
        )?),
        None => None,
    };

    let mut block = Block::dummy();

    let reporter = Arc::new(ProgressReporter::new(
        &stats,
        "SBX block sorting progress",
        "blocks",
        param.pr_verbosity_level,
        param.json_printer.json_enabled(),
    ));

    let mut meta_written = false;

    let pred = block_pred_same_ver_uid!(ref_block);

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
        Some(ver_to_block_size(version) as u64),
    );

    // seek to calculated position
    reader.seek(SeekFrom::Start(seek_to))?;

    let read_offset = seek_to % ver_to_block_size(version) as u64;

    reporter.start();

    let mut bytes_processed: u64 = 0;

    let mut check_block = Block::dummy();

    let mut check_buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    loop {
        let mut stats = stats.lock().unwrap();

        break_if_atomic_bool!(ctrlc_stop_flag);

        break_if_reached_required_len!(bytes_processed, required_len);

        let read_pos = reader.cur_pos()?;

        let read_res = reader.read(sbx_block::slice_buf_mut(version, &mut buffer))?;

        bytes_processed += read_res.len_read as u64;

        break_if_eof_seen!(read_res);

        if let Err(_) = block.sync_from_buffer(&buffer, Some(&pred)) {
            stats.blocks_decode_failed += 1;
            continue;
        }

        if block.is_meta() {
            if !meta_written {
                let write_pos_s =
                    sbx_block::calc_meta_block_all_write_pos_s(version, data_par_burst);

                // copy the value of current position in original container
                let reader_cur_pos = reader.cur_pos()?;

                for &p in write_pos_s.iter() {
                    let do_write = match param.multi_pass {
                        None | Some(MultiPassType::OverwriteAll) => true,
                        Some(MultiPassType::SkipGood) => {
                            if let Some(ref mut writer) = writer {
                                // read metadata blocks
                                writer.seek(SeekFrom::Start(p))?;
                                let read_res = writer
                                    .read(sbx_block::slice_buf_mut(version, &mut check_buffer))?;

                                read_res.eof_seen
                                    ||
                                {
                                    // if block at output position is a valid metadata block,
                                    // then don't overwrite
                                    match check_block.sync_from_buffer(&check_buffer, Some(&pred)) {
                                        Ok(()) => check_block.get_seq_num() != 0,
                                        Err(_) => true,
                                    }
                                }
                            } else {
                                // doesn't really matter what to put here, but let's pick default to true
                                true
                            }
                        }
                    };

                    if do_write {
                        if let Some(ref mut writer) = writer {
                            // write metadata blocks
                            writer.seek(SeekFrom::Start(p))?;
                            writer.write(sbx_block::slice_buf(version, &buffer))?;
                        }
                    }

                    // read block in original container
                    reader.seek(SeekFrom::Start(p + seek_to))?;
                    let read_res = reader.read(sbx_block::slice_buf_mut(version, &mut check_buffer))?;

                    let same_order = !read_res.eof_seen &&
                        match buffer.cmp(&check_buffer) {
                            Ordering::Equal => true,
                            _ => false,
                        };

                    if same_order {
                        stats.meta_blocks_same_order += 1
                    } else {
                        stats.meta_blocks_diff_order += 1
                    }
                }

                // restore the position of reader
                reader.seek(SeekFrom::Start(reader_cur_pos))?;

                meta_written = true;
            }
        } else {
            let write_pos = sbx_block::calc_data_block_write_pos(
                version,
                block.get_seq_num(),
                None,
                data_par_burst,
            );

            let do_write = match param.multi_pass {
                None | Some(MultiPassType::OverwriteAll) => true,
                Some(MultiPassType::SkipGood) => {
                    if let Some(ref mut writer) = writer {
                        // read block in output container
                        writer.seek(SeekFrom::Start(write_pos))?;
                        let read_res =
                            writer.read(sbx_block::slice_buf_mut(version, &mut check_buffer))?;

                        read_res.eof_seen
                        || {
                            // if block at output position is a valid block and has same seq number,
                            // then don't overwrite
                            match check_block.sync_from_buffer(&check_buffer, Some(&pred)) {
                                Ok(()) => check_block.get_seq_num() != block.get_seq_num(),
                                Err(_) => true,
                            }
                        }
                    } else {
                        // doesn't really matter what to put here, but let's pick default to true
                        true
                    }
                }
            };

            if do_write {
                if let Some(ref mut writer) = writer {
                    writer.seek(SeekFrom::Start(write_pos))?;
                    writer.write(sbx_block::slice_buf(version, &buffer))?;
                }
            }

            if read_pos - read_offset == write_pos {
                if block.is_parity_w_data_par_burst(data_par_burst) {
                    stats.parity_blocks_same_order += 1;
                } else {
                    stats.data_blocks_same_order += 1;
                }
            } else {
                if block.is_parity_w_data_par_burst(data_par_burst) {
                    stats.parity_blocks_diff_order += 1;
                } else {
                    stats.data_blocks_diff_order += 1;
                }
            }
        }

        if block.is_meta() {
            stats.meta_blocks_decoded += 1;
        } else if block.is_parity_w_data_par_burst(data_par_burst) {
            stats.parity_blocks_decoded += 1;
        } else {
            stats.data_blocks_decoded += 1;
        }
    }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();

    Ok(Some(stats))
}
