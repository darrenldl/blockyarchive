use super::file_error;
use super::rs_codec;
use super::log;
use std::fmt;

#[derive(Clone)]
pub enum ErrorKind {
    RSError(rs_codec::RSError),
    FileError(file_error::FileError),
    LogError(log::LogError),
    MessageOnly(String)
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

    pub fn with_message(msg : &str) -> Error {
        Error {
            kind : ErrorKind::MessageOnly(String::from(msg))
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;
        match self.kind {
            RSError(ref e)     => writeln!(f, "FEC codec error : {}", e),
            FileError(ref e)   => writeln!(f, "File error : {}", e),
            LogError(ref e)    => writeln!(f, "Log error : {}", e),
            MessageOnly(ref e) => writeln!(f, "Error : {}", e),
        }
    }
}
