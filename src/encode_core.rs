use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use super::worker::reader;
use super::worker::writer;
use super::worker::writer::WriteReq;
use std::fs;
use std::fmt;

use super::SmallVec;

use std::time::UNIX_EPOCH;

use super::file_reader;
use super::file_error::adapt_to_err;

use super::multihash;

use super::pond::Pool;

use std::cell::Cell;

use super::misc_utils::{make_channel_for_ctx,
                        make_sync_channel_for_ctx};

use super::Error;
use super::sbx_specs::Version;
use super::time;
use super::ReedSolomon;

use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Sender,
                      SyncSender,
                      Receiver};

use super::sbx_block::{Block, BlockType};
use super::sbx_block;
use super::sbx_block::metadata;
use super::sbx_block::metadata::Metadata;
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
    pub total_bytes         : u64,
    pub start_time          : i64,
    pub time_elapsed        : i64,
    pub data_shards         : usize,
    pub parity_shards       : usize
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
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
    pub ingress_bytes : (SyncSender<Option<(usize, Box<[u8]>)>>,
                         Cell<Option<Receiver<Option<(usize, Box<[u8]>)>>>>),
    pub egress_bytes  : (SyncSender<Option<WriteReq>>,
                         Cell<Option<Receiver<Option<WriteReq>>>>),
    pub file_metadata : fs::Metadata
}

impl Context {
    pub fn new(param         : &Param,
               file_metadata : fs::Metadata)
               -> Context {
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
            file_metadata
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
            time_elapsed        : 0,
            data_shards         : 0,
            parity_shards       : 0,
        }
    }

    pub fn set_time_elapsed(&mut self) {
        self.time_elapsed = time::get_time().sec - self.start_time;
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

fn pack_metadata(block         : &mut Block,
                 param         : &Param,
                 stats         : &Stats,
                 file_metadata : fs::Metadata,
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
        meta.push(Metadata::SDT(stats.start_time)); }
    { // add hash
        if param.hash_enabled {
            let hsh = match hash {
                Some(hsh) => hsh,
                None      => {
                    let ctx = multihash::hash::Ctx::new(param.hash_type).unwrap();
                    ctx.finish_into_hash_bytes()
                }
            };
            meta.push(Metadata::HSH(hsh)); }}
}

