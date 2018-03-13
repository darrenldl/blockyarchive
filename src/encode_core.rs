use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use super::time_utils;
use super::misc_utils;
use std::io::SeekFrom;

use super::progress_report::*;

use std::time::UNIX_EPOCH;
use super::cli_utils::setup_ctrlc_handler;

use super::file_reader::FileReader;
use super::file_reader::FileReaderParam;
use super::file_writer::FileWriter;
use super::file_writer::FileWriterParam;

use super::multihash;

use super::Error;
use super::sbx_specs::Version;
use super::rs_codec::RSEncoder;

use super::sbx_block::{Block, BlockType};
use super::sbx_block;
use super::sbx_block::Metadata;
use super::sbx_specs::SBX_FILE_UID_LEN;
use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_specs::SBX_FIRST_DATA_SEQ_NUM;
use super::sbx_specs::ver_forces_meta_enabled;
use super::sbx_specs::{ver_to_usize,
                       ver_to_block_size,
                       ver_to_data_size,
                       ver_uses_rs,
                       ver_to_max_data_file_size};
use super::sbx_block::{calc_data_block_write_pos,
                       calc_meta_block_dup_write_pos_s};

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    version                     : Version,
    hash_bytes                  : Option<multihash::HashBytes>,
    pub meta_blocks_written     : u32,
    pub data_blocks_written     : u32,
    pub data_par_blocks_written : u32,
    pub data_padding_bytes      : usize,
    total_data_blocks           : u32,
    start_time                  : f64,
    end_time                    : f64,
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
        let time_elapsed            = (self.end_time - self.start_time) as i64;
        let (hour, minute, second)  = time_utils::seconds_to_hms(time_elapsed);

        if rs_enabled {
            writeln!(f, "SBX version                                : {} (0x{:X})",
                     ver_to_usize(self.version),
                     ver_to_usize(self.version))?;
            writeln!(f, "Block size used in encoding                : {}", block_size)?;
            writeln!(f, "Data  size used in encoding                : {}", data_size)?;
            writeln!(f, "Number of blocks written                   : {}", blocks_written)?;
            writeln!(f, "Number of blocks written (metadata)        : {}", meta_blocks_written)?;
            writeln!(f, "Number of blocks written (data only)       : {}", data_blocks_written)?;
            writeln!(f, "Number of blocks written (data parity)     : {}", data_par_blocks_written)?;
            writeln!(f, "Amount of data encoded (bytes)             : {}", data_bytes_encoded)?;
            writeln!(f, "Hash                                       : {}", match self.hash_bytes {
                None        => "N/A".to_string(),
                Some(ref h) => format!("{} - {}",
                                       multihash::hash_type_to_string(h.0),
                                       misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
            writeln!(f, "Time elapsed                               : {:02}:{:02}:{:02}", hour, minute, second)
        } else {
            writeln!(f, "SBX version                         : {}", ver_to_usize(self.version))?;
            writeln!(f, "Block size used in encoding         : {}", block_size)?;
            writeln!(f, "Data  size used in encoding         : {}", data_size)?;
            writeln!(f, "Number of blocks written            : {}", blocks_written)?;
            writeln!(f, "Number of blocks written (metadata) : {}", meta_blocks_written)?;
            writeln!(f, "Number of blocks written (data)     : {}", data_blocks_written)?;
            writeln!(f, "Amount of data encoded (bytes)      : {}", data_bytes_encoded)?;
            writeln!(f, "Hash                                : {}", match self.hash_bytes {
                None    => "N/A".to_string(),
                Some(ref h) => format!("{} - {}",
                                       multihash::hash_type_to_string(h.0),
                                       misc_utils::bytes_to_lower_hex_string(&h.1))
            })?;
            writeln!(f, "Time elapsed                        : {:02}:{:02}:{:02}", hour, minute, second)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    version            : Version,
    uid                : [u8; SBX_FILE_UID_LEN],
    data_par_burst     : Option<(usize, usize, usize)>,
    rs_enabled         : bool,
    meta_enabled       : bool,
    hash_type          : multihash::HashType,
    in_file            : String,
    out_file           : String,
    pr_verbosity_level : PRVerbosityLevel,
}

impl Param {
    pub fn new(version            : Version,
               uid                : &[u8; SBX_FILE_UID_LEN],
               data_par_burst     : Option<(usize, usize, usize)>,
               no_meta            : bool,
               hash_type          : multihash::HashType,
               in_file            : &str,
               out_file           : &str,
               pr_verbosity_level : PRVerbosityLevel) -> Param {
        Param {
            version,
            uid            : uid.clone(),
            data_par_burst,
            rs_enabled     : ver_uses_rs(version),
            meta_enabled   : ver_forces_meta_enabled(version) || !no_meta,
            hash_type,
            in_file        : String::from(in_file),
            out_file       : String::from(out_file),
            pr_verbosity_level,
        }
    }
}

impl Stats {
    pub fn new(param : &Param, file_metadata : &fs::Metadata) -> Stats {
        use file_utils::from_raw_file_metadata::calc_data_chunk_count;
        let total_data_blocks =
            calc_data_chunk_count(param.version, file_metadata) as u32;
        Stats {
            version                 : param.version,
            hash_bytes              : None,
            meta_blocks_written     : 0,
            data_blocks_written     : 0,
            data_par_blocks_written : 0,
            data_padding_bytes      : 0,
            total_data_blocks,
            start_time              : 0.,
            end_time                : 0.,
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
                 hash          : Option<multihash::HashBytes>) {
    block.set_seq_num(0);

    let meta = block.meta_mut().unwrap();

    { // add file name
        meta.push(Metadata::FNM(param
                                .in_file
                                .clone())); }
    { // add sbx file name
        meta.push(Metadata::SNM(param
                                .out_file
                                .clone())); }
    { // add file size
        meta.push(Metadata::FSZ(file_metadata
                                .len())); }
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
                     hash          : Option<multihash::HashBytes>,
                     block         : &mut Block,
                     buf           : &mut [u8],
                     writer        : &mut FileWriter)
                     -> Result<(), Error> {
    // pack metadata into the block
    pack_metadata(block,
                  param,
                  &stats.lock().unwrap(),
                  file_metadata,
                  hash);

    let write_pos_s =
        sbx_block::calc_meta_block_all_write_pos_s(param.version,
                                                   param.data_par_burst);

    for &p in write_pos_s.iter() {
        block.set_seq_num(0);

        block_sync_and_write(block,
                             buf,
                             writer,
                             p)?;

        stats.lock().unwrap().meta_blocks_written += 1;
    }

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
        Err(_) => { return Err(Error::with_message("Too much metadata")); }
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
    let ctrlc_stop_flag = setup_ctrlc_handler();

    // setup file reader and writer
    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;
    let mut writer = FileWriter::new(&param.out_file,
                                     FileWriterParam { read     : false,
                                                       append   : false,
                                                       buffered : true   })?;

    { // check if in file size exceeds maximum
        let in_file_size     = reader.metadata()?.len();
        let max_in_file_size = ver_to_max_data_file_size(param.version);

        if in_file_size > max_in_file_size {
            return Err(Error::with_message(&format!("File size of \"{}\" exceeds the maximum supported file size, size : {}, max : {}",
                                                    &param.in_file,
                                                    in_file_size,
                                                    max_in_file_size)));
        }
    }

    let metadata = file_utils::get_file_metadata(&param.in_file)?;

    // setup stats
    let stats = Arc::new(Mutex::new(Stats::new(param, &metadata)));

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
                          None,
                          &mut block,
                          &mut data,
                          &mut writer)?;
    }

    loop {
        break_if_atomic_bool!(ctrlc_stop_flag);

        let mut data_blocks_written     = 0;
        let mut data_par_blocks_written = 0;

        // read data in
        let read_res =
            reader.read(sbx_block::slice_data_buf_mut(param.version, &mut data))?;

        if read_res.len_read == 0 {
            if let Some(ref mut rs_codec) = rs_codec {
                // check if the current batch of RS blocks are filled
                if (block.get_seq_num() - SBX_FIRST_DATA_SEQ_NUM)
                    % rs_codec.total_shard_count() as u32 != 0
                {
                    // fill remaining slots with padding
                    loop {
                        // write padding
                        write_data_block(param,
                                         &mut block,
                                         &mut padding,
                                         &mut writer)?;

                        data_blocks_written += 1;

                        // keep writing until parity shards are ready
                        // then break loop
                        if let Some(parity_to_use) =
                            rs_codec.encode_no_block_sync(&padding)
                        {
                            for p in parity_to_use.iter_mut() {
                                write_data_block(param,
                                                 &mut block,
                                                 p,
                                                 &mut writer)?;

                                data_par_blocks_written += 1;
                            }

                            break;
                        }
                    }
                }
            }

            break; // break outer loop
        }

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

    if param.meta_enabled {
        let hash_bytes = hash_ctx.finish_into_hash_bytes();

        // write actual medata blocks
        write_meta_blocks(param,
                          &stats,
                          &metadata,
                          Some(hash_bytes.clone()),
                          &mut block,
                          &mut data,
                          &mut writer)?;

        // record hash in stats
        stats.lock().unwrap().hash_bytes = Some(hash_bytes.clone());
    }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();
    Ok(stats)
}
