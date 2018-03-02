use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use std::io::SeekFrom;
use super::ctrlc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use super::misc_utils;
use super::misc_utils::RequiredLenAndSeekTo;

use super::progress_report::*;
use super::log::*;

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
use super::sbx_specs::SBX_FILE_UID_LEN;
use super::sbx_specs::{ver_to_block_size,
                       ver_to_data_size,
                       ver_first_data_seq_num,
                       ver_supports_rs};

use super::block_utils;

use nom::digit;
use std::num::ParseIntError;
use super::integer_utils::IntegerUtils;

pub struct Param {
    in_file         : String,
    out_dir         : String,
    log_file        : Option<String>,
    from_pos        : Option<u64>,
    to_pos          : Option<u64>,
    force_misalign  : bool,
    only_pick_block : Option<BlockType>,
    only_pick_uid   : Option<[u8; SBX_FILE_UID_LEN]>,
    silence_level   : SilenceLevel,
}

impl Param {
    pub fn new(in_file         : &str,
               out_dir         : &str,
               log_file        : Option<&str>,
               from_pos        : Option<u64>,
               to_pos          : Option<u64>,
               force_misalign  : bool,
               only_pick_block : Option<BlockType>,
               only_pick_uid   : Option<&[u8; SBX_FILE_UID_LEN]>,
               silence_level   : SilenceLevel) -> Param {
        Param {
            in_file  : String::from(in_file),
            out_dir  : String::from(out_dir),
            log_file : match log_file {
                None    => None,
                Some(x) => Some(String::from(x)),
            },
            from_pos,
            to_pos,
            force_misalign,
            only_pick_block,
            only_pick_uid : match only_pick_uid {
                None    => None,
                Some(x) => Some(x.clone())
            },
            silence_level,
        }
    }
}

#[derive(Clone)]
pub struct Stats {
    pub meta_or_par_blocks_processed : u64,
    pub data_or_par_blocks_processed : u64,
    pub bytes_processed              : u64,
    total_bytes                      : u64,
    start_time                       : f64,
    end_time                         : f64,
}

impl Stats {
    pub fn new(file_metadata : &fs::Metadata)
               -> Result<Stats, Error> {
        let stats = Stats {
            meta_or_par_blocks_processed : 0,
            data_or_par_blocks_processed : 0,
            bytes_processed              : 0,
            total_bytes                  : file_metadata.len(),
            start_time                   : 0.,
            end_time                     : 0.,
        };
        Ok(stats)
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      { self.bytes_processed }

    fn total_units(&self)        -> u64      { self.total_bytes }
}

mod parsers {
    use nom::IResult;
    use nom::digit;
    use nom::newline;
    use std::num::ParseIntError;

    type StatsParseResult = Result<(u64, u64, u64, u64), ParseIntError>;

    pub fn parse_digits(bytes  : &[u8],
                        blocks : &[u8],
                        meta   : &[u8],
                        data   : &[u8])
                        -> StatsParseResult {
        use std::str::from_utf8;

        let bytes  = from_utf8(bytes).unwrap();
        let blocks = from_utf8(blocks).unwrap();
        let meta   = from_utf8(meta).unwrap();
        let data   = from_utf8(data).unwrap();

        Ok((bytes.parse::<u64>()?,
            blocks.parse::<u64>()?,
            meta.parse::<u64>()?,
            data.parse::<u64>()?))
    }

    named!(pub stats_p <StatsParseResult>,
           do_parse!(
               _id : tag!(b"bytes_processed=") >>
                   bytes  : digit >> _n : newline >>
                   _id : tag!(b"blocks_processed=") >>
                   blocks : digit >> _n : newline >>
                   _id : tag!(b"meta_blocks_processed=") >>
                   meta   : digit >> _n : newline >>
                   _id : tag!(b"data_blocks_processed=") >>
                   data   : digit >> _n : newline >>
                   (parse_digits(bytes, blocks, meta, data))
           )
    );
}

impl Log for Stats {
    fn serialize(&self) -> String {
        let mut string = String::with_capacity(200);
        string.push_str(&format!("bytes_processed={}\n",
                                 self.bytes_processed));
        string.push_str(&format!("blocks_processed={}\n",
                                 self.meta_or_par_blocks_processed
                                 + self.data_or_par_blocks_processed));
        string.push_str(&format!("meta_blocks_processed={}\n",
                                 self.meta_or_par_blocks_processed));
        string.push_str(&format!("data_blocks_processed={}\n",
                                 self.data_or_par_blocks_processed));

        string
    }

