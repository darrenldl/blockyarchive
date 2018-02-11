use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use super::time_utils;
use std::io::SeekFrom;

use progress_report::ProgressReport;
use progress_report::ProgressReporter;

use super::progress_report;

use std::time::UNIX_EPOCH;

use super::file_reader::FileReader;
use super::file_writer::FileWriter;

use super::multihash;

use super::Error;
use super::sbx_specs::Version;
use super::rs_codec::RSEncoder;

use super::sbx_block::{Block, BlockType};
use super::sbx_block;
use super::sbx_block::Metadata;
use super::sbx_specs::SBX_FILE_UID_LEN;
use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_specs::SBX_RS_METADATA_PARITY_COUNT;
use super::sbx_specs::ver_forces_meta_enabled;
use super::sbx_specs::{ver_to_block_size,
                       ver_to_data_size,
                       ver_supports_rs};

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    version                     : Version,
    pub meta_blocks_written     : u32,
    pub meta_par_blocks_written : u32,
    pub data_blocks_written     : u32,
    pub data_par_blocks_written : u32,
    total_data_blocks           : u32,
    start_time                  : f64,
    end_time                    : f64,
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let rs_enabled              = ver_supports_rs(self.version);
        let block_size              = ver_to_block_size(self.version);
        let data_size               = ver_to_data_size(self.version);
        let meta_blocks_written     = self.meta_blocks_written;
        let data_blocks_written     = self.data_blocks_written;
        let meta_par_blocks_written = self.meta_par_blocks_written;
        let data_par_blocks_written = self.data_par_blocks_written;
        let blocks_written = meta_blocks_written
            + data_blocks_written
            + meta_par_blocks_written
            + data_par_blocks_written;
        let data_bytes_encoded      = self.data_blocks_written * data_size as u32;
        let time_elapsed            = (self.end_time - self.start_time) as i64;
        let (hour, minute, second)  = time_utils::seconds_to_hms(time_elapsed);

        if rs_enabled {
            writeln!(f, "Block size used in encoding                : {}", block_size)?;
            writeln!(f, "Data  size used in encoding                : {}", data_size)?;
            writeln!(f, "Number of blocks written                   : {}", blocks_written)?;
            writeln!(f, "Number of blocks written (metadata)        : {}", meta_blocks_written)?;
            writeln!(f, "Number of blocks written (metadata parity) : {}", meta_par_blocks_written)?;
            writeln!(f, "Number of blocks written (data)            : {}", data_blocks_written)?;
            writeln!(f, "Number of blocks written (data parity)     : {}", data_par_blocks_written)?;
            writeln!(f, "Amount of data encoded (bytes)             : {}", data_bytes_encoded)?;
            writeln!(f, "Time elapsed                               : {:02}:{:02}:{:02}", hour, minute, second)
        } else {
            writeln!(f, "Block size used in encoding         : {}", block_size)?;
            writeln!(f, "Data  size used in encoding         : {}", data_size)?;
            writeln!(f, "Number of blocks written            : {}", blocks_written)?;
            writeln!(f, "Number of blocks written (metadata) : {}", meta_blocks_written)?;
            writeln!(f, "Number of blocks written (data)     : {}", data_blocks_written)?;
            writeln!(f, "Amount of data encoded (bytes)      : {}", data_bytes_encoded)?;
            writeln!(f, "Time elapsed                        : {:02}:{:02}:{:02}", hour, minute, second)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    version      : Version,
    file_uid     : [u8; SBX_FILE_UID_LEN],
    rs_data      : usize,
    rs_parity    : usize,
    rs_enabled   : bool,
    meta_enabled : bool,
    hash_type    : multihash::HashType,
    in_file      : String,
    out_file     : String,
    silence_level : progress_report::SilenceLevel
}

impl Param {
    pub fn new(version       : Version,
               file_uid      : &[u8; SBX_FILE_UID_LEN],
               rs_data       : usize,
               rs_parity     : usize,
               meta_enabled  : bool,
               hash_type     : multihash::HashType,
               in_file       : &str,
               out_file      : &str,
               silence_level : progress_report::SilenceLevel) -> Param {
        Param {
            version,
            file_uid : file_uid.clone(),
            rs_data,
            rs_parity,
            rs_enabled : ver_supports_rs(version),
            meta_enabled : ver_forces_meta_enabled(version) || meta_enabled,
            hash_type,
            in_file  : String::from(in_file),
            out_file : String::from(out_file),
            silence_level,
        }
    }
}

impl Stats {
    pub fn new(param : &Param, file_metadata : &fs::Metadata) -> Stats {
        let total_data_blocks =
            file_utils::calc_data_chunk_count(param.version, file_metadata) as u32;
        Stats {
            version                 : param.version,
            meta_blocks_written     : 0,
            data_blocks_written     : 0,
            meta_par_blocks_written : 0,
            data_par_blocks_written : 0,
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
    let meta = block.meta_mut().unwrap();

    { // add file name
        meta.push(Metadata::FNM(param
                                .in_file
                                .clone()
                                .into_bytes()
                                .into_boxed_slice())); }
    { // add sbx file name
        meta.push(Metadata::SNM(param
                                .out_file
                                .clone()
                                .into_bytes()
                                .into_boxed_slice())); }
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
            meta.push(Metadata::RSD(param.rs_data   as u8));
            meta.push(Metadata::RSP(param.rs_parity as u8)); }}
}

