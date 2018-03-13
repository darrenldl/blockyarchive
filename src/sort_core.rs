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
                       ver_to_usize};

use super::cli_utils::report_ref_block_info;
use super::cli_utils::setup_ctrlc_handler;

use std::str::from_utf8;

use super::time_utils;
use super::block_utils;

pub struct Param {
    no_meta            : bool,
    in_file            : String,
    out_file           : String,
    verbose            : bool,
    pr_verbosity_level : PRVerbosityLevel,
    burst              : Option<usize>,
}

impl Param {
    pub fn new(no_meta            : bool,
               in_file            : &str,
               out_file           : &str,
               verbose            : bool,
               pr_verbosity_level : PRVerbosityLevel,
               burst              : Option<usize>) -> Param {
        Param {
            no_meta,
            in_file  : String::from(in_file),
            out_file : String::from(out_file),
            verbose,
            pr_verbosity_level,
            burst,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    version                        : Version,
    pub meta_blocks_decoded        : u64,
    pub data_or_par_blocks_decoded : u64,
    pub blocks_decode_failed       : u64,
    total_blocks                   : u64,
    start_time                     : f64,
    end_time                       : f64,
}

impl Stats {
    pub fn new(ref_block     : &Block,
               file_metadata : &fs::Metadata) -> Stats {
        use file_utils::from_container_metadata::calc_total_block_count;
        let total_blocks =
            calc_total_block_count(ref_block.get_version(),
                                   file_metadata);
        Stats {
            version                 : ref_block.get_version(),
            blocks_decode_failed    : 0,
            meta_blocks_decoded        : 0,
            data_or_par_blocks_decoded : 0,
            total_blocks,
            start_time              : 0.,
            end_time                : 0.,
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

        writeln!(f, "SBX version                        : {}", ver_to_usize(self.version))?;
        writeln!(f, "Block size used in checking        : {}", block_size)?;
        writeln!(f, "Number of blocks processed         : {}", self.units_so_far())?;
        writeln!(f, "Number of blocks sorted (metadata) : {}", self.meta_blocks_decoded)?;
        writeln!(f, "Number of blocks sorted (data)     : {}", self.data_or_par_blocks_decoded)?;
        writeln!(f, "Number of blocks failed to sort    : {}", self.blocks_decode_failed)?;
        writeln!(f, "Time elapsed                       : {:02}:{:02}:{:02}", hour, minute, second)?;

        Ok(())
    }
}

pub fn sort_file(param : &Param)
                 -> Result<Option<Stats>, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler();

    let (ref_block_pos, ref_block) = get_ref_block!(param,
                                                    ctrlc_stop_flag);

    let metadata = file_utils::get_file_metadata(&param.in_file)?;
    let stats = Arc::new(Mutex::new(Stats::new(&ref_block, &metadata)));

    let version   = ref_block.get_version();
    let block_size = ver_to_block_size(version) as u64;

    let rs_enabled = ver_uses_rs(version);

    let burst = get_burst_or_guess!(param,
                                    ref_block_pos,
                                    ref_block);

    let mut data_shards = None;
    let mut parity_shards = None;

    if rs_enabled {
        data_shards   = Some(get_RSD_from_ref_block!(ref_block_pos, ref_block, "sort"));
        parity_shards = Some(get_RSP_from_ref_block!(ref_block_pos, ref_block, "sort"));
    }

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    let mut writer = FileWriter::new(&param.out_file,
                                     FileWriterParam { read     : false,
                                                       append   : false,
                                                       buffered : true   })?;

    let mut block = Block::dummy();

    let reporter = Arc::new(ProgressReporter::new(&stats,
                                                  "SBX block sorting progress",
                                                  "blocks",
                                                  param.pr_verbosity_level));

    let mut meta_written = false;

    let pred = {
        let version = ref_block.get_version();
        let uid     = ref_block.get_uid();
        move |block : &Block| -> bool {
            block.get_version() == version
                && block.get_uid() == uid
        }
    };

    reporter.start();

    loop {
        break_if_atomic_bool!(ctrlc_stop_flag);

        let read_res = reader.read(sbx_block::slice_buf_mut(ref_block.get_version(),
                                                            &mut buffer))?;

        break_if_eof_seen!(read_res);

        if let Err(_) = block.sync_from_buffer(&buffer, Some(&pred)) {
            stats.lock().unwrap().blocks_decode_failed += 1;
            continue;
        }

        if block.is_meta() {
            if !meta_written {
                writer.seek(SeekFrom::Start(0))?;
                writer.write(sbx_block::slice_buf(version,
                                                  &buffer))?;

                if rs_enabled {
                    let write_pos_s =
                        sbx_block::calc_rs_enabled_meta_dup_write_pos_s(version,
                                                                        parity_shards.unwrap(),
                                                                        burst);
                    for p in write_pos_s.iter() {
                        writer.seek(SeekFrom::Start(*p))?;
                        writer.write(sbx_block::slice_buf(version,
                                                          &buffer))?;
                    }
                }

                meta_written = true;
            }
        } else {
            let write_pos =
                if rs_enabled {
                    sbx_block::calc_rs_enabled_data_write_pos(block.get_seq_num(),
                                                              version,
                                                              data_shards.unwrap(),
                                                              parity_shards.unwrap(),
                                                              burst)
                } else {
                    block.get_seq_num() as u64 * block_size
                };

            writer.seek(SeekFrom::Start(write_pos))?;
            writer.write(sbx_block::slice_buf(version,
                                              &buffer))?;
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
