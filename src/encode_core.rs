use std::fs::File;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use super::file_error;

use super::{Error, ErrorKind};
use super::Reader;
use super::Writer;
use super::sbx_specs;
use super::sbx_specs::Version;
use super::time;

use super::multihash;

use std::sync::mpsc::{channel,
                      sync_channel,
                      Sender,
                      SyncSender,
                      Receiver};

use super::sbx_block::{Block, BlockType};
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       ver_to_block_size,
                       ver_to_data_size};

type SharedStats = Arc<Mutex<Stats>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub version             : Version,
    pub meta_blocks_written : u64,
    pub data_blocks_written : u64,
    pub data_bytes_encoded  : u64,
    pub start_time          : u64,
    pub data_shards         : usize,
    pub parity_shards       : usize
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub version      : Version,
    pub file_uid     : [u8; SBX_FILE_UID_LEN],
    pub rs_enabled   : bool,
    pub rs_parity    : usize,
    pub rs_data      : usize,
    pub hash_enabled : bool,
    pub hash_type    : multihash::HashType,
    pub in_file      : String,
    pub out_file     : String
}

pub struct Context {
    pub err_collect   : (Sender<Error>, Receiver<Error>),
    pub data_block    : Block,
    pub parity_blocks : Vec<Block>,
    pub ingress_bytes : (SyncSender<Box<[u8]>>, Receiver<Box<[u8]>>),
    pub egress_bytes  : (SyncSender<Box<[u8]>>, Receiver<Box<[u8]>>)
}

impl Context {
    pub fn new(param : &Param) -> Context {
        let data_block = Block::new(param.version,
                                    &param.file_uid,
                                    BlockType::Data);
        let parity_blocks = if param.rs_enabled {
            let mut vec = Vec::with_capacity(param.rs_parity);
            for _ in 0..param.rs_parity {
                vec.push(Block::new(param.version,
                                    &param.file_uid,
                                    BlockType::Data))
            };
            vec
        } else {
            Vec::new()
        };

        Context {
            err_collect : channel(),
            data_block,
            parity_blocks,
            ingress_bytes : sync_channel(100),
            egress_bytes  : sync_channel(100),
        }
    }
}

impl Stats {
    pub fn new(param : &Param) -> Stats {
        Stats {
            version             : param.version,
            meta_blocks_written : 0,
            data_blocks_written : 0,
            data_bytes_encoded  : 0,
            start_time          : time::precise_time_ns(),
            data_shards         : 0,
            parity_shards       : 0
        }
    }

    pub fn time_elapsed(&self) -> u64 {
        time::precise_time_ns() - self.start_time
    }
}

fn make_reader(param   : &Param,
               stats   : &SharedStats,
               context : &mut Context)
               -> Result<JoinHandle<()>, Error> {
    let mut reader = file_error::adapt_to_err(Reader::new(&param.in_file))?;
    let stats = Arc::clone(stats);
    let tx_bytes = context.ingress_bytes.0.clone();
    let tx_error = context.err_collect.0.clone();
    let block_size = ver_to_block_size(param.version);
    Ok(thread::spawn(move || {
        loop {
            // allocate buffer on heap
            let mut buf : Box<[u8]> = vec![0; block_size].into_boxed_slice();

            let len_read = reader.read(&mut buf);
        }
    }))
}

fn make_packer(param  : &Param,
               stats  : &SharedStats,
               buffer : Vec<Block>)
               -> Result<(), Error> {
    use self::Version::*;
    match param.version {
        V1  | V2  | V3 => {
        },
        V11 | V12 | V13 => {
        }
    }
    Ok(())
}

fn make_writer() -> Result<(), Error> {
    Ok(())
}

pub fn encode_file(param    : &Param)
                   -> Result<Stats, Error> {
    let stats : SharedStats =
        Arc::new(Mutex::new(Stats::new(param)));

    let mut ctx = Context::new(param);

    let reader = make_reader(param, &stats, &mut ctx);

    Ok(Stats::new(param))
}
