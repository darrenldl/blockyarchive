use file_reader::{FileReader,
                  FileReaderParam};
use file_writer::{FileWriter,
                  FileWriterParam};
use general_error::Error;
use std::fmt;

use std::time::Duration;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use std::sync::Arc;
use std::sync::Mutex;

use std::thread;

const LOG_MAX_SIZE : usize = 1024;

const LOG_WRITE_INTERVAL_IN_MILLISEC : u64 = 1000;

pub struct LogHandler<T : 'static + Log + Send> {
    log_file   : Option<String>,
    stats      : Arc<Mutex<T>>,
    write_flag : Arc<AtomicBool>,
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

        let _len_written = writer.write(output.as_bytes())?;

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
        let write_flag = Arc::new(AtomicBool::new(true));
        let runner_write_flag = Arc::clone(&write_flag);
        thread::spawn(move || {
            loop {
                runner_write_flag.store(true, Ordering::Relaxed);

                thread::sleep(Duration::from_millis(LOG_WRITE_INTERVAL_IN_MILLISEC));
            }
        });
        LogHandler {
            log_file,
            stats    : Arc::clone(stats),
            write_flag,
        }
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

    pub fn write_to_file(&self, force_write : bool) -> Result<(), Error> {
        if force_write || self.write_flag.swap(false, Ordering::SeqCst) {
            match self.log_file {
                None        => {},
                Some(ref x) => self.stats.lock().unwrap().write_to_file(x)?,
            }
        }

        Ok(())
    }
}
