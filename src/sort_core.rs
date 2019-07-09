use crate::file_utils;
use crate::misc_utils;
use std::fmt;
use std::io::SeekFrom;
use std::sync::mpsc::channel;
use std::sync::mpsc::sync_channel;
use std::sync::Barrier;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::misc_utils::RequiredLenAndSeekTo;

use crate::progress_report::*;

use crate::json_printer::{BracketType, JSONPrinter};

use crate::file_reader::{FileReader, FileReaderParam};
use crate::file_writer::{FileWriter, FileWriterParam};
use crate::writer::{Writer, WriterType};

use crate::data_block_buffer::{
    BlockArrangement, DataBlockBuffer, InputType, OutputType, Slot, SlotView,
};

use crate::general_error::Error;
use crate::sbx_specs::Version;

use crate::sbx_block;
use crate::sbx_block::Block;
use crate::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use crate::sbx_specs::{ver_to_block_size, ver_to_usize, ver_uses_rs};

use crate::cli_utils::setup_ctrlc_handler;

use crate::time_utils;

use crate::misc_utils::MultiPassType;

use crate::block_utils::RefBlockChoice;

use crate::misc_utils::{PositionOrLength, RangeEnd};

const PIPELINE_BUFFER_IN_ROTATION: usize = 9;

enum SendToWriter {
    Meta(Vec<u8>),
    Data(DataBlockBuffer),
}

pub struct Param {
    ref_block_choice: RefBlockChoice,
    ref_block_from_pos: Option<u64>,
    ref_block_to_pos: Option<RangeEnd<u64>>,
    report_blank: bool,
    guess_burst_from_pos: Option<u64>,
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
        report_blank: bool,
        guess_burst_from_pos: Option<u64>,
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
            report_blank,
            guess_burst_from_pos,
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
    block_size: u64,
    pub meta_blocks_decoded: u64,
    pub data_blocks_decoded: u64,
    pub parity_blocks_decoded: u64,
    pub blocks_decode_failed: u64,
    pub okay_blank_blocks: u64,
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
        let version = ref_block.get_version();
        Stats {
            version,
            block_size: ver_to_block_size(version) as u64,
            meta_blocks_decoded: 0,
            data_blocks_decoded: 0,
            parity_blocks_decoded: 0,
            blocks_decode_failed: 0,
            okay_blank_blocks: 0,
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

    fn blocks_so_far(&self) -> u64 {
        self.meta_blocks_decoded
            + self.data_blocks_decoded
            + self.parity_blocks_decoded
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
            "SBX version                               : {}",
            ver_to_usize(self.version)
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Block size used in checking               : {}",
            block_size
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks processed                : {}",
            self.blocks_so_far()
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks sorted (metadata)        : {}",
            self.meta_blocks_decoded
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks sorted (data)            : {}",
            self.data_blocks_decoded
        )?;
        if ver_uses_rs(self.version) {
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks sorted (parity)          : {}",
                self.parity_blocks_decoded
            )?;
        }
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks in same order (metadata) : {}",
            self.meta_blocks_same_order
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks in diff order (metadata) : {}",
            self.meta_blocks_diff_order
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks in same order (data)     : {}",
            self.data_blocks_same_order
        )?;
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks in diff order (data)     : {}",
            self.data_blocks_diff_order
        )?;
        if ver_uses_rs(self.version) {
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks in same order (parity)   : {}",
                self.parity_blocks_same_order
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks in diff order (parity)   : {}",
                self.parity_blocks_diff_order
            )?;
        }
        write_maybe_json!(
            f,
            json_printer,
            "Number of blocks failed to sort           : {}",
            self.blocks_decode_failed
        )?;
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

