use super::FileError;
use std::io::Read;
use std::fs::File;

const READ_RETRIES : usize = 5;

pub struct Reader {
    file : File,
    path : String,
}

impl Reader {
    pub fn new(path : &str) -> Result<Reader, FileError> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => { return Err(FileError::new(e.kind(), path)); }
        };
        Ok (Reader {
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
}
