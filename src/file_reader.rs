use super::Error;
use super::file_error::FileError;
use super::file_error::to_err;
use std::io::Read;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Seek;
use std::fs::Metadata;

const READ_RETRIES : usize = 5;

pub struct FileReader {
    file : File,
    path : String,
}

impl FileReader {
    pub fn new(path : &str) -> Result<FileReader, Error> {
        let file = match File::open(path) {
            Ok(f)  => f,
            Err(e) => { return Err(to_err(FileError::new(e.kind(), path))); }
        };
        Ok (FileReader {
            file,
            path : String::from(path)
        })
    }

    pub fn read(&mut self, buf : &mut [u8]) -> Result<usize, Error> {
        let mut len_read = 0;
        let mut tries    = 0;
        while len_read < buf.len() && tries < READ_RETRIES {
            match self.file.read(&mut buf[len_read..]) {
                Ok(len) => { len_read += len; },
                Err(e)  => { return Err(to_err(FileError::new(e.kind(), &self.path))); }
            }

            tries += 1;
        }

        Ok(len_read)
    }

    pub fn seek(&mut self, pos : SeekFrom)
                -> Result<u64, Error> {
        match self.file.seek(pos) {
            Ok(pos) => Ok(pos),
            Err(e)  => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
    }

    pub fn cur_pos(&mut self) -> Result<u64, Error> {
        match self.file.seek(SeekFrom::Current(0)) {
            Ok(pos) => Ok(pos),
            Err(e)  => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
    }

    pub fn metadata(&self) -> Result<Metadata, Error> {
        match self.file.metadata() {
            Ok(data) => Ok(data),
            Err(e)   => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
    }
}
