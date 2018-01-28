use super::file_error;
use super::rs_codec;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    RSError(rs_codec::Error),
    FileError(file_error::FileError),
}

#[derive(Clone, Debug, PartialEq)]
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
            RSCodecError(ref e) => write!(f, "RS codec error : {}", e),
            FileError(ref e)    => write!(f, "File error : {}", e),
        }
    }
}
