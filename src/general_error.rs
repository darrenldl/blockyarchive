use super::file_error;
use super::rs_codec;
use std::fmt;

#[derive(Clone)]
pub enum ErrorKind {
    RSError(rs_codec::RSError),
    FileError(file_error::FileError),
}

#[derive(Clone)]
pub struct Error {
    kind : ErrorKind
}

impl Error {
    pub fn new(kind : ErrorKind) -> Error {
        Error {
            kind
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;
        match self.kind {
            RSError(ref e)   => writeln!(f, "FEC codec error : {}", e),
            FileError(ref e) => writeln!(f, "File error : {}", e),
        }
    }
}