    fn deserialize(&mut self, input : &[u8]) -> Result<(), ()> {
        use nom::IResult;
        use std::cmp::min;

        match parsers::stats_p(input) {
            IResult::Done(_, Ok((bytes, _, meta, data))) => {
                self.bytes_processed              =
                    u64::round_down_to_multiple(
                        u64::ensure_at_most(self.total_bytes, bytes),
                        SBX_SCAN_BLOCK_SIZE as u64);
                self.meta_or_par_blocks_processed = meta;
                self.data_or_par_blocks_processed = data;
                Ok(())
            },
            _                                             => Err(())
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Number of bytes processed             : {}", self.bytes_processed)?;
        writeln!(f, "Number of blocks processed            : {}",
                 self.meta_or_par_blocks_processed
                 + self.data_or_par_blocks_processed)?;
        writeln!(f, "Number of blocks processed (metadata) : {}", self.meta_or_par_blocks_processed)?;
        writeln!(f, "Number of blocks processed (data)     : {}", self.data_or_par_blocks_processed)
    }
}

pub fn rescue_from_file(param : &Param)
                        -> Result<Stats, Error> {
    let metadata = file_utils::get_file_metadata(&param.in_file)?;
    let stats = Arc::new(Mutex::new(Stats::new(&metadata)?));

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    let log_handler = Arc::new(match param.log_file {
        None        => LogHandler::new(None,    &stats),
        Some(ref f) => LogHandler::new(Some(f), &stats),
    });
    let reporter = Arc::new(ProgressReporter::new(&stats,
                                                  "Data rescue progress",
                                                  "bytes",
                                                  param.silence_level));

    let loop_stop_flag = Arc::new(AtomicBool::new(false));

    let mut block = Block::dummy();

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    reporter.start();

    // read from log file if it exists
    log_handler.read_from_file()?;

    { // setup Ctrl-C handler
        let loop_stop_flag = Arc::clone(&loop_stop_flag);

        ctrlc::set_handler(move || {
            println!("Interrupted");
            loop_stop_flag.store(true, Ordering::Relaxed);
        }).expect("Failed to set Ctrl-C handler"); }

    // calulate length to read and position to seek to
    let RequiredLenAndSeekTo { required_len, seek_to } =
        misc_utils::calc_required_len_and_seek_to_from_byte_range(param.from_pos,
                                                                  param.to_pos,
                                                                  param.force_misalign,
                                                                  stats.lock().unwrap().bytes_processed,
                                                                  metadata.len());

    // seek to calculated position
    reader.seek(SeekFrom::Start(seek_to))?;

    loop {
        if loop_stop_flag.load(Ordering::Relaxed) {
            break;
        }

        if stats.lock().unwrap().bytes_processed > required_len {
            break;
        }

        let lazy_read_res = block_utils::read_block_lazily(&mut block,
                                                           &mut buffer,
                                                           &mut reader)?;

        stats.lock().unwrap().bytes_processed += lazy_read_res.len_read as u64;

        if lazy_read_res.eof     { break; }

        if !lazy_read_res.usable { continue; }

        // update stats
        match block.block_type() {
            BlockType::Meta => {
                stats.lock().unwrap().meta_or_par_blocks_processed += 1;
            },
            BlockType::Data => {
                stats.lock().unwrap().data_or_par_blocks_processed += 1;
            }
        }

        // write block out
        let uid_str = misc_utils::bytes_to_upper_hex_string(&block.get_file_uid());
        let path    = misc_utils::make_path(&[&param.out_dir, &uid_str]);
        let mut writer = FileWriter::new(&path,
                                         FileWriterParam { read     : false,
                                                           append   : true,
                                                           buffered : false  })?;
        writer.write(sbx_block::slice_buf(block.get_version(), &buffer))?;

        // update log file
        log_handler.write_to_file(false)?;
    }

    reporter.stop();

    log_handler.write_to_file(true)?;

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