fn make_packer(param   : &Param,
               stats   : &SharedStats,
               context : &Context)
               -> Result<JoinHandle<()>, Error> {
    let stats         = Arc::clone(stats);
    let rx_bytes      = context.ingress_bytes.1.replace(None).unwrap();
    let tx_bytes      = context.egress_bytes.0.clone();
    let tx_error      = context.err_collect.0.clone();
    let shutdown_flag = Arc::clone(&context.shutdown);
    let param         = param.clone();
    let data_size     = ver_to_data_size(param.version);
    let block_size    = ver_to_block_size(param.version);
    let file_metadata = context.file_metadata.clone();

    Ok(thread::spawn(move || {
        let mut block             = Block::new(param.version,
                                               &param.file_uid,
                                               BlockType::Data);
        let mut thread_pool       = Pool::new(2);
        let mut cur_seq_num : u64 = 1;
        let mut hash_ctx          =
            multihash::hash::Ctx::new(param.hash_type).unwrap();
        let rs_codec              = ReedSolomon::new(param.rs_data,
                                                     param.rs_parity).unwrap();

        let mut parity : Vec<Box<[u8]>> = Vec::with_capacity(param.rs_parity);
        if param.rs_enabled {
            for _ in 0..param.rs_parity {
                parity.push(vec![0; data_size].into_boxed_slice());
            }
        }
        let mut partiy_refs : SmallVec<[&mut [u8]; 32]> =
            convert_2D_slices!(parity =to_mut=> SmallVec<[&mut [u8]; 32]>,
                               SmallVec::with_capacity);

        {
            let mut block = Block::new(param.version,
                                       &param.file_uid,
                                       BlockType::Meta);
            // write dummy metadata block
            pack_metadata(&mut block,
                          &param,
                          &stats.lock().unwrap(),
                          file_metadata.clone(),
                          None);
            let mut buf = vec![0; block_size].into_boxed_slice();
            block.sync_to_buffer(None, &mut buf).unwrap();
            send!(no_back_off_ret Some(WriteReq::Write(buf)) =>
                  tx_bytes, tx_error, shutdown_flag);
        }

        loop {
            let rs_data_index =
                (cur_seq_num - 1) as usize % param.rs_data;

            let (len_read, mut buf) =
                recv!(no_timeout_shutdown_if_none =>
                      rx_bytes, tx_error, shutdown_flag);

            // start packing
            block.header.seq_num = cur_seq_num as u32;

            thread_pool.scoped(|scope| {
                // update CRC
                scope.execute(|| {
                    block.update_crc(&buf).unwrap();
                });
                // update hash state
                scope.execute(|| {
                    if param.hash_enabled {
                        let data_buf = &buf[SBX_HEADER_SIZE..
                                            SBX_HEADER_SIZE + len_read];
                        hash_ctx.update(data_buf);
                        println!("Updated hash");
                    }
                });
                // update rs parity
                scope.execute(|| {
                    if param.rs_enabled {
                        rs_codec.encode_single_sep(rs_data_index,
                                                   sbx_block::slice_data_buf(param.version,
                                                                             &buf),
                                                   &mut partiy_refs).unwrap();
                    }
                });
            });

            block.sync_to_buffer(Some(false), &mut buf).unwrap();

            send!(back_off Some(WriteReq::Write(buf)) =>
                  tx_bytes, tx_error, shutdown_flag);

            if param.rs_enabled && rs_data_index == param.rs_parity - 1 {
                // output parity blocks
                for p in partiy_refs.iter() {
                    let mut buf = vec![0; block_size].into_boxed_slice();
                    buf[SBX_HEADER_SIZE..].copy_from_slice(p);

                    block.header.seq_num = cur_seq_num as u32;
                    block.sync_to_buffer(None, &mut buf).unwrap();

                    send!(back_off Some(WriteReq::Write(buf)) =>
                          tx_bytes, tx_error, shutdown_flag);

                    cur_seq_num += 1;
                    stats.lock().unwrap().data_blocks_written = cur_seq_num;
                }
            }

            // update stats
            cur_seq_num += 1;
            stats.lock().unwrap().data_blocks_written = cur_seq_num;
        }

        {
            println!("Writing actual metadata block");
            // write actual metadata block
            let mut block = Block::new(param.version,
                                       &param.file_uid,
                                       BlockType::Meta);
            pack_metadata(&mut block,
                          &param,
                          &stats.lock().unwrap(),
                          file_metadata,
                          Some(hash_ctx.finish_into_hash_bytes()));
            let mut buf = vec![0; block_size].into_boxed_slice();
            block.sync_to_buffer(None, &mut buf).unwrap();
            send!(no_back_off_ret Some(WriteReq::WriteTo(0, buf)) =>
                  tx_bytes, tx_error, shutdown_flag);
        }

        worker_stop!(graceful_ret =>
                     tx_error, shutdown_flag [tx_bytes]);
    }))
}

fn make_writer(param   : &Param,
               stats   : &SharedStats,
               context : &mut Context,
               counter : &Arc<Mutex<u64>>)
               -> Result<JoinHandle<()>, Error> {
    writer::make_writer(None,
                        None,
                        counter,
                        &context.shutdown,
                        &param.out_file,
                        context.egress_bytes.1.replace(None).unwrap(),
                        context.err_collect.0.clone())
}

pub fn encode_file(param    : &Param)
                   -> Result<Stats, Error> {
    let metadata = {
        let reader = adapt_to_err(file_reader::FileReader::new(&param.in_file))?;

        adapt_to_err(reader.metadata())?
    };

    let mut ctx = Context::new(param, metadata);

    let stats : SharedStats =
        Arc::new(Mutex::new(Stats::new(param)));

    let read_byte_counter  = Arc::new(Mutex::new(0u64));
    let write_byte_counter = Arc::new(Mutex::new(0u64));

    {
        let reader = make_reader(param, &stats, &mut ctx, &read_byte_counter).unwrap();
        let packer = make_packer(param, &stats, &mut ctx).unwrap();
        let writer = make_writer(param, &stats, &mut ctx, &write_byte_counter).unwrap();

        reader.join().unwrap();
        packer.join().unwrap();
        writer.join().unwrap();
    }

    let rx_error : Receiver<Option<Error>> =
        ctx.err_collect.1.replace(None).unwrap();
    let mut ret_error : Option<Error> = None;
    for _ in 0..3 {
        match rx_error.recv().unwrap() {
            None    => {},
            Some(e) => { ret_error = Some(e); break; }
        }
    }

    stats.lock().unwrap().set_time_elapsed();

    let bytes_read  : &u64 = &read_byte_counter.lock().unwrap();

    stats.lock().unwrap().data_bytes_encoded = *bytes_read;

    match ret_error {
        Some(e) => Err(e),
        None    => { let stats = stats.lock().unwrap().clone();
                     Ok(stats) }
    }
}
