use std::io::ErrorKind;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct FileError {
    kind : ErrorKind,
    path : String,
}

impl FileError {
    pub fn new(kind : ErrorKind, path : String) -> FileError {
        FileError {
            kind,
            path
        }
    }
}

use std::error::Error;

impl fmt::Display for FileError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;
        match self.kind {
            NotFound => write!(f, " file : {} not found", self.path),
            _        => write!(f, "Unknown error")
        }
    }
}
