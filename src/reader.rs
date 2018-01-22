use super::FileError;
use std::io::Read;
use std::fs::File;

const READ_RETRIES : usize = 5;

pub struct Reader {
    file : File,
}

impl Reader {
    pub fn new(path : String) -> Result<Reader, FileError> {
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => { return Err(FileError::new(e.kind(), path)); }
        };
        Ok (Reader {
            file
        })
    }
}