fn check_metadata_blocks_reader(
    version: Version,
    data_par_burst: Option<(usize, usize, usize)>,
    metadata_block: &[u8],
    offset: u64,
    reader: &mut FileReader,
) -> Result<(u64, u64), Error> {
    let mut meta_blocks_same_order = 0;
    let mut meta_blocks_diff_order = 0;

    let initial_read_pos = reader.cur_pos()?;

    let mut check_buffer = [0u8; SBX_LARGEST_BLOCK_SIZE];

    let check_buffer = sbx_block::slice_buf_mut(
        version,
        &mut check_buffer,
    );

    // read blocks in original container
    // and check against current metadata block
    let write_pos_s =
        sbx_block::calc_meta_block_all_write_pos_s(
            version,
            data_par_burst,
        );

    for &p in write_pos_s.iter() {
        reader.seek(SeekFrom::Start(p + offset))?;

        let read_res = reader.read(check_buffer)?;

        let same_order =
            !read_res.eof_seen && check_buffer == metadata_block;

        if same_order {
            meta_blocks_same_order += 1;
        } else {
            meta_blocks_diff_order += 1;
        }
    }

    // reset read position
    reader.seek(SeekFrom::Start(initial_read_pos))?;

    Ok((meta_blocks_same_order, meta_blocks_diff_order))
}

