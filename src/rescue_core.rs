use std::fmt;
use std::io::SeekFrom;
use std::sync::mpsc::channel;
use std::sync::mpsc::sync_channel;
use std::sync::Barrier;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::file_utils;

use crate::misc_utils;
use crate::misc_utils::RequiredLenAndSeekTo;

use crate::log::*;
use crate::progress_report::*;

use crate::cli_utils::setup_ctrlc_handler;

use crate::file_reader::{FileReader, FileReaderParam};

use crate::general_error::Error;

use crate::sbx_specs::{SBX_FILE_UID_LEN, SBX_SCAN_BLOCK_SIZE};

use crate::sbx_block::BlockType;

use crate::block_utils;

use crate::integer_utils::IntegerUtils;

use crate::misc_utils::{PositionOrLength, RangeEnd};

use crate::json_printer::{BracketType, JSONPrinter};

use crate::rescue_buffer::{RescueBuffer, Slot};

const PIPELINE_BUFFER_IN_ROTATION: usize = 9;

const BLOCK_COUNT_IN_BUFFER: usize = 1000;

pub struct Param {
    in_file: String,
    out_dir: String,
    log_file: Option<String>,
    from_pos: Option<u64>,
    to_pos: Option<RangeEnd<u64>>,
    force_misalign: bool,
    json_printer: Arc<JSONPrinter>,
    only_pick_block: Option<BlockType>,
    only_pick_uid: Option<[u8; SBX_FILE_UID_LEN]>,
    pr_verbosity_level: PRVerbosityLevel,
}

