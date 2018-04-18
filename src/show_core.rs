use std::sync::{Arc, Mutex};
use std::fmt;
use file_utils;
use misc_utils;
use misc_utils::RequiredLenAndSeekTo;

use cli_utils::report_ref_block_info;
use cli_utils::setup_ctrlc_handler;

use std::io::SeekFrom;

use progress_report::*;

use file_reader::{FileReader,
                  FileReaderParam};

use multihash::*;

use general_error::Error;

use sbx_block::Block;
use sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use sbx_specs::ver_to_usize;
use sbx_specs::ver_uses_rs;

use time_utils;
use block_utils;

use block_utils::RefBlockChoice;
use sbx_block::BlockType;

use json_utils::JSONContext;

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub bytes_processed : u64,
    pub total_bytes     : u64,
    meta_block_count    : u64,
    start_time          : f64,
    end_time            : f64,
    json_enabled        : bool,
}

impl Stats {
    pub fn new(file_size    : u64,
               json_enabled : bool) -> Stats {
        Stats {
            bytes_processed   : 0,
            total_bytes       : file_size,
            meta_block_count  : 0,
            start_time        : 0.,
            end_time          : 0.,
            json_enabled,
        }
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      { self.bytes_processed }

    fn total_units(&self)        -> u64      { self.total_bytes }
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        if self.meta_block_count == 0 {
            writeln!(f, "No metadata blocks found")
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    show_all            : bool,
    guess_burst         : bool,
    force_misalign      : bool,
    json_enabled        : bool,
    from_pos            : Option<u64>,
    to_pos              : Option<u64>,
    in_file             : String,
    pr_verbosity_level  : PRVerbosityLevel
}

impl Param {
    pub fn new(show_all            : bool,
               guess_burst         : bool,
               force_misalign      : bool,
               json_enabled        : bool,
               from_pos            : Option<u64>,
               to_pos              : Option<u64>,
               in_file             : &str,
               pr_verbosity_level  : PRVerbosityLevel) -> Param {
        Param {
            show_all,
            guess_burst,
            force_misalign,
            json_enabled,
            from_pos,
            to_pos,
            in_file : String::from(in_file),
            pr_verbosity_level,
        }
    }
}

pub fn show_file(param : &Param)
                 -> Result<Stats, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_enabled);

    let json_context = JSONContext::new(json_enabled);

    if param.guess_burst {
        print_if_not_json!(json_enabled, "Guessing burst error resistance level");
        print_if_not_json!(json_enabled, "");

        let (ref_block_pos, ref_block) =
            match block_utils::get_ref_block(&param.in_file,
                                             RefBlockChoice::MustBe(BlockType::Meta),
                                             param.pr_verbosity_level,
                                             &ctrlc_stop_flag)? {
                None => { return Err(Error::with_message("Failed to find reference block")); },
                Some(x) => x,
            };

        report_ref_block_info(json_enabled, None, ref_block_pos, &ref_block);

        print_if_not_json!(json_enabled, "");

        if ver_uses_rs(ref_block.get_version()) {
            match block_utils::guess_burst_err_resistance_level(&param.in_file,
                                                                ref_block_pos,
                                                                &ref_block) {
                Err(e)      => { return Err(Error::with_message(&format!("Error encountered when guessing : {}", e))) },
                Ok(None)    => print_if_not_json!(json_enabled, "Failed to guess level"),
                Ok(Some(x)) => print_maybe_json!(json_enabled, "Best guess for burst error resistance level : {}", x => skip_quotes),
            }
        } else {
            print_if_not_json!(json_enabled, "Reference block version does not use Reed-Solomon erasure code");
        }

        print_if_not_json!(json_enabled, "");
        print_if_not_json!(json_enabled, "========================================");
        print_if_not_json!(json_enabled, "");
    }

    let file_size = file_utils::get_file_size(&param.in_file)?;

    let stats = Arc::new(Mutex::new(Stats::new(file_size,
                                               param.json_enabled)));

    let reporter = ProgressReporter::new(&stats,
                                         "Metadata block scanning progress",
                                         "bytes",
                                         param.pr_verbosity_level);

    let mut block = Block::dummy();
    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    // calulate length to read and position to seek to
    let RequiredLenAndSeekTo { required_len, seek_to } =
        misc_utils::calc_required_len_and_seek_to_from_byte_range_inc(param.from_pos,
                                                                      param.to_pos,
                                                                      param.force_misalign,
                                                                      stats.lock().unwrap().bytes_processed,
                                                                      file_size);

    // seek to calculated position
    reader.seek(SeekFrom::Start(seek_to))?;

    reporter.start();

    let mut meta_block_count : u64 = 0;

    let mut block_pos       : u64;
    let mut bytes_processed : u64 = 0;

    if param.guess_burst {
        print_if_json!(json_context, ",\"blocks\" : [");
    } else {
        print_if_json!(json_context, "\"blocks\" : [");
    }

