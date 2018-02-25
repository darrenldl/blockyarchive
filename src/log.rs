use super::file_reader::FileReader;
use super::file_reader::FileReaderParam;
use super::file_writer::FileWriter;
use super::file_writer::FileWriterParam;
use super::Error;
use std::fmt;

use std::time::Duration;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use super::time_utils;

use std::sync::Arc;
use std::sync::Mutex;

use std::thread;

const LOG_MAX_SIZE : usize = 1024;

const LOG_WRITE_INTERVAL_IN_MILLISEC : u64 = 1000;

pub struct LogHandler<T : 'static + Log + Send> {
    log_file   : String,
    stats      : Arc<Mutex<T>>,
    runner     : JoinHandle<()>,
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
            ParseError => writeln!(f, "failed to parse log file \"{}\"", self.path),
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
                                                           buffered : false  })?;
        let output = self.serialize();

        let _len_written = writer.write(output.as_bytes())?;

        Ok(())
    }
}

impl<T : 'static + Log + Send> LogHandler<T> {
    pub fn new(log_file : &str,
               stats    : &Arc<Mutex<T>>) -> LogHandler<T> {
        let write_flag = Arc::new(AtomicBool::new(true));
        let runner_write_flag = Arc::clone(&write_flag);
        let runner = thread::spawn(move || {
            loop {
                runner_write_flag.store(true, Ordering::Relaxed);

                thread::sleep(Duration::from_millis(LOG_WRITE_INTERVAL_IN_MILLISEC));
            }
        });
        LogHandler {
            log_file : log_file.to_string(),
            stats    : Arc::clone(stats),
            runner,
            write_flag,
        }
    }

    pub fn read_from_file(&self) -> Result<(), Error> {
        self.stats.lock().unwrap().read_from_file(&self.log_file)
    }

    pub fn write_to_file(&mut self, force_write : bool) -> Result<(), Error> {
        if force_write || self.write_flag.load(Ordering::Relaxed) {
            self.write_flag.store(false, Ordering::Relaxed);

            self.stats.lock().unwrap().write_to_file(&self.log_file)?;
        }

        Ok(())
    }
}
