use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use super::file_error;
use super::worker::reader;

use super::multihash;

use super::scoped_threadpool::Pool;

use std::cell::Cell;

use std::time::Duration;

use std::sync::mpsc::RecvTimeoutError;

use super::misc_utils::{make_channel_for_ctx,
                        make_sync_channel_for_ctx};

use super::Error;
use super::FileWriter;
use super::sbx_specs::Version;
use super::time;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender,
                      SyncSender,
                      Receiver};

use super::sbx_block::{Block, BlockType};
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       SBX_HEADER_SIZE,
                       ver_to_block_size};

type SharedStats = Arc<Mutex<Stats>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub version             : Version,
    pub meta_blocks_written : u64,
    pub data_blocks_written : u64,
    pub data_bytes_encoded  : u64,
    pub total_bytes         : u64,
    pub start_time          : i64,
    pub data_shards         : usize,
    pub parity_shards       : usize
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub version      : Version,
    pub file_uid     : [u8; SBX_FILE_UID_LEN],
    pub rs_enabled   : bool,
    pub rs_data      : usize,
    pub rs_parity    : usize,
    pub hash_enabled : bool,
    pub hash_type    : multihash::HashType,
    pub in_file      : String,
    pub out_file     : String
}

pub struct Context {
    pub shutdown      : Arc<AtomicBool>,
    pub err_collect   : (Sender<Option<Error>>,
                         Cell<Option<Receiver<Option<Error>>>>),
    pub data_block    : Block,
    pub parity_blocks : Vec<Block>,
    pub ingress_bytes : (SyncSender<Box<[u8]>>,
                         Cell<Option<Receiver<Box<[u8]>>>>),
    pub egress_bytes  : (SyncSender<Box<[u8]>>,
                         Cell<Option<Receiver<Box<[u8]>>>>)
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
            total_bytes         : 0,
            start_time          : time::get_time().sec,
            data_shards         : 0,
            parity_shards       : 0,
        }
    }

    pub fn time_elapsed(&self) -> i64 {
        time::get_time().sec - self.start_time
    }
}

fn make_reader(param   : &Param,
               stats   : &SharedStats,
               context : &mut Context,
               counter : &Arc<Mutex<u64>>)
               -> Result<JoinHandle<()>, Error> {
    reader::make_reader(ver_to_block_size(param.version),
                        Some(SBX_HEADER_SIZE),
                        None,
                        counter,
                        &context.shutdown,
                        &param.in_file,
                        context.ingress_bytes.0.clone(),
                        context.err_collect.0.clone())
}

fn make_packer(param   : &Param,
               stats   : &SharedStats,
               context : &mut Context)
               -> Result<JoinHandle<()>, Error> {
    let stats         = Arc::clone(stats);
    let rx_bytes      = context.ingress_bytes.1.replace(None).unwrap();
    let tx_bytes      = context.egress_bytes.0.clone();
    let tx_error      = context.err_collect.0.clone();
    let shutdown_flag = Arc::clone(&context.shutdown);
    let param         = param.clone();

    Ok(thread::spawn(move || {
        let mut thread_pool       = Pool::new(2);
        let mut cur_seq_num : u64 = 1;
        let mut hash_ctx          =
            multihash::hash::Ctx::new(param.hash_type).unwrap();
        loop {
            worker_stop!(graceful_if_shutdown => tx_error, shutdown_flag);

            let mut buf = match rx_bytes.recv_timeout(Duration::from_millis(10)) {
                Ok(buf)                             => buf,
                Err(RecvTimeoutError::Timeout)      => { continue; },
                Err(RecvTimeoutError::Disconnected) => { panic!(); }
            };

            // start packing
            let mut block = Block::new(param.version,
                                       &param.file_uid,
                                       BlockType::Data);
            block.header.seq_num = 1;

            thread_pool.scoped(|scope| {
                // update CRC
                scope.execute(|| {
                    block.update_crc(&buf).unwrap();
                });
                // update hash state
                if param.hash_enabled {
                    scope.execute(|| {
                        hash_ctx.update(&buf);
                    });
                }
            });

            block.sync_to_buffer(Some(false), &mut buf).unwrap();

            tx_bytes.send(buf).unwrap();

            // update stats
            cur_seq_num += 1;
            stats.lock().unwrap().data_blocks_written = cur_seq_num;
        }
    }))
}

fn make_writer(param   : &Param,
               stats   : &SharedStats,
               context : &mut Context) -> Result<JoinHandle<()>, Error> {
    let mut writer    = file_error::adapt_to_err(FileWriter::new(&param.out_file))?;
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
                Err(e) => { worker_stop!(with_error file_error::to_err(e) =>
                                         tx_error, shutdown_flag); }
            }
        }
    }))
}

pub fn encode_file(param    : &Param)
                   -> Result<Stats, Error> {
    let stats : SharedStats =
        Arc::new(Mutex::new(Stats::new(param)));
    let read_byte_counter = Arc::new(Mutex::new(0u64));

    let mut ctx = Context::new(param);

    let reader = make_reader(param, &stats, &mut ctx, &read_byte_counter)?;
    let packer = make_packer(param, &stats, &mut ctx)?;
    let writer = make_writer(param, &stats, &mut ctx)?;

    reader.join().unwrap();
    packer.join().unwrap();
    writer.join().unwrap();

    Ok(Stats::new(param))
}
