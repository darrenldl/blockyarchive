use std::sync::{Arc, Mutex};
use std::fmt;
use std::io::SeekFrom;

use file_utils;

use misc_utils;
use misc_utils::RequiredLenAndSeekTo;

use progress_report::*;
use log::*;

use cli_utils::setup_ctrlc_handler;

use file_reader::{FileReader,
                  FileReaderParam};
use file_writer::{FileWriter,
                  FileWriterParam};

use general_error::Error;

use sbx_specs::{SBX_LARGEST_BLOCK_SIZE,
                SBX_SCAN_BLOCK_SIZE,
                SBX_FILE_UID_LEN};

use sbx_block;
use sbx_block::{Block, BlockType};

use block_utils;

use integer_utils::IntegerUtils;

use misc_utils::{RangeEnd,
                 PositionOrLength};

use json_printer::{JSONPrinter,
                   BracketType};

pub struct Param {
    in_file            : String,
    out_dir            : String,
    log_file           : Option<String>,
    from_pos           : Option<u64>,
    to_pos             : Option<RangeEnd<u64>>,
    force_misalign     : bool,
    json_printer       : Arc<JSONPrinter>,
    only_pick_block    : Option<BlockType>,
    only_pick_uid      : Option<[u8; SBX_FILE_UID_LEN]>,
    pr_verbosity_level : PRVerbosityLevel,
}

impl Param {
    pub fn new(in_file            : &str,
               out_dir            : &str,
               log_file           : Option<&str>,
               from_pos           : Option<u64>,
               to_pos             : Option<RangeEnd<u64>>,
               force_misalign     : bool,
               json_printer       : &Arc<JSONPrinter>,
               only_pick_block    : Option<BlockType>,
               only_pick_uid      : Option<&[u8; SBX_FILE_UID_LEN]>,
               pr_verbosity_level : PRVerbosityLevel) -> Param {
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
            json_printer : Arc::clone(json_printer),
            only_pick_block,
            only_pick_uid : match only_pick_uid {
                None    => None,
                Some(x) => Some(x.clone())
            },
            pr_verbosity_level,
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
    json_printer                     : Arc<JSONPrinter>,
}

impl Stats {
    pub fn new(required_len : u64,
               json_printer : &Arc<JSONPrinter>)
               -> Result<Stats, Error> {
        let stats = Stats {
            meta_or_par_blocks_processed : 0,
            data_or_par_blocks_processed : 0,
            bytes_processed              : 0,
            total_bytes                  : required_len,
            start_time                   : 0.,
            end_time                     : 0.,
            json_printer                 : Arc::clone(json_printer),
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
        match parsers::stats_p(input) {
            Ok((_, Ok((bytes, _, meta, data)))) => {
                self.bytes_processed =
                    u64::round_down_to_multiple(
                        u64::ensure_at_most(self.total_bytes, bytes),
                        SBX_SCAN_BLOCK_SIZE as u64);
                self.meta_or_par_blocks_processed = meta;
                self.data_or_par_blocks_processed = data;
                Ok(())
            },
            _                                   => Err(())
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let json_printer = &self.json_printer;

        json_printer.write_open_bracket(f, Some("stats"), BracketType::Curly)?;

        write_maybe_json!(f, json_printer, "Number of bytes processed             : {}",
                          self.bytes_processed                                          => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks processed            : {}",
                          self.meta_or_par_blocks_processed
                          + self.data_or_par_blocks_processed                           => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks processed (metadata) : {}",
                          self.meta_or_par_blocks_processed                             => skip_quotes)?;
        write_maybe_json!(f, json_printer, "Number of blocks processed (data)     : {}",
                          self.data_or_par_blocks_processed                             => skip_quotes)?;

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

pub fn rescue_from_file(param : &Param)
                        -> Result<Stats, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let file_size = file_utils::get_file_size(&param.in_file)?;

    // calulate length to read and position to seek to
    let RequiredLenAndSeekTo { required_len, .. } =
        misc_utils::calc_required_len_and_seek_to_from_byte_range(param.from_pos,
                                                                  param.to_pos,
                                                                  param.force_misalign,
                                                                  0,
                                                                  PositionOrLength::Len(file_size),
                                                                  None);

    let stats = Arc::new(Mutex::new(Stats::new(required_len,
                                               &param.json_printer)?));

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
                                                  param.pr_verbosity_level,
                                                  param.json_printer.json_enabled()));

    let mut block = Block::dummy();

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    // read from log file if it exists
    log_handler.read_from_file()?;

    log_handler.start();
    reporter.start();

    let RequiredLenAndSeekTo { seek_to, .. } =
        misc_utils::calc_required_len_and_seek_to_from_byte_range(param.from_pos,
                                                                  param.to_pos,
                                                                  param.force_misalign,
                                                                  stats.lock().unwrap().bytes_processed,
                                                                  PositionOrLength::Len(file_size),
                                                                  None);

    // seek to calculated position
    reader.seek(SeekFrom::Start(seek_to))?;

    loop {
        let mut stats = stats.lock().unwrap();

        break_if_atomic_bool!(ctrlc_stop_flag);

        break_if_reached_required_len!(stats.bytes_processed,
                                       required_len);

        let lazy_read_res = block_utils::read_block_lazily(&mut block,
                                                           &mut buffer,
                                                           &mut reader)?;

        stats.bytes_processed += lazy_read_res.len_read as u64;

        break_if_eof_seen!(lazy_read_res);

        if !lazy_read_res.usable { continue; }

        // update stats
        match block.block_type() {
            BlockType::Meta => {
                stats.meta_or_par_blocks_processed += 1;
            },
            BlockType::Data => {
                stats.data_or_par_blocks_processed += 1;
            }
        }

        // check if block matches required block type
        if let Some(x) = param.only_pick_block {
            if block.block_type() != x { continue; }
        }

        // check if block has the required UID
        if let Some(x) = param.only_pick_uid {
            if block.get_uid() != x { continue; }
        }

        // write block out
        let uid_str    = misc_utils::bytes_to_upper_hex_string(&block.get_uid());
        let path       = misc_utils::make_path(&[&param.out_dir, &uid_str]);
        let mut writer = FileWriter::new(&path,
                                         FileWriterParam { read     : false,
                                                           append   : true,
                                                           truncate : false,
                                                           buffered : false  })?;

        // use the original bytes which are still in the buffer
        writer.write(sbx_block::slice_buf(block.get_version(), &buffer))?;

        // check if there's any error in log handling
        log_handler.pop_error()?;
    }

    reporter.stop();
    log_handler.stop();

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
