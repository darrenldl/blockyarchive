use super::super::Error;
use super::super::file_error;
use super::super::FileReader;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::TrySendError;
use std::time::Duration;
use std::thread;
use std::thread::JoinHandle;

pub enum WriteReq {
    Seek(u64),
    WriteTo(u64, Box<[u8]>),
    Write(Box<[u8]>)
}

pub fn make_writer(read_start    : Option<usize>,
                   read_end_exc  : Option<usize>,
                   counter       : &Arc<Mutex<u64>>,
                   shutdown_flag : &Arc<AtomicBool>,
                   out_file      : &str,
                   rx_write_req  : Receiver<WriteReq>,
                   tx_error      : Sender<Option<Error>>)
                   -> Result<JoinHandle<()>, Error> {
    Ok(thread::spawn(move || {
        
    }))
}
