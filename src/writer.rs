#![allow(dead_code)]
use crate::file_writer::FileWriter;
use crate::general_error::Error;
use crate::stdout_error::{to_err, StdoutError};
use std::fs::Metadata;
use std::io::SeekFrom;
use std::io::Write;
use crate::reader::ReadResult;

pub enum WriterType {
    File(FileWriter),
    Stdout(std::io::Stdout),
}

pub struct Writer {
    writer: WriterType,
}

impl Writer {
    pub fn new(writer: WriterType) -> Writer {
        Writer { writer }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        match self.writer {
            WriterType::File(ref mut f) => f.write(buf),
            WriterType::Stdout(ref mut s) => match s.write(buf) {
                Ok(len) => Ok(len),
                Err(e) => Err(to_err(StdoutError::new(e.kind()))),
            },
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Option<Result<ReadResult, Error>> {
        match self.writer {
            WriterType::File(ref mut f) => Some(f.read(buf)),
            WriterType::Stdout(_) => None,
        }
    }

    pub fn set_len(&mut self, size: u64) -> Option<Result<(), Error>> {
        match self.writer {
            WriterType::File(ref mut f) => Some(f.set_len(size)),
            WriterType::Stdout(_) => None,
        }
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Option<Result<u64, Error>> {
        match self.writer {
            WriterType::File(ref mut f) => Some(f.seek(pos)),
            WriterType::Stdout(_) => None,
        }
    }

    pub fn cur_pos(&mut self) -> Option<Result<u64, Error>> {
        match self.writer {
            WriterType::File(ref mut f) => Some(f.cur_pos()),
            WriterType::Stdout(_) => None,
        }
    }

    pub fn metadata(&self) -> Option<Result<Metadata, Error>> {
        match self.writer {
            WriterType::File(ref f) => Some(f.metadata()),
            WriterType::Stdout(_) => None,
        }
    }

    pub fn get_file_size(&mut self) -> Option<Result<u64, Error>> {
        match self.writer {
            WriterType::File(ref mut f) => Some(f.get_file_size()),
            WriterType::Stdout(_) => None,
        }
    }
}
