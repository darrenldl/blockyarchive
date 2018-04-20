use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use file_utils;
use time_utils;
use misc_utils;
use std::io::SeekFrom;

use json_printer::{JSONPrinter,
                   BracketType};

use progress_report::*;

use std::time::UNIX_EPOCH;
use cli_utils::setup_ctrlc_handler;

use file_reader::{FileReader,
                  FileReaderParam};
use file_writer::{FileWriter,
                  FileWriterParam};

use multihash;

use general_error::Error;
use sbx_specs::Version;
use rs_codec::RSEncoder;

use sbx_block::{Block,
                BlockType,
                Metadata,
                calc_data_block_write_pos,
                make_too_much_meta_err_string};

use sbx_block;
use sbx_specs::{ver_to_usize,
                ver_to_block_size,
                ver_to_data_size,
                ver_forces_meta_enabled,
                SBX_FILE_UID_LEN,
                SBX_LARGEST_BLOCK_SIZE,
                ver_uses_rs,
                ver_to_max_data_file_size};

#[derive(Clone, Debug)]
pub struct Stats {
    uid                         : [u8; SBX_FILE_UID_LEN],
    version                     : Version,
    hash_bytes                  : Option<multihash::HashBytes>,
    pub meta_blocks_written     : u32,
    pub data_blocks_written     : u32,
    pub data_par_blocks_written : u32,
    pub data_padding_bytes      : usize,
    pub in_file_size            : u64,
    pub out_file_size           : u64,
    total_data_blocks           : u32,
    start_time                  : f64,
    end_time                    : f64,
    json_printer                : Arc<JSONPrinter>,
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let rs_enabled              = ver_uses_rs(self.version);
        let block_size              = ver_to_block_size(self.version);
        let data_size               = ver_to_data_size(self.version);
        let meta_blocks_written     = self.meta_blocks_written;
        let data_blocks_written     = self.data_blocks_written;
        let data_par_blocks_written = self.data_par_blocks_written;
        let blocks_written          =
            meta_blocks_written
            + data_blocks_written
            + data_par_blocks_written;
        let data_bytes_encoded      =
            self.data_blocks_written as usize
            * data_size
            - self.data_padding_bytes;
        let in_file_size            = self.in_file_size;
        let out_file_size           = self.out_file_size;
        let time_elapsed            = (self.end_time - self.start_time) as i64;
        let (hour, minute, second)  = time_utils::seconds_to_hms(time_elapsed);

        let json_printer = &self.json_printer;

        json_printer.write_open_bracket(f, Some("stats"), BracketType::Curly)?;

