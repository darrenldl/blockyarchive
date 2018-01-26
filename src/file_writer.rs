use super::Error;
use super::file_error::FileError;
use super::file_error::to_err;
use std::io::Write;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Seek;

pub struct FileWriter {
    file : File,
    path : String,
}

impl FileWriter {
    pub fn new(path : &str) -> Result<FileWriter, Error> {
        let file = match File::create(&path) {
            Ok(f) => f,
            Err(e) => { return Err(to_err(FileError::new(e.kind(), path))); }
        };
        Ok (FileWriter {
            file,
            path : String::from(path)
        })
    }

    pub fn write(&mut self, buf : &[u8]) -> Result<usize, Error> {
        match self.file.write(buf) {
            Ok(len_wrote) => Ok(len_wrote),
            Err(e)        => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
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
}
