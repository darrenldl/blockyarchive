use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use super::misc_utils;
use std::io::SeekFrom;

use super::progress_report::*;

use super::file_reader::FileReader;
use super::file_reader::FileReaderParam;
use super::file_writer::FileWriter;
use super::file_writer::FileWriterParam;

use super::sbx_specs::SBX_SCAN_BLOCK_SIZE;

use file_utils::from_orig_file_size::calc_rs_enabled_total_block_count;

use super::multihash;
use super::multihash::*;

use super::Error;
use super::sbx_specs::Version;

use super::sbx_block::{Block, BlockType};
use super::sbx_block;
use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_specs::{ver_to_block_size,
                       ver_to_data_size,
                       ver_uses_rs,
                       SBX_LAST_SEQ_NUM,
                       SBX_FIRST_DATA_SEQ_NUM,
                       ver_to_usize};

use super::cli_utils::report_ref_block_info;

use std::str::from_utf8;

use super::time_utils;
use super::block_utils;

use rs_codec::RSRepairer;
use rs_codec::RSCodecState;
use rs_codec::RSRepairStats;

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    version                              : Version,
    pub meta_blocks_decoded              : u64,
    pub data_or_par_blocks_decoded       : u64,
    pub blocks_decode_failed             : u64,
    pub meta_blocks_repaired             : u64,
    pub data_or_par_blocks_repaired      : u64,
    pub data_or_par_blocks_repair_failed : u64,
    total_blocks                         : u64,
    start_time                           : f64,
    end_time                             : f64,
}

impl Stats {
    pub fn new(ref_block    : &Block,
               total_blocks : u64) -> Stats {
        Stats {
            version                          : ref_block.get_version(),
            blocks_decode_failed             : 0,
            meta_blocks_decoded              : 0,
            data_or_par_blocks_decoded       : 0,
            meta_blocks_repaired             : 0,
            data_or_par_blocks_repaired      : 0,
            data_or_par_blocks_repair_failed : 0,
            total_blocks,
            start_time                       : 0.,
            end_time                         : 0.,
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

        writeln!(f, "SBX version                              : {}", ver_to_usize(self.version))?;
        writeln!(f, "Block size used in checking              : {}", block_size)?;
        writeln!(f, "Number of blocks processed               : {}", self.units_so_far())?;
        writeln!(f, "Number of blocks processed (metadata)    : {}", self.meta_blocks_decoded)?;
        writeln!(f, "Number of blocks processed (data)        : {}", self.data_or_par_blocks_decoded)?;
        writeln!(f, "Number of blocks failed to process       : {}", self.blocks_decode_failed)?;
        writeln!(f, "Number of blocks repaired (metadata)     : {}", self.meta_blocks_repaired)?;
        writeln!(f, "Number of blocks repaired (data)         : {}", self.data_or_par_blocks_repaired)?;
        writeln!(f, "Number of blocks failed to repair (data) : {}", self.data_or_par_blocks_repair_failed)?;
        writeln!(f, "Time elapsed                             : {:02}:{:02}:{:02}", hour, minute, second)?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    in_file            : String,
    verbose            : bool,
    pr_verbosity_level : PRVerbosityLevel,
    burst              : Option<usize>,
}

impl Param {
    pub fn new(in_file            : &str,
               verbose            : bool,
               pr_verbosity_level : PRVerbosityLevel,
               burst              : Option<usize>) -> Param {
        Param {
            in_file : String::from(in_file),
            verbose,
            pr_verbosity_level,
            burst,
        }
    }
}

pub fn repair_file(param : &Param)
                   -> Result<Option<Stats>, Error> {
    let (ref_block_pos, mut ref_block) = get_ref_block!(&param.in_file,
                                                        false,
                                                        param.verbose,
                                                        param.pr_verbosity_level);

    let version = ref_block.get_version();

    return_if_not_ver_uses_rs!(version);

    let block_size = ver_to_block_size(version);

    let data_shards   = get_RSD_from_ref_block!(ref_block_pos, ref_block, "repair");
    let parity_shards = get_RSP_from_ref_block!(ref_block_pos, ref_block, "repair");

    let total_block_count =
        match ref_block.get_FSZ().unwrap() {
            Some(x) =>
                calc_rs_enabled_total_block_count(version,
                                                  data_shards,
                                                  parity_shards,
                                                  x),
            None    => {
                print_block!(
                    "";
                    "Warning : No recorded file size found, using container file size to estimate total";
                    "          number of blocks. This may overestimate total number of blocks, and may";
                    "          show false repair/verify failures when gaps in container are encountered.";
                    "");
                let metadata = file_utils::get_file_metadata(&param.in_file)?;
                metadata.len() / block_size as u64
            },
        };

    let stats = Arc::new(Mutex::new(Stats::new(&ref_block, total_block_count)));

    let burst =
        match param.burst {
            None => {
                match block_utils::guess_burst_err_resistance_level(&param.in_file,
                                                                    ref_block_pos,
                                                                    &ref_block)?
                {
                    None    => { return Err(Error::with_message("Failed to guess burst resistance level, please specify via --burst option")); },
                    Some(x) => x
                }
            },
            Some(x) => x
        };

    if param.verbose {
        println!("Using burst error resistance level {} for output container",
                 burst);
        println!();
    }

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : true,
                                                       buffered : true  })?;

