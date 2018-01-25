use super::super::Error;
use super::super::file_error::adapt_to_err;
use super::super::FileWriter;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;

use std::io::SeekFrom;

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
    let read_start = match read_start {
        Some(x) => x,
        None    => 0
    };

    let counter       = Arc::clone(counter);
    let shutdown_flag = Arc::clone(shutdown_flag);
    let writer_res    = adapt_to_err(FileWriter::new(out_file));

    Ok(thread::spawn(move || {
        let mut writer = match writer_res {
            Ok(w)  => w,
            Err(e) => worker_stop!(with_error_ret e => tx_error, shutdown_flag)
        };

        loop {
            worker_stop!(graceful_if_shutdown => tx_error, shutdown_flag);

            let req = recv!(timeout_millis 10 => rx_write_req, tx_error, shutdown_flag);

            match req {
                WriteReq::Seek(pos)         => {
                    worker_stop!(with_error_if_fail
                                 (adapt_to_err(
                                     writer.seek(SeekFrom::Start(pos)))) =>
                                 tx_error, shutdown_flag);
                },
                WriteReq::WriteTo(tar_pos, buf) => {
                    let read_end_exc = match read_end_exc {
                        Some(x) => x,
                        None    => buf.len()
                    };
                    let cur_pos = worker_stop!(with_error_if_fail
                                               (adapt_to_err(writer.cur_pos())) =>
                                               tx_error, shutdown_flag);
                    worker_stop!(with_error_if_fail
                                 (adapt_to_err(
                                     writer.seek(SeekFrom::Start(tar_pos)))) =>
                                 tx_error, shutdown_flag);
                    worker_stop!(with_error_if_fail
                                 (adapt_to_err(
                                     writer.write(&buf[read_start..read_end_exc]))) =>
                                 tx_error, shutdown_flag);
                    worker_stop!(with_error_if_fail
                                 (adapt_to_err(
                                     writer.seek(SeekFrom::Start(cur_pos)))) =>
                                 tx_error, shutdown_flag);

                    *counter.lock().unwrap() +=
                        buf[read_start..read_end_exc].len() as u64;
                },
                WriteReq::Write(buf) => {
                    let read_end_exc = match read_end_exc {
                        Some(x) => x,
                        None    => buf.len()
                    };
                    worker_stop!(with_error_if_fail
                                 (adapt_to_err(
                                     writer.write(&buf[read_start..read_end_exc]))) =>
                                 tx_error, shutdown_flag);

                    *counter.lock().unwrap() +=
                        buf[read_start..read_end_exc].len() as u64;
                }
            }
        }
    }))
}