pub fn sort_file(param: &Param) -> Result<Option<Stats>, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let json_printer = &param.json_printer;

    let (ref_block_pos, ref_block) = get_ref_block!(param, json_printer, ctrlc_stop_flag);

    let file_size = file_utils::get_file_size(&param.in_file)?;
    let stats = Arc::new(Mutex::new(Stats::new(&ref_block, file_size, json_printer)));

    let version = ref_block.get_version();

    let data_par_burst = get_data_par_burst!(param, ref_block_pos, ref_block, "sort");

    let mut reader = FileReader::new(
        &param.in_file,
        FileReaderParam {
            write: false,
            buffered: true,
        },
    )?;

    let mut writer = match param.out_file {
        Some(ref f) => Some(Writer::new(WriterType::File(FileWriter::new(
            f,
            FileWriterParam {
                read: param.multi_pass == Some(MultiPassType::SkipGood),
                append: false,
                truncate: param.multi_pass == None,
                buffered: true,
            },
        )?))),
        None => None,
    };

    let reporter = Arc::new(ProgressReporter::new(
        &stats,
        "SBX block sorting progress",
        "bytes",
        param.pr_verbosity_level,
        param.json_printer.json_enabled(),
    ));

    let header_pred = header_pred_same_ver_uid!(ref_block);

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

    let (to_writer, from_reader) = sync_channel(PIPELINE_BUFFER_IN_ROTATION + 2); // one extra space for the case of metadata block
    let (to_reader, from_writer) = sync_channel(PIPELINE_BUFFER_IN_ROTATION + 1);
    let (error_tx_reader, error_rx) = channel::<Error>();
    let error_tx_writer = error_tx_reader.clone();

    let worker_shutdown_barrier = Arc::new(Barrier::new(2));

    let skip_good = match param.multi_pass {
        None | Some(MultiPassType::OverwriteAll) => false,
        Some(MultiPassType::SkipGood) => true,
    };

    // push buffers into pipeline
    let buffers = DataBlockBuffer::new_multi(
        ref_block.get_version(),
        Some(&ref_block.get_uid()),
        InputType::Block,
        OutputType::Block,
        BlockArrangement::Unordered,
        data_par_burst,
        true,
        skip_good,
        PIPELINE_BUFFER_IN_ROTATION,
    );

    for buffer in buffers.into_iter() {
        to_reader.send(Some(buffer)).unwrap();
    }

    let reader_thread = {
        let version = ref_block.get_version();
        let shutdown_barrier = Arc::clone(&worker_shutdown_barrier);
        let report_blank = param.report_blank;
        let block_size = ver_to_block_size(version);
        let stats = Arc::clone(&stats);
        let mut bytes_processed: u64 = 0;

        thread::spawn(move || {
            let mut run = true;
            let mut meta_written = false;

            while let Some(mut buffer) = from_writer.recv().unwrap() {
                if !run {
                    break;
                }

                let mut meta_blocks_same_order = 0;
                let mut meta_blocks_diff_order = 0;
                let mut meta_blocks_decoded = 0;
                let mut parity_blocks_decoded = 0;
                let mut data_blocks_decoded = 0;
                let mut blocks_decode_failed = 0;
                let mut okay_blank_blocks = 0;

                while !buffer.is_full() {
                    stop_run_if_atomic_bool!(run => ctrlc_stop_flag);

                    stop_run_if_reached_required_len!(run => bytes_processed, required_len);

                    let read_pos = match reader.cur_pos() {
                        Ok(read_pos) => read_pos,
                        Err(e) => {
                            stop_run_forward_error!(run => error_tx_reader => e);
                        }
                    };

                    let Slot {
                        block,
                        slot,
                        read_pos: slot_read_pos,
                        content_len_exc_header: _,
                    } = buffer.get_slot().unwrap();
                    match reader.read(slot) {
                        Ok(read_res) => {
                            bytes_processed += read_res.len_read as u64;

                            if read_res.eof_seen {
                                buffer.cancel_slot();
                                run = false;
                                break;
                            }

                            match block.sync_from_buffer(slot, Some(&header_pred), None) {
                                Ok(()) => {
                                    if block.is_meta() {
                                        if !meta_written {
                                            let (same_order, diff_order) =
                                                match check_metadata_blocks_reader(
                                                    version,
                                                    data_par_burst,
                                                    slot,
                                                    seek_to,
                                                    &mut reader
                                                ) {
                                                    Ok(x) => x,
                                                    Err(e) => {
                                                        stop_run_forward_error!(run => error_tx_reader => e);
                                                    }
                                                };

                                            meta_blocks_same_order = same_order;
                                            meta_blocks_diff_order = diff_order;

                                            // copy current metadata block to send to writer
                                            let mut meta_buffer = vec![0u8; block_size];
                                            meta_buffer.clone_from_slice(slot);

                                            to_writer
                                                .send(Some(SendToWriter::Meta(meta_buffer)))
                                                .unwrap();

                                            meta_written = true;
                                        }

                                        buffer.cancel_slot();

                                        meta_blocks_decoded += 1;
                                    } else {
                                        eprintln!("\ndata seq num : {}", block.get_seq_num());
                                        if block.is_parity_w_data_par_burst(data_par_burst) {
                                            parity_blocks_decoded += 1;
                                        } else {
                                            data_blocks_decoded += 1;
                                        }

                                        *slot_read_pos = Some(read_pos);
                                    }
                                }
                                Err(_) => {
                                    // only consider it failed if the buffer is not completely blank
                                    // unless report blank is true
                                    if misc_utils::buffer_is_blank(sbx_block::slice_buf(
                                        version, slot,
                                    )) {
                                        if report_blank {
                                            blocks_decode_failed += 1;
                                        } else {
                                            okay_blank_blocks += 1;
                                        }
                                    } else {
                                        blocks_decode_failed += 1;
                                    }

                                    buffer.cancel_slot();
                                }
                            }
                        }
                        Err(e) => {
                            buffer.cancel_slot();
                            stop_run_forward_error!(run => error_tx_reader => e);
                        }
                    }
                }

                {
                    let mut stats = stats.lock().unwrap();

                    stats.meta_blocks_decoded += meta_blocks_decoded;
                    stats.meta_blocks_same_order += meta_blocks_same_order;
                    stats.meta_blocks_diff_order += meta_blocks_diff_order;
                    stats.parity_blocks_decoded += parity_blocks_decoded;
                    stats.data_blocks_decoded += data_blocks_decoded;
                    stats.blocks_decode_failed += blocks_decode_failed;
                    stats.okay_blank_blocks += okay_blank_blocks;
                }

                to_writer.send(Some(SendToWriter::Data(buffer))).unwrap();
            }

            worker_shutdown!(to_writer, shutdown_barrier);
        })
    };

    let writer_thread = {
        let shutdown_barrier = Arc::clone(&worker_shutdown_barrier);
        let multi_pass = param.multi_pass;
        let stats = Arc::clone(&stats);

        thread::spawn(move || {
            let mut run = true;

            while let Some(data) = from_reader.recv().unwrap() {
                if !run {
                    break;
                }

                match data {
                    SendToWriter::Meta(meta_buffer) => {
                        let mut check_buffer = [0; SBX_LARGEST_BLOCK_SIZE];

                        let check_buffer = sbx_block::slice_buf_mut(version, &mut check_buffer);

                        let mut check_block = Block::dummy();

                        let write_pos_s =
                            sbx_block::calc_meta_block_all_write_pos_s(version, data_par_burst);

                        for &p in write_pos_s.iter() {
                            let do_write = match multi_pass {
                                None | Some(MultiPassType::OverwriteAll) => true,
                                Some(MultiPassType::SkipGood) => {
                                    if let Some(ref mut writer) = writer {
                                        // read blocks in the output container
                                        if let Err(e) = writer.seek(SeekFrom::Start(p)).unwrap() {
                                            stop_run_forward_error!(run => error_tx_writer => e);
                                        }

                                        let read_res = match writer.read(check_buffer).unwrap() {
                                            Ok(read_res) => read_res,
                                            Err(e) => {
                                                stop_run_forward_error!(run => error_tx_writer => e)
                                            }
                                        };

                                        read_res.eof_seen || {
                                            // if block at output position is a valid metadata block,
                                            // then don't overwrite
                                            match check_block.sync_from_buffer(
                                                &check_buffer,
                                                Some(&header_pred),
                                                None,
                                            ) {
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
                                    if let Err(e) = writer.seek(SeekFrom::Start(p)).unwrap() {
                                        stop_run_forward_error!(run => error_tx_writer => e);
                                    }
                                    if let Err(e) = writer.write(&meta_buffer) {
                                        stop_run_forward_error!(run => error_tx_writer => e);
                                    }
                                }
                            }
                        }
                    }
                    SendToWriter::Data(mut buffer) => {
                        buffer.calc_slot_write_pos();

                        if let Some(ref mut writer) = writer {
                            if let Err(e) = buffer.write(writer) {
                                error_tx_writer.send(e).unwrap();
                                break;
                            }
                        }

                        let mut data_blocks_same_order = 0;
                        let mut data_blocks_diff_order = 0;
                        let mut parity_blocks_same_order = 0;
                        let mut parity_blocks_diff_order = 0;

                        for SlotView {
                            block,
                            slot: _,
                            read_pos,
                            write_pos,
                            content_len_exc_header: _,
                        } in buffer.view_slots()
                        {
                            if read_pos.unwrap() - read_offset == write_pos.unwrap() {
                                if block.is_parity_w_data_par_burst(data_par_burst) {
                                    parity_blocks_same_order += 1;
                                } else {
                                    data_blocks_same_order += 1;
                                }
                            } else {
                                if block.is_parity_w_data_par_burst(data_par_burst) {
                                    parity_blocks_diff_order += 1;
                                } else {
                                    data_blocks_diff_order += 1;
                                }
                            }
                        }

                        {
                            let mut stats = stats.lock().unwrap();

                            stats.data_blocks_same_order += data_blocks_same_order;
                            stats.data_blocks_diff_order += data_blocks_diff_order;
                            stats.parity_blocks_same_order += parity_blocks_same_order;
                            stats.parity_blocks_diff_order += parity_blocks_diff_order;
                        }

                        buffer.reset();

                        to_reader.send(Some(buffer)).unwrap();
                    }
                }
            }

            worker_shutdown!(to_reader, shutdown_barrier);
        })
    };

    reader_thread.join().unwrap();
    writer_thread.join().unwrap();

    if let Ok(err) = error_rx.try_recv() {
        return Err(err);
    }

    // loop {
    //     let mut stats = stats.lock().unwrap();

    //     break_if_atomic_bool!(ctrlc_stop_flag);

    //     break_if_reached_required_len!(bytes_processed, required_len);

    //     let read_pos = reader.cur_pos()?;

    //     let read_res = reader.read(sbx_block::slice_buf_mut(version, &mut buffer))?;

    //     bytes_processed += read_res.len_read as u64;

    //     break_if_eof_seen!(read_res);

    //     if let Err(_) = block.sync_from_buffer(&buffer, Some(&header_pred), None) {
    //         // only consider it failed if the buffer is not completely blank
    //         // unless report blank is true
    //         if misc_utils::buffer_is_blank(sbx_block::slice_buf(ref_block.get_version(), &buffer)) {
    //             if param.report_blank {
    //                 stats.blocks_decode_failed += 1;
    //             } else {
    //                 stats.okay_blank_blocks += 1;
    //             }
    //         } else {
    //             stats.blocks_decode_failed += 1;
    //         }
    //         continue;
    //     }

    //     if block.is_meta() {
    //         if !meta_written {
    //             let write_pos_s =
    //                 sbx_block::calc_meta_block_all_write_pos_s(version, data_par_burst);

    //             // copy the value of current position in original container
    //             let reader_cur_pos = reader.cur_pos()?;

    //             for &p in write_pos_s.iter() {
    //                 let do_write = match param.multi_pass {
    //                     None | Some(MultiPassType::OverwriteAll) => true,
    //                     Some(MultiPassType::SkipGood) => {
    //                         if let Some(ref mut writer) = writer {
    //                             // read metadata blocks
    //                             writer.seek(SeekFrom::Start(p))?;
    //                             let read_res = writer
    //                                 .read(sbx_block::slice_buf_mut(version, &mut check_buffer))?;

    //                             read_res.eof_seen || {
    //                                 // if block at output position is a valid metadata block,
    //                                 // then don't overwrite
    //                                 match check_block.sync_from_buffer(
    //                                     &check_buffer,
    //                                     Some(&header_pred),
    //                                     None,
    //                                 ) {
    //                                     Ok(()) => check_block.get_seq_num() != 0,
    //                                     Err(_) => true,
    //                                 }
    //                             }
    //                         } else {
    //                             // doesn't really matter what to put here, but let's pick default to true
    //                             true
    //                         }
    //                     }
    //                 };

    //                 if do_write {
    //                     if let Some(ref mut writer) = writer {
    //                         // write metadata blocks
    //                         writer.seek(SeekFrom::Start(p))?;
    //                         writer.write(sbx_block::slice_buf(version, &buffer))?;
    //                     }
    //                 }

    //                 // read block in original container
    //                 reader.seek(SeekFrom::Start(p + seek_to))?;
    //                 let read_res =
    //                     reader.read(sbx_block::slice_buf_mut(version, &mut check_buffer))?;

    //                 let same_order = !read_res.eof_seen
    //                     && match buffer.cmp(&check_buffer) {
    //                         Ordering::Equal => true,
    //                         _ => false,
    //                     };

    //                 if same_order {
    //                     stats.meta_blocks_same_order += 1
    //                 } else {
    //                     stats.meta_blocks_diff_order += 1
    //                 }
    //             }

    //             // restore the position of reader
    //             reader.seek(SeekFrom::Start(reader_cur_pos))?;

    //             meta_written = true;
    //         }
    //     } else {
    //         let write_pos = sbx_block::calc_data_block_write_pos(
    //             version,
    //             block.get_seq_num(),
    //             None,
    //             data_par_burst,
    //         );

    //         let do_write = match param.multi_pass {
    //             None | Some(MultiPassType::OverwriteAll) => true,
    //             Some(MultiPassType::SkipGood) => {
    //                 if let Some(ref mut writer) = writer {
    //                     // read block in output container
    //                     writer.seek(SeekFrom::Start(write_pos))?;
    //                     let read_res =
    //                         writer.read(sbx_block::slice_buf_mut(version, &mut check_buffer))?;

    //                     read_res.eof_seen || {
    //                         // if block at output position is a valid block and has same seq number,
    //                         // then don't overwrite
    //                         match check_block.sync_from_buffer(
    //                             &check_buffer,
    //                             Some(&header_pred),
    //                             None,
    //                         ) {
    //                             Ok(()) => check_block.get_seq_num() != block.get_seq_num(),
    //                             Err(_) => true,
    //                         }
    //                     }
    //                 } else {
    //                     // doesn't really matter what to put here, but let's pick default to true
    //                     true
    //                 }
    //             }
    //         };

    //         if do_write {
    //             if let Some(ref mut writer) = writer {
    //                 writer.seek(SeekFrom::Start(write_pos))?;
    //                 writer.write(sbx_block::slice_buf(version, &buffer))?;
    //             }
    //         }

    //         if read_pos - read_offset == write_pos {
    //             if block.is_parity_w_data_par_burst(data_par_burst) {
    //                 stats.parity_blocks_same_order += 1;
    //             } else {
    //                 stats.data_blocks_same_order += 1;
    //             }
    //         } else {
    //             if block.is_parity_w_data_par_burst(data_par_burst) {
    //                 stats.parity_blocks_diff_order += 1;
    //             } else {
    //                 stats.data_blocks_diff_order += 1;
    //             }
    //         }
    //     }

    //     if block.is_meta() {
    //         stats.meta_blocks_decoded += 1;
    //     } else if block.is_parity_w_data_par_burst(data_par_burst) {
    //         stats.parity_blocks_decoded += 1;
    //     } else {
    //         stats.data_blocks_decoded += 1;
    //     }
    // }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();

    Ok(Some(stats))
}
