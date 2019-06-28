use crate::file_reader::{FileReader, FileReaderParam};
use crate::general_error::Error;
use crate::hash_stats::HashStats;
use crate::json_printer::JSONPrinter;
use crate::multihash::*;
use crate::progress_report::{PRVerbosityLevel, ProgressReporter};
use crate::sbx_block;
use crate::sbx_block::Block;
use crate::sbx_specs::{ver_to_data_size, SBX_LARGEST_BLOCK_SIZE};
use crate::data_block_buffer::DataBlockBuffer;

use std::io::SeekFrom;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::sync_channel;

const PIPELINE_BUFFER_IN_ROTATION: usize = 9;

pub fn hash(
    json_printer: &JSONPrinter,
    pr_verbosity_level: PRVerbosityLevel,
    data_par_burst: Option<(usize, usize, usize)>,
    ctrlc_stop_flag: &AtomicBool,
    in_file: &str,
    orig_file_size: u64,
    ref_block: &Block,
    mut hash_ctx: hash::Ctx,
) -> Result<(HashStats, HashBytes), Error> {
    let stats = Arc::new(Mutex::new(HashStats::new(orig_file_size)));

    let version = ref_block.get_version();

    let data_chunk_size = ver_to_data_size(version) as u64;

    let mut buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut reader = FileReader::new(
        &in_file,
        FileReaderParam {
            write: false,
            buffered: true,
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

    // push buffers into pipeline
    for i in 0..PIPELINE_BUFFER_IN_ROTATION {
        to_reader
            .send(Some(DataBlockBuffer::new(
                version,
                &,
                param.data_par_burst,
                param.meta_enabled,
                i,
                PIPELINE_BUFFER_IN_ROTATION,
            )))
            .unwrap();
    }

    reporter.start();

    let reader_thread = {
        let mut run = true;

        while let Some(mut buffer) = from_hasher.recv().unwrap() {
        }
    };

    // go through data and parity blocks
    let mut seq_num = 1;
    loop {
        let mut stats = stats.lock().unwrap();

        break_if_atomic_bool!(ctrlc_stop_flag);

        let pos = sbx_block::calc_data_block_write_pos(version, seq_num, None, data_par_burst);

        reader.seek(SeekFrom::Start(pos))?;

        // read at reference block block size
        let read_res = reader.read(sbx_block::slice_buf_mut(version, &mut buffer))?;

        let decode_successful = !read_res.eof_seen
            && match block.sync_from_buffer(&buffer, Some(&header_pred), None) {
                Ok(_) => block.get_seq_num() == seq_num,
                _ => false,
            };

        let bytes_remaining = stats.total_bytes - stats.bytes_processed;

        let is_last_data_block = bytes_remaining <= data_chunk_size;

        if !sbx_block::seq_num_is_meta(seq_num)
            && !sbx_block::seq_num_is_parity_w_data_par_burst(seq_num, data_par_burst)
        {
            if decode_successful {
                let slice = if is_last_data_block {
                    &sbx_block::slice_data_buf(version, &buffer)[0..bytes_remaining as usize]
                } else {
                    sbx_block::slice_data_buf(version, &buffer)
                };

                // hash data chunk
                hash_ctx.update(slice);

                stats.bytes_processed += slice.len() as u64;
            } else {
                return Err(Error::with_msg("Failed to decode data block"));
            }
        }

        if is_last_data_block {
            break;
        }

        incre_or_break_if_last!(seq_num => seq_num);
    }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();

    Ok((stats, hash_ctx.finish_into_hash_bytes()))
}
