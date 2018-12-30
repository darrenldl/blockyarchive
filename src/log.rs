#![allow(dead_code)]
use file_reader::{FileReader,
                  FileReaderParam};
use file_writer::{FileWriter,
                  FileWriterParam};
use general_error::Error;
use std::fmt;

use std::time::Duration;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Barrier;

use std::sync::Arc;
use std::sync::Mutex;

use std::thread;

const LOG_MAX_SIZE : usize = 1024;

const LOG_WRITE_INTERVAL_IN_MILLISEC : u64 = 1000;

pub struct LogHandler<T : 'static + Log + Send> {
    start_barrier    : Arc<Barrier>,
    start_flag       : Arc<AtomicBool>,
    shutdown_flag    : Arc<AtomicBool>,
    shutdown_barrier : Arc<Barrier>,
    log_file         : Option<String>,
    stats            : Arc<Mutex<T>>,
    error            : Arc<Mutex<Option<Error>>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ErrorKind {
    ParseError,
}

#[derive(Clone)]
pub struct LogError {
    kind : ErrorKind,
    path : String,
}

impl fmt::Display for LogError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;
        match self.kind {
            ParseError => write!(f, "failed to parse log file \"{}\"", self.path),
        }
    }
}

impl LogError {
    pub fn new(kind : ErrorKind, path : &str) -> LogError {
        LogError {
            kind,
            path : String::from(path),
        }
    }
}

pub fn to_err(e : LogError) -> super::Error {
    use super::{Error, ErrorKind};
    Error::new(ErrorKind::LogError(e))
}

pub trait Log {
    fn serialize(&self) -> String;

    fn deserialize(&mut self, &[u8]) -> Result<(), ()>;

    fn read_from_file(&mut self, log_file : &str) -> Result<(), Error> {
        let mut reader = FileReader::new(log_file,
                                         FileReaderParam { write    : false,
                                                           buffered : false  })?;
        let mut buffer : [u8; LOG_MAX_SIZE] = [0; LOG_MAX_SIZE];
        let _len_read = reader.read(&mut buffer)?;

        match self.deserialize(&buffer) {
            Ok(())  => Ok(()),
            Err(()) => Err(to_err(LogError::new(ErrorKind::ParseError, log_file))),
        }
    }

    fn write_to_file(&self, log_file : &str) -> Result<(), Error> {
        let mut writer = FileWriter::new(log_file,
                                         FileWriterParam { read     : false,
                                                           append   : false,
                                                           truncate : true,
                                                           buffered : false  })?;
        let output = self.serialize();

        writer.write(output.as_bytes())?;

        Ok(())
    }
}

impl<T : 'static + Log + Send> LogHandler<T> {
    pub fn new(log_file : Option<&str>,
               stats    : &Arc<Mutex<T>>) -> LogHandler<T> {
        let log_file = match log_file {
            None         => None,
            Some(ref lg) => Some(lg.to_string()),
        };
        let stats                   = Arc::clone(stats);
        let error                   = Arc::new(Mutex::new(None));
        let start_barrier           = Arc::new(Barrier::new(2));
        let start_flag              = Arc::new(AtomicBool::new(false));
        let shutdown_flag           = Arc::new(AtomicBool::new(false));
        let shutdown_barrier        = Arc::new(Barrier::new(2));
        let runner_start_barrier    = Arc::clone(&start_barrier);
        let runner_shutdown_flag    = Arc::clone(&shutdown_flag);
        let runner_shutdown_barrier = Arc::clone(&shutdown_barrier);
        let runner_log_file         = log_file.clone();
        let runner_stats            = Arc::clone(&stats);
        let runner_error            = Arc::clone(&error);
        thread::spawn(move || {
            loop {
                // wait to be kickstarted
                runner_start_barrier.wait();

                while !runner_shutdown_flag.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(LOG_WRITE_INTERVAL_IN_MILLISEC));

                    match Self::write_to_file(&runner_log_file,
                                              &runner_stats.lock().unwrap()) {
                        Ok(()) => {},
                        Err(e) => *runner_error.lock().unwrap() = Some(e),
                    }
                }

                runner_shutdown_barrier.wait();
            }
        });
        LogHandler {
            start_barrier,
            start_flag,
            shutdown_flag,
            shutdown_barrier,
            log_file,
            stats,
            error,
        }
    }

    pub fn start(&self) {
        if !self.start_flag.swap(true, Ordering::SeqCst) {
            self.start_barrier.wait();
        }
    }

    pub fn stop(&self) {
        if self.start_flag.load(Ordering::SeqCst)
            && !self.shutdown_flag.load(Ordering::SeqCst)
        {
            self.shutdown_flag.store(true, Ordering::SeqCst);

            self.shutdown_barrier.wait();
        }
    }

    pub fn pop_error(&self) -> Result<(), Error>{
        let mut error = self.error.lock().unwrap();

        let res =
            match *error {
                None        => Ok(()),
                Some(ref e) => Err(e.clone()),
            };

        *error = None;

        res
    }

    pub fn read_from_file(&self) -> Result<(), Error> {
        use super::ErrorKind;
        use super::file_error;

        match self.log_file {
            None         => Ok(()),
            Some(ref lg) => {
                let res = self.stats.lock().unwrap().read_from_file(lg);

                if let Err(ref e) = res {
                    if let ErrorKind::FileError(ref fe) = e.kind {
                        if let file_error::ErrorKind::NotFound = fe.kind {
                            return Ok(());
                        }
                    }
                }
                res
            }
        }
    }

    fn write_to_file(log_file : &Option<String>, stats : &T) -> Result<(), Error> {
        match log_file {
            &None        => {},
            &Some(ref x) => stats.write_to_file(x)?,
        }

        Ok(())
    }
}

impl<T : 'static + Log + Send> Drop for LogHandler<T> {
    fn drop(&mut self) {
        self.stop();
    }
}