        if rs_enabled {
            write_maybe_json!(f, json_printer, "File UID                                   : {}",
                              misc_utils::bytes_to_upper_hex_string(&self.uid))?;
            write_maybe_json!(f, json_printer, "SBX version                                : {}",
                              ver_to_usize(self.version))?;
            write_maybe_json!(f, json_printer, "Block size used in encoding                : {}", block_size              => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Data  size used in encoding                : {}", data_size               => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks written                   : {}", blocks_written          => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks written (metadata)        : {}", meta_blocks_written     => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks written (data only)       : {}", data_blocks_written     => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks written (data parity)     : {}", data_par_blocks_written => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Amount of data encoded (bytes)             : {}", data_bytes_encoded      => skip_quotes)?;
            write_maybe_json!(f, json_printer, "File size                                  : {}", in_file_size            => skip_quotes)?;
            write_maybe_json!(f, json_printer, "SBX container size                         : {}", out_file_size           => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Hash                                       : {}", match self.hash_bytes {
                None        => "N/A".to_string(),
                Some(ref h) => format!("{} - {}",
                                       multihash::hash_type_to_string(h.0),
                                       misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
            write_maybe_json!(f, json_printer, "Time elapsed                               : {:02}:{:02}:{:02}", hour, minute, second)?;
        } else {
            write_maybe_json!(f, json_printer, "File UID                            : {}",
                              misc_utils::bytes_to_upper_hex_string(&self.uid))?;
            write_maybe_json!(f, json_printer, "SBX version                         : {}", ver_to_usize(self.version))?;
            write_maybe_json!(f, json_printer, "Block size used in encoding         : {}", block_size          => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Data  size used in encoding         : {}", data_size           => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks written            : {}", blocks_written      => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks written (metadata) : {}", meta_blocks_written => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Number of blocks written (data)     : {}", data_blocks_written => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Amount of data encoded (bytes)      : {}", data_bytes_encoded  => skip_quotes)?;
            write_maybe_json!(f, json_printer, "File size                           : {}", in_file_size        => skip_quotes)?;
            write_maybe_json!(f, json_printer, "SBX container size                  : {}", out_file_size       => skip_quotes)?;
            write_maybe_json!(f, json_printer, "Hash                                : {}", match self.hash_bytes {
                None    => "N/A".to_string(),
                Some(ref h) => format!("{} - {}",
                                       multihash::hash_type_to_string(h.0),
                                       misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
            write_maybe_json!(f, json_printer, "Time elapsed                        : {:02}:{:02}:{:02}", hour, minute, second)?;
        }

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    version            : Version,
    uid                : [u8; SBX_FILE_UID_LEN],
    data_par_burst     : Option<(usize, usize, usize)>,
    rs_enabled         : bool,
    meta_enabled       : bool,
    json_printer       : Arc<JSONPrinter>,
    hash_type          : multihash::HashType,
    in_file            : String,
    out_file           : String,
    pr_verbosity_level : PRVerbosityLevel,
}

impl Param {
    pub fn new(version            : Version,
               uid                : &[u8; SBX_FILE_UID_LEN],
               data_par_burst     : Option<(usize, usize, usize)>,
               meta_enabled       : bool,
               json_printer       : &Arc<JSONPrinter>,
               hash_type          : multihash::HashType,
               in_file            : &str,
               out_file           : &str,
               pr_verbosity_level : PRVerbosityLevel) -> Param {
        Param {
            version,
            uid            : uid.clone(),
            data_par_burst,
            rs_enabled     : ver_uses_rs(version),
            meta_enabled   : ver_forces_meta_enabled(version) || meta_enabled,
            json_printer   : Arc::clone(json_printer),
            hash_type,
            in_file        : String::from(in_file),
            out_file       : String::from(out_file),
            pr_verbosity_level,
        }
    }
}

impl Stats {
    pub fn new(param : &Param, file_size : u64) -> Stats {
        use file_utils::from_orig_file_size::calc_data_chunk_count;
        let total_data_blocks =
            calc_data_chunk_count(param.version, file_size) as u32;
        Stats {
            uid                     : param.uid,
            version                 : param.version,
            hash_bytes              : None,
            meta_blocks_written     : 0,
            data_blocks_written     : 0,
            data_par_blocks_written : 0,
            data_padding_bytes      : 0,
            total_data_blocks,
            in_file_size            : 0,
            out_file_size           : 0,
            start_time              : 0.,
            end_time                : 0.,
            json_printer            : Arc::clone(&param.json_printer),
        }
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      { self.data_blocks_written as u64 }

    fn total_units(&self)        -> u64      { self.total_data_blocks as u64 }
}

fn pack_metadata(block         : &mut Block,
                 param         : &Param,
                 stats         : &Stats,
                 file_metadata : &fs::Metadata,
                 file_size     : u64,
                 hash          : Option<multihash::HashBytes>) {
    block.set_seq_num(0);

    let meta = block.meta_mut().unwrap();

    { // add file name
        let file_name = file_utils::get_file_name_part_of_path(&param.in_file);
        meta.push(Metadata::FNM(file_name)); }
    { // add sbx file name
        let file_name = file_utils::get_file_name_part_of_path(&param.out_file);
        meta.push(Metadata::SNM(file_name)); }
    { // add file size
        meta.push(Metadata::FSZ(file_size)); }
    { // add file last modifcation time
        match file_metadata.modified() {
            Ok(t)  => match t.duration_since(UNIX_EPOCH) {
                Ok(t)  => meta.push(Metadata::FDT(t.as_secs() as i64)),
                Err(_) => {}
            },
            Err(_) => {} }}
    { // add sbx encoding time
        meta.push(Metadata::SDT(stats.start_time as i64)); }
    { // add hash
        let hsh = match hash {
            Some(hsh) => hsh,
            None      => {
                let ctx = multihash::hash::Ctx::new(param.hash_type).unwrap();
                ctx.finish_into_hash_bytes()
            }
        };
        meta.push(Metadata::HSH(hsh)); }
    { // add RS params
        if param.rs_enabled {
            meta.push(Metadata::RSD(param.data_par_burst.unwrap().0 as u8));
            meta.push(Metadata::RSP(param.data_par_burst.unwrap().1 as u8)); }}
}

fn write_meta_blocks(param         : &Param,
                     stats         : &Arc<Mutex<Stats>>,
                     file_metadata : &fs::Metadata,
                     file_size     : u64,
                     hash          : Option<multihash::HashBytes>,
                     block         : &mut Block,
                     buffer        : &mut [u8],
                     writer        : &mut FileWriter,
                     record_stats  : bool)
                     -> Result<(), Error> {
    // pack metadata into the block
    pack_metadata(block,
                  param,
                  &stats.lock().unwrap(),
                  file_metadata,
                  file_size,
                  hash);

    match block.sync_to_buffer(None, buffer) {
        Ok(()) => {},
        Err(sbx_block::Error::TooMuchMetadata(ref m)) => {
            return Err(Error::with_message(
                &make_too_much_meta_err_string(block.get_version(), m)));
        },
        Err(_) => unreachable!(),
    }

    let write_pos_s =
        sbx_block::calc_meta_block_all_write_pos_s(param.version,
                                                   param.data_par_burst);

    for &p in write_pos_s.iter() {
        writer.seek(SeekFrom::Start(p))?;

        writer.write(sbx_block::slice_buf(block.get_version(), buffer))?;

        if record_stats {
            stats.lock().unwrap().meta_blocks_written += 1;
        }
    }

    block.add1_seq_num().unwrap();

    Ok(())
}

fn write_data_block(param  : &Param,
                    block  : &mut Block,
                    buffer : &mut [u8],
                    writer : &mut FileWriter)
                    -> Result<(), Error> {
    let write_pos =
        calc_data_block_write_pos(param.version,
                                  block.get_seq_num(),
                                  Some(param.meta_enabled),
                                  param.data_par_burst);

    block_sync_and_write(block,
                         buffer,
                         writer,
                         write_pos)
}

fn block_sync_and_write(block  : &mut Block,
                        buffer : &mut [u8],
                        writer : &mut FileWriter,
                        pos    : u64)
                        -> Result<(), Error> {
    match block.sync_to_buffer(None, buffer) {
        Ok(()) => {},
        Err(sbx_block::Error::TooMuchMetadata(ref m)) => {
            return Err(Error::with_message(
                &make_too_much_meta_err_string(block.get_version(), m)));
        },
        Err(_) => unreachable!(),
    }

    writer.seek(SeekFrom::Start(pos))?;

    writer.write(sbx_block::slice_buf(block.get_version(), buffer))?;

    match block.add1_seq_num() {
        Ok(_)  => Ok(()),
        Err(_) => Err(Error::with_message("Block seq num already at max, addition causes overflow. This might be due to file size has changed during the encoding"))
    }
}

pub fn encode_file(param : &Param)
                   -> Result<Stats, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    // setup file reader and writer
    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;
    let mut writer = FileWriter::new(&param.out_file,
                                     FileWriterParam { read     : false,
                                                       append   : false,
                                                       buffered : true   })?;

    { // check if in file size exceeds maximum
        let in_file_size     = reader.get_file_size()?;
        let max_in_file_size = ver_to_max_data_file_size(param.version);

        if in_file_size > max_in_file_size {
            return Err(Error::with_message(&format!("File size of \"{}\" exceeds the maximum supported file size, size : {}, max : {}",
                                                    &param.in_file,
                                                    in_file_size,
                                                    max_in_file_size)));
        }
    }

    let metadata = reader.metadata()?;

    let file_size = reader.get_file_size()?;

    // setup stats
    let stats = Arc::new(Mutex::new(Stats::new(param, file_size)));

    // setup reporter
    let reporter = ProgressReporter::new(&stats,
                                         "Data encoding progress",
                                         "chunks",
                                         param.pr_verbosity_level);

    // set up hash state
    let mut hash_ctx =
        multihash::hash::Ctx::new(param.hash_type).unwrap();

    // setup Reed-Solomon things
    let mut rs_codec =
        match param.data_par_burst {
            None                    => None,
            Some((data, parity, _)) => Some(RSEncoder::new(param.version,
                                                           data,
                                                           parity))
        };

    // setup main data buffer
    let mut data : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    // setup padding block
    let mut padding : [u8; SBX_LARGEST_BLOCK_SIZE] = [0x1A; SBX_LARGEST_BLOCK_SIZE];

    // setup main data block
    let mut block = Block::new(param.version,
                               &param.uid,
                               BlockType::Data);

    reporter.start();

    if param.meta_enabled { // write dummy metadata block
        write_meta_blocks(param,
                          &stats,
                          &metadata,
                          file_size,
                          None,
                          &mut block,
                          &mut data,
                          &mut writer,
                          true)?;
    }

    loop {
        break_if_atomic_bool!(ctrlc_stop_flag);

        // read data in
        let read_res =
            reader.read(sbx_block::slice_data_buf_mut(param.version, &mut data))?;

        if read_res.len_read == 0 { break; }

        let mut data_blocks_written     = 0;
        let mut data_par_blocks_written = 0;

        stats.lock().unwrap().data_padding_bytes +=
            sbx_block::write_padding(param.version, read_res.len_read, &mut data);

        // start encoding
        write_data_block(param,
                         &mut block,
                         &mut data,
                         &mut writer)?;

        data_blocks_written += 1;

        // update hash state if needed
        if param.meta_enabled {
            let data_part = &sbx_block::slice_data_buf(param.version, &data)[0..read_res.len_read];
            hash_ctx.update(data_part);
        }

        // update Reed-Solomon data if needed
        if let Some(ref mut rs_codec) = rs_codec {
            // encode normally once
            if let Some(parity_to_use) = rs_codec.encode_no_block_sync(&data) {
                for p in parity_to_use.iter_mut() {
                    write_data_block(param,
                                     &mut block,
                                     p,
                                     &mut writer)?;

                    data_par_blocks_written += 1;
                }
            }
        }

        // update stats
        stats.lock().unwrap().data_blocks_written     += data_blocks_written;
        stats.lock().unwrap().data_par_blocks_written += data_par_blocks_written;
    }

    if let Some(ref mut rs_codec) = rs_codec {
        // fill remaining slots with padding if required
        if rs_codec.active() {
            let slots_to_fill = rs_codec.unfilled_slot_count();
            for i in 0..slots_to_fill {
                // write padding
                write_data_block(param,
                                 &mut block,
                                 &mut padding,
                                 &mut writer)?;

                stats.lock().unwrap().data_blocks_written += 1;

                if let Some(parity_to_use) =
                    rs_codec.encode_no_block_sync(&padding)
                {
                    // this should only be executed at the last iteration
                    assert_eq!(i, slots_to_fill - 1);

                    for p in parity_to_use.iter_mut() {
                        write_data_block(param,
                                         &mut block,
                                         p,
                                         &mut writer)?;

                        stats.lock().unwrap().data_par_blocks_written += 1;
                    }
                }
            }
        }
    }

    if param.meta_enabled {
        let hash_bytes = hash_ctx.finish_into_hash_bytes();

        // write actual medata blocks
        write_meta_blocks(param,
                          &stats,
                          &metadata,
                          file_size,
                          Some(hash_bytes.clone()),
                          &mut block,
                          &mut data,
                          &mut writer,
                          false)?;

        // record hash in stats
        stats.lock().unwrap().hash_bytes = Some(hash_bytes.clone());
    }

    reporter.stop();

    stats.lock().unwrap().in_file_size  = reader.get_file_size()?;
    stats.lock().unwrap().out_file_size = writer.get_file_size()?;

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
