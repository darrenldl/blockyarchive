use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use file_utils;
use misc_utils;
use std::io::SeekFrom;

use std::sync::atomic::AtomicBool;

use progress_report::*;
use cli_utils::setup_ctrlc_handler;

use file_reader::{FileReader,
                  FileReaderParam};
use file_writer::{FileWriter,
                  FileWriterParam};

use multihash;
use multihash::*;

use cli_utils::report_ref_block_info;

use general_error::Error;
use sbx_specs::Version;

use sbx_block::Block;
use sbx_block;
use sbx_specs::{ver_to_block_size,
                SBX_LARGEST_BLOCK_SIZE,
                ver_uses_rs,
                ver_to_usize};

use time_utils;
use block_utils;

const HASH_FILE_BLOCK_SIZE : usize = 4096;

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    version                     : Version,
    pub meta_blocks_decoded     : u64,
    pub data_blocks_decoded     : u64,
    pub data_par_blocks_decoded : u64,
    pub blocks_decode_failed    : u64,
    total_blocks                : u64,
    start_time                  : f64,
    end_time                    : f64,
    pub recorded_hash           : Option<multihash::HashBytes>,
    pub computed_hash           : Option<multihash::HashBytes>,
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

        if rs_enabled {
            writeln!(f, "SBX version                            : {} (0x{:X})",
                     ver_to_usize(self.version),
                     ver_to_usize(self.version))?;
            writeln!(f, "Block size used in decoding            : {}", block_size)?;
            writeln!(f, "Number of blocks processed             : {}", self.units_so_far())?;
            writeln!(f, "Number of blocks decoded (metadata)    : {}", self.meta_blocks_decoded)?;
            writeln!(f, "Number of blocks decoded (data only)   : {}", self.data_blocks_decoded)?;
            writeln!(f, "Number of blocks decoded (data parity) : {}", self.data_par_blocks_decoded)?;
            writeln!(f, "Number of blocks failed to decode      : {}", self.blocks_decode_failed)?;
            writeln!(f, "Time elapsed                           : {:02}:{:02}:{:02}", hour, minute, second)?;
            writeln!(f, "Recorded hash                          : {}", match *recorded_hash {
                None        => "N/A".to_string(),
                Some(ref h) => format!("{} - {}",
                                       hash_type_to_string(h.0),
                                       misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
            writeln!(f, "Hash of output file                    : {}", match (recorded_hash, computed_hash) {
                (&None,    &None)        => "N/A".to_string(),
                (&Some(_), &None)        => "N/A - recorded hash type is not supported by rsbx".to_string(),
                (_,        &Some(ref h)) => format!("{} - {}",
                                                    hash_type_to_string(h.0),
                                                    misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
        } else {
            writeln!(f, "SBX version                         : {}", ver_to_usize(self.version))?;
            writeln!(f, "Block size used in decoding         : {}", block_size)?;
            writeln!(f, "Number of blocks processed          : {}", self.units_so_far())?;
            writeln!(f, "Number of blocks decoded (metadata) : {}", self.meta_blocks_decoded)?;
            writeln!(f, "Number of blocks decoded (data)     : {}", self.data_blocks_decoded)?;
            writeln!(f, "Number of blocks failed to decode   : {}", self.blocks_decode_failed)?;
            writeln!(f, "Time elapsed                        : {:02}:{:02}:{:02}", hour, minute, second)?;
            writeln!(f, "Recorded hash                       : {}", match *recorded_hash {
                None        => "N/A".to_string(),
                Some(ref h) => format!("{} - {}",
                                       hash_type_to_string(h.0),
                                       misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
            writeln!(f, "Hash of output file                 : {}", match (recorded_hash, computed_hash) {
                (&None,    &None)        => "N/A".to_string(),
                (&Some(_), &None)        => "N/A - recorded hash type is not supported by rsbx".to_string(),
                (_,        &Some(ref h)) => format!("{} - {}",
                                                    hash_type_to_string(h.0),
                                                    misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
        }
        match (recorded_hash, computed_hash) {
            (&Some(ref recorded_hash), &Some(ref computed_hash)) => {
                if recorded_hash.1 == computed_hash.1 {
                    writeln!(f, "The output file hash matches the recorded hash")?;
                } else {
                    writeln!(f, "The output file does NOT match the recorded hash")?;
                }
            },
            (&Some(_),                 &None)                    => {
                writeln!(f, "No hash is available for output file")?;
            },
            (&None,                    &Some(_))                 => {
                writeln!(f, "No recorded hash is available")?;
            },
            (&None,                    &None)                    => {
                writeln!(f, "Neither recorded hash nor output file hash is available")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    no_meta            : bool,
    force_write        : bool,
    in_file            : String,
    out_file           : Option<String>,
    verbose            : bool,
    pr_verbosity_level : PRVerbosityLevel
}

impl Param {
    pub fn new(no_meta            : bool,
               force_write        : bool,
               in_file            : &str,
               out_file           : Option<&str>,
               verbose            : bool,
               pr_verbosity_level : PRVerbosityLevel) -> Param {
        Param {
            no_meta,
            force_write,
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
    pub fn new(file_metadata : &fs::Metadata) -> HashStats {
        HashStats {
            bytes_processed : 0,
            total_bytes     : file_metadata.len(),
            start_time      : 0.,
            end_time        : 0.,
        }
    }
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
            meta_blocks_decoded     : 0,
            data_blocks_decoded     : 0,
            data_par_blocks_decoded : 0,
            total_blocks,
            start_time              : 0.,
            end_time                : 0.,
            recorded_hash           : None,
            computed_hash           : None,
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

pub fn decode(param           : &Param,
              ref_block_pos   : u64,
              ref_block       : &Block,
              ctrlc_stop_flag : &Arc<AtomicBool>)
              -> Result<Stats, Error> {
    let metadata = file_utils::get_file_metadata(&param.in_file)?;

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;
    let out_file : &str = match param.out_file {
        None        => panic!(),
        Some(ref x) => x,
    };
    let mut writer = FileWriter::new(out_file,
                                     FileWriterParam { read     : false,
                                                       append   : false,
                                                       buffered : true   })?;

    let stats = Arc::new(Mutex::new(Stats::new(&ref_block, &metadata)));

    let reporter = ProgressReporter::new(&stats,
                                         "Data decoding progress",
                                         "blocks",
                                         param.pr_verbosity_level);

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

    let pred = block_pred_same_ver_uid!(ref_block);

    reporter.start();

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
                writer.seek(SeekFrom::Start(write_pos as u64))?;

                writer.write(sbx_block::slice_data_buf(ref_block.get_version(),
                                                       &buffer))?;
            }
        }
    }

    reporter.stop();

    // truncate file possibly
    if ref_block.is_meta() {
        match ref_block.get_FSZ().unwrap() {
            None    => {},
            Some(x) => {
                writer.set_len(x)?;
            }
        }
    } else {
        print_block!(
            "";
            "Warning : Reference block is not a metadata block, output file may contain data padding";
            "";)
    }

    let res = stats.lock().unwrap().clone();

    Ok(res)
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

    let mut reader = FileReader::new(&param.out_file.clone().unwrap(),
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    let metadata = reader.metadata()?;

    let stats = Arc::new(Mutex::new(HashStats::new(&metadata)));

    let reporter = ProgressReporter::new(&stats,
                                         "Output file hashing progress",
                                         "bytes",
                                         param.pr_verbosity_level);

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
    let ctrlc_stop_flag = setup_ctrlc_handler();

    let (ref_block_pos, ref_block) = get_ref_block!(param,
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
    let out_file_path : String = match param.out_file {
        None => {
            match recorded_file_name {
                None    => { return Err(Error::with_message("No original file name was found in SBX container and no output file name/path was provided")); },
                Some(x) => x
            }
        },
        Some(ref out) => {
            if file_utils::check_if_file_is_dir(&out) {
                match recorded_file_name {
                    None    => { return Err(Error::with_message(&format!("No original file name was found in SBX container and \"{}\" is a directory",
                                                                         &out))); }
                    Some(x) => {
                        misc_utils::make_path(&[&out, &x])
                    }
                }
            } else {
                out.clone()
            }
        }
    };

    // check if can write out
    if !param.force_write {
        if file_utils::check_if_file_exists(&out_file_path) {
            return Err(Error::with_message(&format!("File \"{}\" already exists",
                                                    out_file_path)));
        }
    }

    // regenerate param
    let param = Param::new(param.no_meta,
                           param.force_write,
                           &param.in_file,
                           Some(&out_file_path),
                           param.verbose,
                           param.pr_verbosity_level);

    let mut stats = decode(&param,
                           ref_block_pos,
                           &ref_block,
                           &ctrlc_stop_flag)?;

    stats.computed_hash = hash(&param,
                               &ref_block,
                               &ctrlc_stop_flag)?;

    Ok(Some(stats))
}
