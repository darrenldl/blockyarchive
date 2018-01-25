use super::FileError;
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
    pub fn new(path : &str) -> Result<FileReader, FileError> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => { return Err(FileError::new(e.kind(), path)); }
        };
        Ok (FileReader {
            file,
            path : String::from(path)
        })
    }

    pub fn read(&mut self, buf : &mut [u8]) -> Result<usize, FileError> {
        let mut len_read = 0;
        let mut tries    = 0;
        while len_read < buf.len() && tries < READ_RETRIES {
            match self.file.read(&mut buf[len_read..]) {
                Ok(len) => { len_read += len; },
                Err(e)  => { return Err(FileError::new(e.kind(), &self.path)); }
            }

            tries += 1;
        }

        Ok(len_read)
    }

    pub fn seek(&mut self, pos : SeekFrom)
                -> Result<u64, FileError> {
        match self.file.seek(pos) {
            Ok(pos) => Ok(pos),
            Err(e)  => Err(FileError::new(e.kind(), &self.path))
        }
    }

    pub fn cur_pos(&mut self) -> Result<u64, FileError> {
        match self.file.seek(SeekFrom::Current(0)) {
            Ok(pos) => Ok(pos),
            Err(e)  => Err(FileError::new(e.kind(), &self.path))
        }
    }

    pub fn metadata(&self) -> Result<Metadata, FileError> {
        match self.file.metadata() {
            Ok(data) => Ok(data),
            Err(e)   => Err(FileError::new(e.kind(), &self.path))
        }
    }
}