    let mut block = Block::dummy();

    let reporter = Arc::new(ProgressReporter::new(&stats,
                                                  "SBX block repairing progress",
                                                  "blocks",
                                                  param.pr_verbosity_level));

    let pred = {
        let version = ref_block.get_version();
        let uid     = ref_block.get_uid();
        move |block : &Block| -> bool {
            block.get_version() == version
                && block.get_uid() == uid
        }
    };

    let mut rs_codec = RSRepairer::new(version,
                                       &ref_block,
                                       data_shards,
                                       parity_shards);

    reporter.start();

    // replace metadata blocks with reference block if broken
    {
        let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] =
            [0; SBX_LARGEST_BLOCK_SIZE];

        ref_block.sync_to_buffer(None, &mut buffer).unwrap();

        for p in sbx_block::calc_rs_enabled_meta_all_write_pos_s(version,
                                                                 parity_shards,
                                                                 burst).iter()
        {
            reader.seek(SeekFrom::Start(*p))?;
            reader.read(sbx_block::slice_buf_mut(version, &mut buffer))?;
            match block.sync_from_buffer(&buffer, Some(&pred)) {
                Ok(()) => {
                    stats.lock().unwrap().meta_blocks_decoded += 1;
                },
                Err(_) => {
                    stats.lock().unwrap().blocks_decode_failed += 1;

                    reader.seek(SeekFrom::Start(*p))?;

                    ref_block.sync_to_buffer(None, &mut buffer).unwrap();
                    reader.write(sbx_block::slice_buf(version, &buffer))?;

                    stats.lock().unwrap().meta_blocks_repaired += 1;
                }
            }
        }
    }

    // repair data blocks
    for seq_num in 1..SBX_LAST_SEQ_NUM {
        if stats.lock().unwrap().units_so_far() >= total_block_count { break; }

        let pos = sbx_block::calc_rs_enabled_data_write_pos(seq_num,
                                                            version,
                                                            data_shards,
                                                            parity_shards,
                                                            burst);

        reader.seek(SeekFrom::Start(pos))?;

        let read_res = reader.read(rs_codec.get_block_buffer())?;

        let codec_state =
            if read_res.len_read < block_size {   // read an incomplete block
                stats.lock().unwrap().blocks_decode_failed += 1;
                rs_codec.mark_missing()
            } else if let Err(_) = block.sync_from_buffer(rs_codec.get_block_buffer(),
                                                          Some(&pred)) {
                stats.lock().unwrap().blocks_decode_failed += 1;
                rs_codec.mark_missing()
            } else {
                if block.get_seq_num() != seq_num {
                    stats.lock().unwrap().blocks_decode_failed += 1;
                    rs_codec.mark_missing()
                } else {
                    if block.is_meta() {
                        stats.lock().unwrap().meta_blocks_decoded += 1;
                    } else {
                        stats.lock().unwrap().data_or_par_blocks_decoded += 1;
                    }

                    rs_codec.mark_present()
                }
            };

        match codec_state {
            RSCodecState::Ready => {
                let (repair_stats, repaired_blocks) =
                    rs_codec.repair_with_block_sync(seq_num, burst);

                if repair_stats.successful {
                    stats.lock().unwrap().data_or_par_blocks_repaired +=
                        repair_stats.missing_count as u64;
                } else {
                    stats.lock().unwrap().data_or_par_blocks_repair_failed +=
                        repair_stats.missing_count as u64;
                }

                if param.verbose {
                    reporter.pause();
                    println!("{}", repair_stats);
                    reporter.resume();
                }

                // write the repaired data blocks
                for &(pos, block_buf) in repaired_blocks.iter() {
                    reader.seek(SeekFrom::Start(pos))?;
                    reader.write(&block_buf)?;
                }

                // update stats
                stats.lock().unwrap().blocks_decode_failed += repair_stats.missing_count as u64;
            },
            RSCodecState::NotReady => {},
        }
    }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();

    Ok(Some(stats))
}
