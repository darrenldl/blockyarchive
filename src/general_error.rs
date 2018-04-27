use file_error;
use log;
use std::fmt;

#[derive(Clone)]
pub enum ErrorKind {
    FileError(file_error::FileError),
    LogError(log::LogError),
    MessageOnly(String)
}

#[derive(Clone)]
pub struct Error {
    pub kind : ErrorKind
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
            FileError(ref e)   => write!(f, "File error : {}", e),
            LogError(ref e)    => write!(f, "Log error : {}", e),
            MessageOnly(ref e) => write!(f, "Error : {}", e),
        }
    }
}
