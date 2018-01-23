use super::FileError;
use std::io::Write;
use std::fs::File;

pub struct Writer {
    file : File,
    path : String,
}

impl Writer {
    pub fn new(path : String) -> Result<Writer, FileError> {
        let file = match File::create(&path) {
            Ok(f) => f,
            Err(e) => { return Err(FileError::new(e.kind(), path)); }
        };
        Ok (Writer {
            file,
            path
        })
    }
}
