use crate::data_block_buffer::{DataBlockBuffer, InputType, OutputType, Slot, BlockArrangement};
use crate::file_reader::{FileReader, FileReaderParam};
use crate::general_error::Error;
use crate::hash_stats::HashStats;
use crate::json_printer::JSONPrinter;
use crate::multihash::*;
use crate::progress_report::{PRVerbosityLevel, ProgressReporter};
use crate::sbx_block;
use crate::sbx_block::Block;
use crate::sbx_specs::{ver_to_data_size, SBX_LAST_SEQ_NUM};

use std::io::SeekFrom;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::sync::mpsc::sync_channel;
use std::sync::Barrier;
use std::sync::{Arc, Mutex};
use std::thread;

const PIPELINE_BUFFER_IN_ROTATION: usize = 2;

pub fn hash(
    json_printer: &JSONPrinter,
    pr_verbosity_level: PRVerbosityLevel,
    data_par_burst: Option<(usize, usize, usize)>,
    ctrlc_stop_flag: &Arc<AtomicBool>,
    in_file: &str,
    orig_file_size: u64,
    ref_block: &Block,
    mut hash_ctx: hash::Ctx,
) -> Result<(HashStats, HashBytes), Error> {
    let stats = Arc::new(Mutex::new(HashStats::new(orig_file_size)));

    let version = ref_block.get_version();

    let data_chunk_size = ver_to_data_size(version) as u64;

    let mut reader = FileReader::new(
        &in_file,
        FileReaderParam {
            write: false,
            buffered: false,
        },
    )?;

    let mut block = Block::dummy();

    let reporter = Arc::new(ProgressReporter::new(
        &stats,
        "Stored data hashing progress",
        "bytes",
        pr_verbosity_level,
        json_printer.json_enabled(),
    ));

    let header_pred = header_pred_same_ver_uid!(ref_block);

    let (to_hasher, from_reader) = sync_channel(PIPELINE_BUFFER_IN_ROTATION);
    let (to_reader, from_hasher) = sync_channel(PIPELINE_BUFFER_IN_ROTATION);
    let (error_tx_reader, error_rx) = channel::<Error>();
    let (hash_bytes_tx, hash_bytes_rx) = channel();

    let worker_shutdown_barrier = Arc::new(Barrier::new(2));

    // push buffers into pipeline
    for i in 0..PIPELINE_BUFFER_IN_ROTATION {
        to_reader
            .send(Some(DataBlockBuffer::new(
                version,
                None,
                InputType::Block,
                OutputType::Disabled,
                BlockArrangement::Ordered,
                data_par_burst,
                true,
                i,
                PIPELINE_BUFFER_IN_ROTATION,
            )))
            .unwrap();
    }

    reporter.start();

    let reader_thread = {
        let ctrlc_stop_flag = Arc::clone(ctrlc_stop_flag);
        let shutdown_barrier = Arc::clone(&worker_shutdown_barrier);
        let stats = Arc::clone(&stats);

        thread::spawn(move || {
            let mut run = true;
            let mut seq_num = 1;

            while let Some(mut buffer) = from_hasher.recv().unwrap() {
                let mut stats = stats.lock().unwrap();

                if !run {
                    break;
                }

                while !buffer.is_full() {
                    break_if_atomic_bool!(ctrlc_stop_flag);

                    let pos = sbx_block::calc_data_block_write_pos(
                        version,
                        seq_num,
                        None,
                        data_par_burst,
                    );

                    if let Err(e) = reader.seek(SeekFrom::Start(pos)) {
                        error_tx_reader.send(e).unwrap();
                        run = false;
                        break;
                    }

                    let Slot {slot, content_len_exc_header} = buffer.get_slot().unwrap();
                    match reader.read(slot) {
                        Ok(read_res) => {
                            eprintln!("seq_num : {}", seq_num);
                            let decode_successful = !read_res.eof_seen
                                && match block.sync_from_buffer(slot, Some(&header_pred), None) {
                                    Ok(_) => block.get_seq_num() == seq_num,
                                    _ => false,
                                };

                            let bytes_remaining = stats.total_bytes - stats.bytes_processed;

                            eprintln!("bytes_remaining : {}", bytes_remaining);

                            if !sbx_block::seq_num_is_parity_w_data_par_burst(
                                    seq_num,
                                    data_par_burst,
                                )
                            {
                                let is_last_data_block = bytes_remaining <= data_chunk_size;

                                if decode_successful {
                                    let bytes_processed = if is_last_data_block {
                                        bytes_remaining
                                    } else {
                                        data_chunk_size
                                    };

                                    stats.bytes_processed += bytes_processed as u64;
                                } else {
                                    error_tx_reader
                                        .send(Error::with_msg("Failed to decode data block"))
                                        .unwrap();
                                    run = false;
                                    break;
                                }

                                if is_last_data_block {
                                    *content_len_exc_header = Some(bytes_remaining as usize);
                                    run = false;
                                    break;
                                }
                            }

                            if seq_num == SBX_LAST_SEQ_NUM {
                                run = false;
                                break;
                            }

                            seq_num += 1;
                        }
                        Err(e) => {
                            error_tx_reader.send(e).unwrap();
                            run = false;
                            break;
                        }
                    }
                }

                break_if_atomic_bool!(ctrlc_stop_flag);

                to_hasher.send(Some(buffer)).unwrap();
            }

            to_hasher.send(None).unwrap();

            shutdown_barrier.wait();
        })
    };

    let hasher_thread = {
        let shutdown_barrier = Arc::clone(&worker_shutdown_barrier);

        thread::spawn(move || {
            while let Some(mut buffer) = from_reader.recv().unwrap() {
                buffer.hash(&mut hash_ctx);

                buffer.reset();

                to_reader.send(Some(buffer)).unwrap();
            }

            to_reader.send(None).unwrap();

            hash_bytes_tx
                .send(hash_ctx.finish_into_hash_bytes())
                .unwrap();

            shutdown_barrier.wait();
        })
    };

    reader_thread.join().unwrap();
    hasher_thread.join().unwrap();

    if let Ok(err) = error_rx.try_recv() {
        return Err(err);
    }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();

    Ok((stats, hash_bytes_rx.recv().unwrap()))
}