    loop {
        break_if_atomic_bool!(ctrlc_stop_flag);

        break_if_reached_required_len!(bytes_processed,
                                       required_len);

        let lazy_read_res = block_utils::read_block_lazily(&mut block,
                                                           &mut buffer,
                                                           &mut reader)?;

        block_pos        = bytes_processed;
        bytes_processed += lazy_read_res.len_read as u64;

        stats.lock().unwrap().bytes_processed = bytes_processed;

        break_if_eof_seen!(lazy_read_res);

        if !lazy_read_res.usable { continue; }

        if block.is_meta() {
            reporter.pause();

            if meta_block_count == 0 {
                print_if_json!(json_context, "{{");
            } else {
                print_if_json!(json_context, ",{{");
            }

            if param.show_all {
                if meta_block_count > 0 {
                    print_if_not_json!(json_context, "");
                }
                print_maybe_json!(json_context,    "Metadata block number : {}", meta_block_count => skip_quotes);
                print_if_not_json!(json_context,   "========================================");
            } else {
                print_field_if_json!(json_context, "Metadata block number : {}", meta_block_count => skip_quotes);
            }

            print_if_not_json!(json_context, "Found at byte          : {} (0x{:X})",
                               block_pos + seek_to,
                               block_pos + seek_to);
            print_maybe_json!(json_context,  "Found at byte          : {}",
                              block_pos + seek_to => skip_quotes);

            print_if_not_json!(json_context, "");

            print_maybe_json!(json_context,  "File UID               : {}",
                              misc_utils::bytes_to_upper_hex_string(&block.get_uid()));
            print_maybe_json!(json_context,  "File name              : {}",
                              block.get_FNM().unwrap().unwrap_or("N/A".to_string()));
            print_maybe_json!(json_context,  "SBX container name     : {}",
                              block.get_SNM().unwrap().unwrap_or("N/A".to_string()));
            print_maybe_json!(json_context, "SBX container version  : {}",
                              if ver_uses_rs(block.get_version()) && !json_context {
                                  format!("{} (0x{:X})",
                                          ver_to_usize(block.get_version()),
                                          ver_to_usize(block.get_version()))
                              } else {
                                  ver_to_usize(block.get_version()).to_string()
                              });
            print_maybe_json!(json_context, "RS data shard count    : {}",
                              if ver_uses_rs(block.get_version()) {
                                  match block.get_RSD().unwrap() {
                                      None    => null_if_json_else!(json_context, "N/A").to_string(),
                                      Some(x) => x.to_string(),
                                  }
                              } else {
                                  "version does not use RS".to_string()
                              }                                                    => skip_quotes);
            print_maybe_json!(json_context, "RS parity shard count  : {}",
                              if ver_uses_rs(block.get_version()) {
                                  match block.get_RSP().unwrap() {
                                      None    => null_if_json_else!(json_context, "N/A").to_string(),
                                      Some(x) => x.to_string(),
                                  }
                              } else {
                                  "version does not use RS".to_string()
                              }                                                    => skip_quotes);
            print_maybe_json!(json_context, "File size              : {}", match block.get_FSZ().unwrap() {
                None    => null_if_json_else!(json_context, "N/A").to_string(),
                Some(x) => x.to_string()
            }                                                                      => skip_quotes);
            print_maybe_json!(json_context, "File modification time : {}", match block.get_FDT().unwrap() {
                None    => null_if_json_else!(json_context, "N/A").to_string(),
                Some(x) => match (time_utils::i64_secs_to_date_time_string(x, time_utils::TimeMode::UTC),
                                  time_utils::i64_secs_to_date_time_string(x, time_utils::TimeMode::Local)) {
                    (Some(u), Some(l)) => format!("{} (UTC)  {} (Local)", u, l),
                    _                  => "Invalid recorded date time".to_string(),
                }
            });
            print_maybe_json!(json_context, "SBX encoding time      : {}", match block.get_SDT().unwrap() {
                None    => null_if_json_else!(json_context, "N/A").to_string(),
                Some(x) => match (time_utils::i64_secs_to_date_time_string(x, time_utils::TimeMode::UTC),
                                  time_utils::i64_secs_to_date_time_string(x, time_utils::TimeMode::Local)) {
                    (Some(u), Some(l)) => format!("{} (UTC)  {} (Local)", u, l),
                    _                  => "Invalid recorded date time".to_string(),
                }
            });
            print_maybe_json!(json_context, "Hash                   : {}", match block.get_HSH().unwrap() {
                None    => null_if_json_else!(json_context, "N/A").to_string(),
                Some(h) => format!("{} - {}",
                                   hash_type_to_string(h.0),
                                   misc_utils::bytes_to_lower_hex_string(&h.1))
            });

            meta_block_count += 1;

            reporter.resume();

            print_if_json!(json_context, "}}");

            if !param.show_all { break; }
        }
    }

    print_if_json!(json_context, "]");

    reporter.stop();

    stats.lock().unwrap().meta_block_count = meta_block_count;

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
