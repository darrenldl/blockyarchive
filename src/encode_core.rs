use std::fs::File;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::sync::RwLock;
use super::file_error;

use super::scoped_threadpool::Pool;

use std::cell::Cell;

use std::time::Duration;

use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::TrySendError;

use super::misc_utils::{make_channel_for_ctx,
                        make_sync_channel_for_ctx};

use super::{Error, ErrorKind};
use super::Reader;
use super::Writer;
use super::sbx_specs;
use super::sbx_specs::Version;
use super::time;

use super::multihash;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel,
                      sync_channel,
                      Sender,
                      SyncSender,
                      Receiver};

use super::sbx_block::{Block, BlockType};
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       SBX_HEADER_SIZE,
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
    pub shutdown      : Arc<AtomicBool>,
    pub err_collect   : (Sender<Error>, Cell<Option<Receiver<Error>>>),
    pub data_block    : Block,
    pub parity_blocks : Vec<Block>,
    pub ingress_bytes : (SyncSender<Box<[u8]>>, Cell<Option<Receiver<Box<[u8]>>>>),
    pub egress_bytes  : (SyncSender<Box<[u8]>>, Cell<Option<Receiver<Box<[u8]>>>>)
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
            shutdown    : Arc::new(AtomicBool::new(false)),
            err_collect : make_channel_for_ctx(),
            data_block,
            parity_blocks,
            ingress_bytes : make_sync_channel_for_ctx(100),
            egress_bytes  : make_sync_channel_for_ctx(100),
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
    let mut reader    = file_error::adapt_to_err(Reader::new(&param.in_file))?;
    let stats         = Arc::clone(stats);
    let tx_bytes      = context.ingress_bytes.0.clone();
    let tx_error      = context.err_collect.0.clone();
    let block_size    = ver_to_block_size(param.version);
    let shutdown_flag = Arc::clone(&context.shutdown);
    Ok(thread::spawn(move || {
        let mut secondary_buf : Option<Box<[u8]>> = None;

        loop {
            if shutdown_flag.load(Ordering::Relaxed) { break; }

            // allocate if secondary_buf is empty
            let mut buf = match secondary_buf {
                None    => vec![0; block_size].into_boxed_slice(),
                Some(b) => { secondary_buf = None; b }
            };

            // read into buffer
            let len_read = match reader.read(&mut buf[SBX_HEADER_SIZE..]) {
                Ok(l) => l,
                Err(e) => { tx_error.send(file_error::to_err(e));
                            break; }
            };

            if len_read == 0 {
                break;
            }

            // update stats
            stats.lock().unwrap().data_bytes_encoded += len_read as u64;

            // send bytes over
            // if full, then put current buffer into secondary buffer and wait
            match tx_bytes.try_send(buf) {
                Ok(()) => {},
                Err(TrySendError::Full(b)) => {
                    secondary_buf = Some(b);
                    thread::sleep(Duration::from_millis(10)); },
                Err(TrySendError::Disconnected(_)) => panic!()
            }
        }
    }))
}

fn make_packer(param   : &Param,
               stats   : &SharedStats,
               context : &mut Context)
               -> Result<JoinHandle<()>, Error> {
    let stats         = Arc::clone(stats);
    let rx_bytes      = context.ingress_bytes.1.replace(None).unwrap();
    let tx_bytes      = context.egress_bytes.0.clone();
    let shutdown_flag = Arc::clone(&context.shutdown);
    let version       = param.version;
    let file_uid      = param.file_uid;
    let mut thread_pool   = Pool::new(2);
    Ok(thread::spawn(move || {
        let mut cur_seq_num : u64 = 1;
        loop {
            if shutdown_flag.load(Ordering::Relaxed) { break; }

            let mut buf = match rx_bytes.recv_timeout(Duration::from_millis(10)) {
                Ok(buf)                             => buf,
                Err(RecvTimeoutError::Timeout)      => { continue; },
                Err(RecvTimeoutError::Disconnected) => { panic!(); }
            };

            // start packing
            let mut block = Block::new(version,
                                   &file_uid,
                                   BlockType::Data);
            block.header.seq_num = 1;
            {
                thread_pool.scoped(|scoped| {
                    block.calc_crc(&buf).unwrap();
                });
            }

            block.sync_to_buffer(Some(false), &mut buf);

            tx_bytes.send(buf);

            // update stats
            cur_seq_num += 1;
            stats.lock().unwrap().data_blocks_written = cur_seq_num;
        }
    }))
}

fn make_writer(param   : &Param,
               stats   : &SharedStats,
               context : &mut Context) -> Result<JoinHandle<()>, Error> {
    let mut writer    = file_error::adapt_to_err(Writer::new(&param.out_file))?;
    let stats         = Arc::clone(stats);
    let rx_bytes      = context.egress_bytes.1.replace(None).unwrap();
    let tx_error      = context.err_collect.0.clone();
    let shutdown_flag = Arc::clone(&context.shutdown);
    Ok(thread::spawn(move || {
        loop {
            if shutdown_flag.load(Ordering::Relaxed) { break; }

            let buf = match rx_bytes.recv_timeout(Duration::from_millis(10)) {
                Ok(buf)                             => buf,
                Err(RecvTimeoutError::Timeout)      => { continue; },
                Err(RecvTimeoutError::Disconnected) => { panic!(); }
            };

            match writer.write(&buf) {
                Ok(_) => {},
                Err(e) => { tx_error.send(file_error::to_err(e));
                            break; }
            }
        }
    }))
}

pub fn encode_file(param    : &Param)
                   -> Result<Stats, Error> {
    let stats : SharedStats =
        Arc::new(Mutex::new(Stats::new(param)));

    let mut ctx = Context::new(param);

    let reader = make_reader(param, &stats, &mut ctx);

    Ok(Stats::new(param))
}