fn write_metadata_block(param         : &Param,
                        stats         : &Stats,
                        file_metadata : &fs::Metadata,
                        hash          : Option<multihash::HashBytes>,
                        buf           : &mut [u8]) {
    let mut block = Block::new(param.version,
                               &param.file_uid,
                               BlockType::Meta);
    pack_metadata(&mut block,
                  param,
                  stats,
                  file_metadata,
                  hash);
    block.sync_to_buffer(None, buf).unwrap();
}

pub fn encode_file(param : &Param)
                   -> Result<Stats, Error> {
    let metadata = file_utils::get_file_metadata(&param.in_file)?;

    // setup stats
    let stats = Arc::new(Mutex::new(Stats::new(param, &metadata)));

    // setup reporter
    let mut reporter = ProgressReporter::new(&stats,
                                             "Data encoding progress",
                                             "chunks",
                                             param.silence_level);

    // setup file reader and writer
    let mut reader = FileReader::new(&param.in_file)?;
    let mut writer = FileWriter::new(&param.out_file)?;

    // set up hash state
    let mut hash_ctx =
        multihash::hash::Ctx::new(param.hash_type).unwrap();

    // setup Reed-Solomon things
    let mut rs_codec_meta = RSEncoder::new(param.version,
                                           1,
                                           SBX_RS_METADATA_PARITY_COUNT,
                                           1);
    let mut rs_codec_data = RSEncoder::new(param.version,
                                           param.rs_data,
                                           param.rs_parity,
                                           file_utils::calc_data_chunk_count(param.version,
                                                                             &metadata));

    // setup main data buffer
    let mut data : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    // setup main data block
    let mut block = Block::new(param.version,
                               &param.file_uid,
                               BlockType::Data);

    reporter.start();

    if param.meta_enabled { // write dummy metadata block
        write_metadata_block(param,
                             &stats.lock().unwrap(),
                             &metadata,
                             None,
                             &mut data);
        writer.write(sbx_block::slice_buf(param.version,
                                          &data))?;

        block.add1_seq_num();

        stats.lock().unwrap().meta_blocks_written += 1;

        if param.rs_enabled {
            let parity_to_use = rs_codec_meta.encode(&data).unwrap();

            for p in parity_to_use.iter_mut() {
                block.sync_to_buffer(None, p).unwrap();

                // write data out
                writer.write(sbx_block::slice_buf(param.version, p))?;

                block.add1_seq_num();
            }
        }

        stats.lock().unwrap().meta_par_blocks_written += SBX_RS_METADATA_PARITY_COUNT as u32;
    }

    loop {
        let mut data_blocks_written     = 0;
        let mut data_par_blocks_written = 0;

        // read data in
        let len_read =
            reader.read(sbx_block::slice_data_buf_mut(param.version, &mut data))?;

        if len_read == 0 {
            break;
        }

        sbx_block::write_padding(param.version, len_read, &mut data);

        // start encoding
        block.sync_to_buffer(None, &mut data).unwrap();

        block.add1_seq_num();
        data_blocks_written += 1;

        // write data out
        writer.write(sbx_block::slice_buf(param.version, &mut data))?;

        // update hash state if needed
        if param.meta_enabled {
            let data_part = &sbx_block::slice_data_buf(param.version, &data)[0..len_read];
            hash_ctx.update(data_part);
        }

        // update Reed-Solomon data if needed
        if param.rs_enabled {
            if let Some(parity_to_use) = rs_codec_data.encode(&data) {
                for p in parity_to_use.iter_mut() {
                    block.sync_to_buffer(None, p).unwrap();

                    // write data out
                    writer.write(sbx_block::slice_buf(param.version, p))?;

                    block.add1_seq_num();
                    data_par_blocks_written += 1;
                }
            }
        }

        // update stats
        stats.lock().unwrap().data_blocks_written     += data_blocks_written;
        stats.lock().unwrap().data_par_blocks_written += data_par_blocks_written;
    }

    if param.meta_enabled { // write actual metadata block
        block.set_seq_num(0);

        write_metadata_block(param,
                             &stats.lock().unwrap(),
                             &metadata,
                             Some(hash_ctx.finish_into_hash_bytes()),
                             &mut data);

        writer.seek(SeekFrom::Start(0))?;

        writer.write(sbx_block::slice_buf(param.version, &data))?;

        block.add1_seq_num();

        if param.rs_enabled {
            let parity_to_use = rs_codec_meta.encode(&data).unwrap();

            for p in parity_to_use.iter_mut() {
                block.sync_to_buffer(None, p).unwrap();

                // write data out
                writer.write(sbx_block::slice_buf(param.version, p))?;

                block.add1_seq_num();
            }
        }
    }

    reporter.stop();

    let stats = stats.lock().unwrap().clone();
    Ok(stats)
}
