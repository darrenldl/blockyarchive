use std::sync::{Arc, Mutex};
use std::fmt;
use file_utils;
use misc_utils;
use std::io::SeekFrom;

use json_printer::{JSONPrinter,
                   BracketType};

use std::sync::atomic::AtomicBool;

use progress_report::*;
use cli_utils::setup_ctrlc_handler;

use file_reader::{FileReader,
                  FileReaderParam};
use file_writer::{FileWriter,
                  FileWriterParam};
use writer::{Writer,
             WriterType};

use multihash;
use multihash::*;

use cli_utils::report_ref_block_info;

use general_error::Error;
use sbx_specs::Version;

use sbx_block::Block;
use sbx_block;
use sbx_specs::{ver_to_block_size,
                ver_to_data_size,
                SBX_LARGEST_BLOCK_SIZE,
                SBX_FILE_UID_LEN,
                ver_uses_rs,
                SBX_LAST_SEQ_NUM,
                ver_to_usize};

use time_utils;
use block_utils;

use block_utils::RefBlockChoice;

const HASH_FILE_BLOCK_SIZE : usize = 4096;

const BLANK_BUFFER : [u8; 4096] = [0; 4096];

#[derive(Clone, Debug)]
pub struct Stats {
    uid                         : [u8; SBX_FILE_UID_LEN],
    version                     : Version,
    pub meta_blocks_decoded     : u64,
    pub data_blocks_decoded     : u64,
    pub data_par_blocks_decoded : u64,
    pub blocks_decode_failed    : u64,
    pub in_file_size            : u64,
    pub out_file_size           : u64,
    total_blocks                : u64,
    start_time                  : f64,
    end_time                    : f64,
    pub recorded_hash           : Option<multihash::HashBytes>,
    pub computed_hash           : Option<multihash::HashBytes>,
    json_printer                : Arc<JSONPrinter>,
}

struct HashStats {
    pub bytes_processed : u64,
    pub total_bytes     : u64,
    start_time          : f64,
    end_time            : f64,
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let rs_enabled              = ver_uses_rs(self.version);
        let block_size              = ver_to_block_size(self.version);
        let recorded_hash           = &self.recorded_hash;
        let computed_hash           = &self.computed_hash;
        let time_elapsed            = (self.end_time - self.start_time) as i64;
        let (hour, minute, second)  = time_utils::seconds_to_hms(time_elapsed);

        let json_printer = &self.json_printer;

        json_printer.write_open_bracket(f, Some("stats"), BracketType::Curly)?;