impl Param {
    pub fn new(
        in_file: &str,
        out_dir: &str,
        log_file: Option<&str>,
        from_pos: Option<u64>,
        to_pos: Option<RangeEnd<u64>>,
        force_misalign: bool,
        json_printer: &Arc<JSONPrinter>,
        only_pick_block: Option<BlockType>,
        only_pick_uid: Option<&[u8; SBX_FILE_UID_LEN]>,
        pr_verbosity_level: PRVerbosityLevel,
    ) -> Param {
        Param {
            in_file: String::from(in_file),
            out_dir: String::from(out_dir),
            log_file: match log_file {
                None => None,
                Some(x) => Some(String::from(x)),
            },
            from_pos,
            to_pos,
            force_misalign,
            json_printer: Arc::clone(json_printer),
            only_pick_block,
            only_pick_uid: match only_pick_uid {
                None => None,
                Some(x) => Some(x.clone()),
            },
            pr_verbosity_level,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    pub meta_blocks_processed: u64,
    pub data_or_par_blocks_processed: u64,
    pub bytes_processed: u64,
    total_bytes: u64,
    start_time: f64,
    end_time: f64,
    json_printer: Arc<JSONPrinter>,
}

impl Stats {
    pub fn new(required_len: u64, json_printer: &Arc<JSONPrinter>) -> Result<Stats, Error> {
        let stats = Stats {
            meta_blocks_processed: 0,
            data_or_par_blocks_processed: 0,
            bytes_processed: 0,
            total_bytes: required_len,
            start_time: 0.,
            end_time: 0.,
            json_printer: Arc::clone(json_printer),
        };
        Ok(stats)
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
        self.bytes_processed
    }

    fn total_units(&self) -> Option<u64> {
        Some(self.total_bytes)
    }
}

mod parsers {
    use nom::character::complete::digit1;
    use nom::character::complete::newline;
    use std::num::ParseIntError;

    type StatsParseResult = Result<(u64, u64, u64, u64), ParseIntError>;

    pub fn parse_digits(bytes: &[u8], blocks: &[u8], meta: &[u8], data: &[u8]) -> StatsParseResult {
        use std::str::from_utf8;

        let bytes = from_utf8(bytes).unwrap();
        let blocks = from_utf8(blocks).unwrap();
        let meta = from_utf8(meta).unwrap();
        let data = from_utf8(data).unwrap();

        Ok((
            bytes.parse::<u64>()?,
            blocks.parse::<u64>()?,
            meta.parse::<u64>()?,
            data.parse::<u64>()?,
        ))
    }

    named!(pub stats_p <StatsParseResult>,
           do_parse!(
               _id : tag!(b"bytes_processed=") >>
                   bytes  : digit1 >> _n : newline >>
                   _id : tag!(b"blocks_processed=") >>
                   blocks : digit1 >> _n : newline >>
                   _id : tag!(b"meta_blocks_processed=") >>
                   meta   : digit1 >> _n : newline >>
                   _id : tag!(b"data_blocks_processed=") >>
                   data   : digit1 >> _n : newline >>
                   (parse_digits(bytes, blocks, meta, data))
           )
    );
}

impl Log for Stats {
    fn serialize(&self) -> String {
        let mut string = String::with_capacity(200);
        string.push_str(&format!("bytes_processed={}\n", self.bytes_processed));
        string.push_str(&format!(
            "blocks_processed={}\n",
            self.meta_blocks_processed + self.data_or_par_blocks_processed
        ));
        string.push_str(&format!(
            "meta_blocks_processed={}\n",
            self.meta_blocks_processed
        ));
        string.push_str(&format!(
            "data_blocks_processed={}\n",
            self.data_or_par_blocks_processed
        ));

        string
    }

    fn deserialize(&mut self, input: &[u8]) -> Result<(), ()> {
        match parsers::stats_p(input) {
            Ok((_, Ok((bytes, _, meta, data)))) => {
                self.bytes_processed = u64::round_down_to_multiple(
                    u64::ensure_at_most(self.total_bytes, bytes),
                    SBX_SCAN_BLOCK_SIZE as u64,
                );
                self.meta_blocks_processed = meta;
                self.data_or_par_blocks_processed = data;
                Ok(())
            }
            _ => Err(()),
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let json_printer = &self.json_printer;

        json_printer.write_open_bracket(f, Some("stats"), BracketType::Curly)?;

        write_maybe_json!(
            f,
            json_printer,
            "Number of bytes processed             : {}",
            self.bytes_processed
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks processed            : {}",
            self.meta_blocks_processed + self.data_or_par_blocks_processed
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks processed (metadata) : {}",
            self.meta_blocks_processed
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks processed (data)     : {}",
            self.data_or_par_blocks_processed
        )?;

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
struct SendToWriter {
    bytes_processed: u64,
    meta_blocks_processed: u64,
    data_or_par_blocks_processed: u64,
}

pub fn rescue_from_file(param: &Param) -> Result<Stats, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let file_size = file_utils::get_file_size(&param.in_file)?;

    // calulate length to read
    let RequiredLenAndSeekTo { required_len, .. } =
        misc_utils::calc_required_len_and_seek_to_from_byte_range(
            param.from_pos,
            param.to_pos,
            param.force_misalign,
            // 0 is fine here as `bytes_so_far` doesn't affect calculation
            // of the required length
            0,
            PositionOrLength::Len(file_size),
            None,
        );

    let stats = Arc::new(Mutex::new(Stats::new(required_len, &param.json_printer)?));

    let mut reader = FileReader::new(
        &param.in_file,
        FileReaderParam {
            write: false,
            buffered: true,
        },
    )?;

    let log_handler = Arc::new(match param.log_file {
        None => LogHandler::new(None, &stats),
        Some(ref f) => LogHandler::new(Some(f), &stats),
    });
    let reporter = Arc::new(ProgressReporter::new(
        &stats,
        "Data rescue progress",
        "bytes",
        param.pr_verbosity_level,
        param.json_printer.json_enabled(),
    ));

    // read from log file and update stats if the log file exists
    log_handler.read_from_file()?;

    // now calculate the position to seek to with the final bytes processed count
    let RequiredLenAndSeekTo { seek_to, .. } =
        misc_utils::calc_required_len_and_seek_to_from_byte_range(
            param.from_pos,
            param.to_pos,
            param.force_misalign,
            stats.lock().unwrap().bytes_processed,
            PositionOrLength::Len(file_size),
            None,
        );

    // seek to calculated position
    reader.seek(SeekFrom::Start(seek_to))?;

    let (to_grouper, from_reader) = sync_channel(PIPELINE_BUFFER_IN_ROTATION + 1);
    let (to_writer, from_grouper) = sync_channel(PIPELINE_BUFFER_IN_ROTATION + 1);
    let (to_reader, from_writer) = sync_channel(PIPELINE_BUFFER_IN_ROTATION + 1);
    let (error_tx_reader, error_rx) = channel::<Error>();
    let error_tx_writer = error_tx_reader.clone();

    let worker_shutdown_barrier = Arc::new(Barrier::new(3));

    // push buffers into pipeline
    for _ in 0..PIPELINE_BUFFER_IN_ROTATION {
        to_reader
            .send(Some(RescueBuffer::new(BLOCK_COUNT_IN_BUFFER)))
            .unwrap();
    }

    log_handler.start();
    reporter.start();

    let reader_thread = {
        let ctrlc_stop_flag = Arc::clone(&ctrlc_stop_flag);
        let shutdown_barrier = Arc::clone(&worker_shutdown_barrier);
        let only_pick_block = param.only_pick_block;
        let only_pick_uid = param.only_pick_uid;
        let stats = stats.lock().unwrap();
        let mut bytes_processed = stats.bytes_processed;
        let mut meta_blocks_processed = stats.meta_blocks_processed;
        let mut data_or_par_blocks_processed = stats.data_or_par_blocks_processed;

        thread::spawn(move || {
            let mut run = true;

            while let Some(mut buffer) = from_writer.recv().unwrap() {
                if !run {
                    break;
                }

                while !buffer.is_full() {
                    stop_run_if_atomic_bool!(run => ctrlc_stop_flag);

                    stop_run_if_reached_required_len!(run => bytes_processed, required_len);

                    let Slot { block, slot } = buffer.get_slot().unwrap();

                    let lazy_read_res =
                        match block_utils::read_block_lazily(block, slot, &mut reader) {
                            Ok(lazy_read_res) => lazy_read_res,
                            Err(e) => stop_run_forward_error!(run => error_tx_reader => e),
                        };

                    bytes_processed += lazy_read_res.len_read as u64;

                    if lazy_read_res.usable {
                        // update stats
                        match block.block_type() {
                            BlockType::Meta => {
                                meta_blocks_processed += 1;
                            }
                            BlockType::Data => {
                                data_or_par_blocks_processed += 1;
                            }
                        }

                        let mut cancel_slot = false;

                        // check if block matches required block type
                        if let Some(x) = only_pick_block {
                            if block.block_type() != x {
                                cancel_slot = true;
                            }
                        }

                        // check if block has the required UID
                        if let Some(x) = only_pick_uid {
                            if block.get_uid() != x {
                                cancel_slot = true;
                            }
                        }

                        if cancel_slot {
                            buffer.cancel_slot();
                        }
                    } else {
                        buffer.cancel_slot();
                    }
                }

                let send_to_writer = SendToWriter {
                    bytes_processed,
                    meta_blocks_processed,
                    data_or_par_blocks_processed,
                };

                to_grouper.send(Some((send_to_writer, buffer))).unwrap();
            }

            worker_shutdown!(to_grouper, shutdown_barrier);
        })
    };

    let grouper_thread = {
        let shutdown_barrier = Arc::clone(&worker_shutdown_barrier);

        thread::spawn(move || {
            while let Some((send_to_writer, mut buffer)) = from_reader.recv().unwrap() {
                buffer.group_by_uid();

                to_writer.send(Some((send_to_writer, buffer))).unwrap();
            }

            worker_shutdown!(to_writer, shutdown_barrier);
        })
    };

    let writer_thread = {
        let shutdown_barrier = Arc::clone(&worker_shutdown_barrier);
        let stats = Arc::clone(&stats);
        let out_dir = param.out_dir.clone();

        thread::spawn(move || {
            while let Some((send_to_writer, mut buffer)) = from_grouper.recv().unwrap() {
                if let Err(e) = buffer.write(&out_dir) {
                    error_tx_writer.send(e).unwrap();
                    break;
                }

                buffer.reset();

                {
                    let SendToWriter {
                        bytes_processed,
                        meta_blocks_processed,
                        data_or_par_blocks_processed,
                    } = send_to_writer;

                    let mut stats = stats.lock().unwrap();

                    stats.bytes_processed = bytes_processed;
                    stats.meta_blocks_processed = meta_blocks_processed;
                    stats.data_or_par_blocks_processed = data_or_par_blocks_processed;
                }

                to_reader.send(Some(buffer)).unwrap();
            }

            worker_shutdown!(to_reader, shutdown_barrier);
        })
    };

    reader_thread.join().unwrap();
    grouper_thread.join().unwrap();
    writer_thread.join().unwrap();

    if let Ok(err) = error_rx.try_recv() {
        return Err(err);
    }

    // // now calculate the position to seek to with the final bytes processed count
    // let RequiredLenAndSeekTo { seek_to, .. } =
    //     misc_utils::calc_required_len_and_seek_to_from_byte_range(
    //         param.from_pos,
    //         param.to_pos,
    //         param.force_misalign,
    //         stats.lock().unwrap().bytes_processed,
    //         PositionOrLength::Len(file_size),
    //         None,
    //     );

    // // seek to calculated position
    // reader.seek(SeekFrom::Start(seek_to))?;

    // loop {
    //     let mut stats = stats.lock().unwrap();

    //     break_if_atomic_bool!(ctrlc_stop_flag);

    //     break_if_reached_required_len!(stats.bytes_processed, required_len);

    //     let lazy_read_res = block_utils::read_block_lazily(&mut block, &mut buffer, &mut reader)?;

    //     stats.bytes_processed += lazy_read_res.len_read as u64;

    //     break_if_eof_seen!(lazy_read_res);

    //     if !lazy_read_res.usable {
    //         continue;
    //     }

    //     // update stats
    //     match block.block_type() {
    //         BlockType::Meta => {
    //             stats.meta_blocks_processed += 1;
    //         }
    //         BlockType::Data => {
    //             stats.data_or_par_blocks_processed += 1;
    //         }
    //     }

    //     // check if block matches required block type
    //     if let Some(x) = param.only_pick_block {
    //         if block.block_type() != x {
    //             continue;
    //         }
    //     }

    //     // check if block has the required UID
    //     if let Some(x) = param.only_pick_uid {
    //         if block.get_uid() != x {
    //             continue;
    //         }
    //     }

    //     // write block out
    //     let uid_str = misc_utils::bytes_to_upper_hex_string(&block.get_uid());
    //     let path = misc_utils::make_path(&[&param.out_dir, &uid_str]);
    //     let mut writer = FileWriter::new(
    //         &path,
    //         FileWriterParam {
    //             read: false,
    //             append: true,
    //             truncate: false,
    //             buffered: false,
    //         },
    //     )?;

    //     // use the original bytes which are still in the buffer
    //     writer.write(sbx_block::slice_buf(block.get_version(), &buffer))?;

    //     // check if there's any error in log handling
    //     log_handler.pop_error()?;
    // }

    reporter.stop();
    log_handler.stop();

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
