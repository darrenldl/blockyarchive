use super::FileError;
use std::io::Read;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Seek;

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
        match self.file.read(buf) {
            Ok(len_read) => Ok(len_read),
            Err(e)       => Err(FileError::new(e.kind(), &self.path))
        }
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
}