        if rs_enabled {
            write_maybe_json!(f, json_printer, "File UID                               : {}",
                              misc_utils::bytes_to_upper_hex_string(&self.uid))?;
            write_maybe_json!(f, json_printer, "SBX version                            : {}", ver_to_usize(self.version))?;
            write_maybe_json!(f, json_printer, "Block size used in decoding            : {}", block_size                   => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks processed             : {}", self.units_so_far()          => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks decoded (metadata)    : {}", self.meta_blocks_decoded     => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks decoded (data only)   : {}", self.data_blocks_decoded     => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks decoded (data parity) : {}", self.data_par_blocks_decoded => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks failed to decode      : {}", self.blocks_decode_failed    => skip_quotes)?;
            write_maybe_json!(f, json_printer, "File size                              : {}", self.out_file_size           => skip_quotes)?;
            write_maybe_json!(f, json_printer, "SBX container size                     : {}", self.in_file_size            => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Time elapsed                           : {:02}:{:02}:{:02}", hour, minute, second)?;
            write_maybe_json!(f, json_printer, "Recorded hash                          : {}", match *recorded_hash {
                None        => null_if_json_else!(json_printer, "N/A").to_string(),
                Some(ref h) => format!("{} - {}",
                                       hash_type_to_string(h.0),
                                       misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
            write_maybe_json!(f, json_printer, "Hash of output file                    : {}", match (recorded_hash, computed_hash) {
                (&None,    &None)        => null_if_json_else!(json_printer, "N/A").to_string(),
                (&Some(_), &None)        => null_if_json_else!(json_printer, "N/A - recorded hash type is not supported by rsbx").to_string(),
                (_,        &Some(ref h)) => format!("{} - {}",
                                                               hash_type_to_string(h.0),
                                                               misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
        } else {
            write_maybe_json!(f, json_printer, "File UID                            : {}",
                              misc_utils::bytes_to_upper_hex_string(&self.uid))?;
            write_maybe_json!(f, json_printer, "SBX version                         : {}", ver_to_usize(self.version))?;
            write_maybe_json!(f, json_printer, "Block size used in decoding         : {}", block_size                   => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks processed          : {}", self.units_so_far()          => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks decoded (metadata) : {}", self.meta_blocks_decoded     => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks decoded (data)     : {}", self.data_blocks_decoded     => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks failed to decode   : {}", self.blocks_decode_failed    => skip_quotes)?;
            write_maybe_json!(f, json_printer, "File size                           : {}", self.out_file_size           => skip_quotes)?;
            write_maybe_json!(f, json_printer, "SBX container size                  : {}", self.in_file_size            => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Time elapsed                        : {:02}:{:02}:{:02}", hour, minute, second)?;
            write_maybe_json!(f, json_printer, "Recorded hash                       : {}", match *recorded_hash {
                None        => null_if_json_else!(json_printer, "N/A").to_string(),
                Some(ref h) => format!("{} - {}",
                                       hash_type_to_string(h.0),
                                       misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
            write_maybe_json!(f, json_printer, "Hash of output file                 : {}", match (recorded_hash, computed_hash) {
                (&None,    &None)        => null_if_json_else!(json_printer, "N/A").to_string(),
                (&Some(_), &None)        => null_if_json_else!(json_printer, "N/A - recorded hash type is not supported by rsbx").to_string(),
                (_,        &Some(ref h)) => format!("{} - {}",
                                                               hash_type_to_string(h.0),
                                                               misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
        }
        match (recorded_hash, computed_hash) {
            (&Some(ref recorded_hash), &Some(ref computed_hash)) => {
                if recorded_hash.1 == computed_hash.1 {
                    write_if!(not_json => f, json_printer => "The output file hash matches the recorded hash";)?;
                } else {
                    write_if!(not_json => f, json_printer => "The output file does NOT match the recorded hash";)?;
                }
            },
            (&Some(_),                 &None)                    => {
                write_if!(not_json => f, json_printer => "No hash is available for output file";)?;
            },
            (&None,                    &Some(_))                 => {
                write_if!(not_json => f, json_printer => "No recorded hash is available";)?;
            },
            (&None,                    &None)                    => {
                write_if!(not_json => f, json_printer => "Neither recorded hash nor output file hash is available";)?;
            }
        }

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    ref_block_choice   : RefBlockChoice,
    force_write        : bool,
    json_printer       : Arc<JSONPrinter>,
    in_file            : String,
    out_file           : Option<String>,
    verbose            : bool,
    pr_verbosity_level : PRVerbosityLevel
}

impl Param {
    pub fn new(ref_block_choice   : RefBlockChoice,
               force_write        : bool,
               json_printer       : &Arc<JSONPrinter>,
               in_file            : &str,
               out_file           : Option<&str>,
               verbose            : bool,
               pr_verbosity_level : PRVerbosityLevel) -> Param {
        Param {
            ref_block_choice,
            force_write,
            json_printer : Arc::clone(json_printer),
            in_file  : String::from(in_file),
            out_file : match out_file {
                None    => None,
                Some(x) => Some(String::from(x))
            },
            verbose,
            pr_verbosity_level,
        }
    }
}

impl HashStats {
    pub fn new(file_size : u64) -> HashStats {
        HashStats {
            bytes_processed : 0,
            total_bytes     : file_size,
            start_time      : 0.,
            end_time        : 0.,
        }
    }
}

impl Stats {
    pub fn new(ref_block    : &Block,
               in_file_size : u64,
               json_printer : &Arc<JSONPrinter>) -> Stats {
        use file_utils::from_container_size::calc_total_block_count;
        let total_blocks =
            calc_total_block_count(ref_block.get_version(),
                                   in_file_size);
        Stats {
            uid                     : ref_block.get_uid(),
            version                 : ref_block.get_version(),
            blocks_decode_failed    : 0,
            meta_blocks_decoded     : 0,
            data_blocks_decoded     : 0,
            data_par_blocks_decoded : 0,
            in_file_size,
            out_file_size           : 0,
            total_blocks,
            start_time              : 0.,
            end_time                : 0.,
            recorded_hash           : None,
            computed_hash           : None,
            json_printer            : Arc::clone(json_printer),
        }
    }
}

impl ProgressReport for HashStats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      { self.bytes_processed }

    fn total_units(&self)        -> u64      { self.total_bytes }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      {
        (self.meta_blocks_decoded
         + self.data_blocks_decoded
         + self.data_par_blocks_decoded
         + self.blocks_decode_failed) as u64
    }

    fn total_units(&self)        -> u64      { self.total_blocks as u64 }
}

fn write_data_only_block(data_par_shards              : Option<(usize, usize)>,
                         is_last_data_block           : bool,
                         data_size_of_last_data_block : Option<u64>,
                         ref_block                    : &Block,
                         block                        : &Block,
                         writer                       : &mut Writer,
                         hash_ctx                     : &mut Option<hash::Ctx>,
                         buffer                       : &[u8])
                         -> Result<(), Error> {
    let slice =
        if is_last_data_block {
            &sbx_block::slice_data_buf(ref_block.get_version(),
                                       &buffer)
                [0..data_size_of_last_data_block.unwrap() as usize]
        } else {
            sbx_block::slice_data_buf(ref_block.get_version(),
                                      &buffer)
        };

    match data_par_shards {
        Some((data, par)) => {
            if !block.is_parity(data, par) {
                writer.write(slice)?;

                if let &mut Some(ref mut ctx) = hash_ctx {
                    ctx.update(slice);
                }
            }
        },
        None              => {
            writer.write(slice)?;

            if let &mut Some(ref mut ctx) = hash_ctx {
                ctx.update(slice);
            }
        }
    }

    Ok(())
}

fn write_blank_chunk(ref_block : &Block,
                     writer    : &mut Writer,
                     hash_ctx  : &mut Option<hash::Ctx>)
                     -> Result<(), Error> {
    let slice =
        &sbx_block::slice_data_buf(ref_block.get_version(),
                                   &BLANK_BUFFER);

    writer.write(slice)?;

    if let &mut Some(ref mut ctx) = hash_ctx {
        ctx.update(slice);
    }

    Ok(())
}

pub fn decode(param           : &Param,
              ref_block_pos   : u64,
              ref_block       : &Block,
              ctrlc_stop_flag : &Arc<AtomicBool>)
              -> Result<(Stats, Option<HashBytes>), Error> {
    let in_file_size = file_utils::get_file_size(&param.in_file)?;

    let orig_file_size =
        if ref_block.is_meta() {
            ref_block.get_FSZ().unwrap()
        } else {
            None
        };

    let data_par_burst =
        if ver_uses_rs(ref_block.get_version()) && ref_block.is_meta() {
            match (ref_block.get_RSD(), ref_block.get_RSP()) {
                (Ok(Some(data)), Ok(Some(parity))) => {
                    let data   = data   as usize;
                    let parity = parity as usize;

                    // try to obtain burst error resistance level
                    match block_utils::guess_burst_err_resistance_level(&param.in_file,
                                                                        ref_block_pos,
                                                                        &ref_block) {
                        Ok(Some(l)) => Some((data, parity, l)),
                        _           => Some((data, parity, 0)), // assume burst resistance level is 0
                    }
                },
                _                                  => None,
            }
        } else {
            None
        };

    let data_size                    = ver_to_data_size(ref_block.get_version());
    let data_size_of_last_data_block =
        match orig_file_size {
            Some(orig_file_size) =>
                match orig_file_size % data_size as u64 {
                    0 => Some(data_size as u64),
                    x => Some(x),
                }
            None    => None,
        };

    let json_printer = &param.json_printer;

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    let mut writer = match param.out_file {
        Some(ref out_file) => Writer::new(WriterType::File(FileWriter::new(out_file,
                                                                           FileWriterParam { read     : false,
                                                                                             append   : false,
                                                                                             buffered : true   })?)),
        None               => Writer::new(WriterType::Stdout(std::io::stdout())),
    };

    let stats = Arc::new(Mutex::new(Stats::new(&ref_block,
                                               in_file_size,
                                               &param.json_printer)));

    let mut hash_bytes = None;

    let reporter = ProgressReporter::new(&stats,
                                         "Data decoding progress",
                                         "blocks",
                                         param.pr_verbosity_level,
                                         param.json_printer.json_enabled());

    let mut block = Block::dummy();

    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    // get hash possibly
    if ref_block.is_meta() {
        match ref_block.get_HSH().unwrap() {
            None    => {},
            Some(x) => { stats.lock().unwrap().recorded_hash = Some(x); }
        }
    }

    // deal with RS related stuff
    let rs_enabled = ver_uses_rs(ref_block.get_version());

    if rs_enabled { // must be metadata block, and must contain fields `RSD`, `RSP`
        return_if_ref_not_meta!(ref_block_pos, ref_block, "decode");
    }

    let data_par_shards =
        if rs_enabled {
            Some((get_RSD_from_ref_block!(ref_block_pos, ref_block, "decode"),
                  get_RSP_from_ref_block!(ref_block_pos, ref_block, "decode")))
        } else {
            None
        };

    let version = ref_block.get_version();

    let block_size = ver_to_block_size(version);

    let pred = block_pred_same_ver_uid!(ref_block);

    reporter.start();

    match param.out_file {
        Some(_) => // output to file
            loop {
                break_if_atomic_bool!(ctrlc_stop_flag);

                // read at reference block block size
                let read_res = reader.read(sbx_block::slice_buf_mut(ref_block.get_version(),
                                                                    &mut buffer))?;

                break_if_eof_seen!(read_res);

                if let Err(_) = block.sync_from_buffer(&buffer, Some(&pred)) {
                    stats.lock().unwrap().blocks_decode_failed += 1;
                    continue;
                }

                if block.is_meta() { // do nothing if block is meta
                    stats.lock().unwrap().meta_blocks_decoded += 1;
                } else {
                    match data_par_shards {
                        Some((data, par)) => {
                            if block.is_parity(data, par) {
                                stats.lock().unwrap().data_par_blocks_decoded += 1;
                            } else {
                                stats.lock().unwrap().data_blocks_decoded += 1;
                            }
                        },
                        None => {
                            stats.lock().unwrap().data_blocks_decoded += 1;
                        }
                    }

                    // write data block
                    if let Some(write_pos) =
                        sbx_block::calc_data_chunk_write_pos(ref_block.get_version(),
                                                             block.get_seq_num(),
                                                             data_par_shards)
                    {
                        writer.seek(SeekFrom::Start(write_pos as u64)).unwrap()?;

                        writer.write(sbx_block::slice_data_buf(ref_block.get_version(),
                                                               &buffer))?;
                    }
                }
            },
        None    => { // output to stdout
            let stored_hash_bytes =
                if ref_block.is_data() {
                    None
                } else {
                    ref_block.get_HSH().unwrap()
                };

            let mut hash_ctx =
                match stored_hash_bytes {
                    None          => None,
                    Some((ht, _)) => match hash::Ctx::new(ht) {
                        Err(()) => None,
                        Ok(ctx) => Some(ctx),
                    }
                };

            let total_block_count = {
                use file_utils::from_orig_file_size::calc_total_block_count_exc_burst_gaps;
                match ref_block.get_FSZ() {
                    Ok(Some(x)) =>
                        calc_total_block_count_exc_burst_gaps(version,
                                                              None,
                                                              data_par_burst,
                                                              x),
                    _           => {
                        print_if!(not_json => json_printer =>
                                  "Warning :";
                                  "";
                                  "    No recorded file size found, using container file size to estimate total";
                                  "    number of blocks. This may overestimate total number of blocks, and may";
                                  "    show incorrect block counts.";
                                  "";
                        );
                        let file_size = file_utils::get_file_size(&param.in_file)?;
                        file_size / block_size as u64
                    },
                }
            };

            let total_data_chunk_count = match orig_file_size {
                Some(orig_file_size) => Some(file_utils::from_orig_file_size::calc_data_chunk_count(version, orig_file_size)),
                None                 => None,
            };

            match data_par_burst {
                Some((data, parity, _)) => { // do burst resistant pattern read
                    let mut seq_num = 1;
                    while seq_num <= SBX_LAST_SEQ_NUM {
                        break_if_atomic_bool!(ctrlc_stop_flag);

                        if stats.lock().unwrap().units_so_far() >= total_block_count { break; }

                        let pos = sbx_block::calc_data_block_write_pos(ref_block.get_version(),
                                                                       seq_num,
                                                                       None,
                                                                       data_par_burst);

                        reader.seek(SeekFrom::Start(pos))?;

                        // read at reference block block size
                        let read_res = reader.read(sbx_block::slice_buf_mut(ref_block.get_version(),
                                                                            &mut buffer))?;

                        if read_res.eof_seen {
                            stats.lock().unwrap().blocks_decode_failed += 1;
                            continue;
                        }

                        match block.sync_from_buffer(&buffer, Some(&pred)) {
                            Ok(_)  => {
                                if block.get_seq_num() != seq_num {
                                    stats.lock().unwrap().blocks_decode_failed += 1;

                                    write_blank_chunk(&ref_block,
                                                      &mut writer,
                                                      &mut hash_ctx)?;

                                    continue;
                                }
                            },
                            Err(_) => {
                                stats.lock().unwrap().blocks_decode_failed += 1;

                                write_blank_chunk(&ref_block,
                                                  &mut writer,
                                                  &mut hash_ctx)?;

                                continue;
                            },
                        }

                        if block.is_meta() { // do nothing if block is meta
                            stats.lock().unwrap().meta_blocks_decoded += 1;
                        } else {
                            if block.is_parity(data, parity) {
                                stats.lock().unwrap().data_par_blocks_decoded += 1;
                            } else {
                                stats.lock().unwrap().data_blocks_decoded += 1;

                                let is_last_data_block = match total_data_chunk_count {
                                    Some(count) => stats.lock().unwrap().data_blocks_decoded == count,
                                    None        => false,
                                };

                                // write data chunk
                                write_data_only_block(data_par_shards,
                                                      is_last_data_block,
                                                      data_size_of_last_data_block,
                                                      &ref_block,
                                                      &block,
                                                      &mut writer,
                                                      &mut hash_ctx,
                                                      &buffer)?;

                                if is_last_data_block { break; }
                            }
                        }

                        seq_num += 1;
                    }
                },
                None                        => { // do sequential read
                    let mut seq_num = 1;
                    while seq_num <= SBX_LAST_SEQ_NUM {
                        break_if_atomic_bool!(ctrlc_stop_flag);

                        // read at reference block block size
                        let read_res = reader.read(sbx_block::slice_buf_mut(ref_block.get_version(),
                                                                            &mut buffer))?;

                        break_if_eof_seen!(read_res);

                        match block.sync_from_buffer(&buffer, Some(&pred)) {
                            Ok(_)  => {
                                if block.get_seq_num() != seq_num {
                                    stats.lock().unwrap().blocks_decode_failed += 1;

                                    write_blank_chunk(&ref_block,
                                                      &mut writer,
                                                      &mut hash_ctx)?;

                                    continue;
                                }
                            },
                            Err(_) => {
                                stats.lock().unwrap().blocks_decode_failed += 1;

                                write_blank_chunk(&ref_block,
                                                  &mut writer,
                                                  &mut hash_ctx)?;

                                continue;
                            },
                        }

                        if block.is_meta() { // do nothing if block is meta
                            stats.lock().unwrap().meta_blocks_decoded += 1;
                        } else {
                            match data_par_shards {
                                Some((data, par)) => {
                                    if block.is_parity(data, par) {
                                        stats.lock().unwrap().data_par_blocks_decoded += 1;
                                    } else {
                                        stats.lock().unwrap().data_blocks_decoded += 1;
                                    }
                                },
                                None => {
                                    stats.lock().unwrap().data_blocks_decoded += 1;
                                }
                            }

                            let is_last_data_block = match total_data_chunk_count {
                                Some(count) => stats.lock().unwrap().data_blocks_decoded == count,
                                None        => false,
                            };

                            // write data block
                            write_data_only_block(None,
                                                  is_last_data_block,
                                                  data_size_of_last_data_block,
                                                  &ref_block,
                                                  &block,
                                                  &mut writer,
                                                  &mut hash_ctx,
                                                  &buffer)?;

                            if is_last_data_block { break; }
                        }

                        seq_num += 1;
                    }
                }
            }

            if let Some(ctx) = hash_ctx {
                hash_bytes = Some(ctx.finish_into_hash_bytes());
            }
        },
    }
    reporter.stop();

    // truncate file possibly
    if ref_block.is_meta() {
        match ref_block.get_FSZ().unwrap() {
            None    => {},
            Some(x) => {
                if let Some(r) = writer.set_len(x) {
                    r?;
                }
            }
        }
    } else {
        if !json_printer.json_enabled() {
            print_block!(
                "";
                "Warning :";
                "";
                "    Reference block is not a metadata block, output file";
                "    may contain data padding.";
                "";)
        }
    }

    let data_blocks_decoded = stats.lock().unwrap().data_blocks_decoded;

    stats.lock().unwrap().out_file_size = match writer.get_file_size() {
        Some(r) => r?,
        None    => data_blocks_decoded * ver_to_data_size(ref_block.get_version()) as u64,
    };

    let res = stats.lock().unwrap().clone();

    Ok((res, hash_bytes))
}

fn hash(param           : &Param,
        ref_block       : &Block,
        ctrlc_stop_flag : &Arc<AtomicBool>)
        -> Result<Option<HashBytes>, Error> {
    let hash_bytes : Option<HashBytes> =
        if ref_block.is_data() {
            None
        } else {
            ref_block.get_HSH().unwrap()
        };

    let mut hash_ctx : hash::Ctx =
        match hash_bytes {
            None          => { return Ok(None); },
            Some((ht, _)) => match hash::Ctx::new(ht) {
                Err(()) => { return Ok(None); },
                Ok(ctx) => ctx,
            }
        };

    let mut reader = match param.out_file {
        Some(ref f) => FileReader::new(f,
                                       FileReaderParam { write    : false,
                                                         buffered : true   })?,
        None        => return Ok(None),
    };

    let file_size = reader.get_file_size()?;

    let stats = Arc::new(Mutex::new(HashStats::new(file_size)));

    let reporter = ProgressReporter::new(&stats,
                                         "Output file hashing progress",
                                         "bytes",
                                         param.pr_verbosity_level,
                                         param.json_printer.json_enabled());

    let mut buffer : [u8; HASH_FILE_BLOCK_SIZE] = [0; HASH_FILE_BLOCK_SIZE];

    reporter.start();

    loop {
        break_if_atomic_bool!(ctrlc_stop_flag);

        let read_res = reader.read(&mut buffer)?;

        // update hash context/state
        hash_ctx.update(&buffer[0..read_res.len_read]);

        // update stats
        stats.lock().unwrap().bytes_processed += read_res.len_read as u64;

        break_if_eof_seen!(read_res);
    }

    reporter.stop();

    Ok(Some(hash_ctx.finish_into_hash_bytes()))
}

pub fn decode_file(param : &Param)
                   -> Result<Option<Stats>, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let json_printer = &param.json_printer;

    let (ref_block_pos, ref_block) = get_ref_block!(param,
                                                    json_printer,
                                                    ctrlc_stop_flag);

    // get FNM of ref_block
    let recorded_file_name : Option<String> =
        if ref_block.is_data() {
            None
        } else {
            match ref_block.get_FNM().unwrap() {
                None    => None,
                Some(x) => Some(file_utils::get_file_name_part_of_path(&x))
            }
        };

    // compute output file name
    let out_file_path : Option<String> = match param.out_file {
        None => {
            match recorded_file_name {
                None    => { return Err(Error::with_message("No original file name was found in SBX container and no output file name/path was provided")); },
                Some(x) => Some(x)
            }
        },
        Some(ref out) => {
            if        file_utils::check_if_file_is_stdout(out) {
                None
            } else if file_utils::check_if_file_is_dir(out) {
                match recorded_file_name {
                    None    => { return Err(Error::with_message(&format!("No original file name was found in SBX container and \"{}\" is a directory",
                                                                         &out))); }
                    Some(x) => {
                        Some(misc_utils::make_path(&[out, &x]))
                    }
                }
            } else {
                Some(out.clone())
            }
        }
    };

    // check if can write out
    if let Some(ref out_file_path) = out_file_path {
        if !param.force_write {
            if file_utils::check_if_file_exists(out_file_path) {
                return Err(Error::with_message(&format!("File \"{}\" already exists",
                                                        out_file_path)));
            }
        }
    }

    let out_file_path : Option<&str> = match out_file_path {
        Some(ref f) => Some(f),
        None        => None,
    };

    // regenerate param
    let param = Param::new(param.ref_block_choice,
                           param.force_write,
                           &param.json_printer,
                           &param.in_file,
                           out_file_path,
                           param.verbose,
                           param.pr_verbosity_level);

    let (mut stats, hash_res) = decode(&param,
                                       ref_block_pos,
                                       &ref_block,
                                       &ctrlc_stop_flag)?;

    stats.computed_hash = match hash_res {
        Some(r) => Some(r),
        None    => hash(&param,
                        &ref_block,
                        &ctrlc_stop_flag)?,
    };

    Ok(Some(stats))
}
