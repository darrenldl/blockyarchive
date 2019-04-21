use crate::file_error;
use crate::log;
use crate::stdin_error;
use crate::stdout_error;
use std::fmt;

#[derive(Clone)]
pub enum ErrorKind {
    FileError(file_error::FileError),
    StdinError(stdin_error::StdinError),
    StdoutError(stdout_error::StdoutError),
    LogError(log::LogError),
    MessageOnly(String),
}

#[derive(Clone)]
pub struct Error {
    pub kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }

    pub fn with_msg(msg: &str) -> Error {
        Error {
            kind: ErrorKind::MessageOnly(String::from(msg)),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;
        match self.kind {
            FileError(ref e) => write!(f, "File error : {}", e),
            StdinError(ref e) => write!(f, "Stdin error : {}", e),
            StdoutError(ref e) => write!(f, "Stdout error : {}", e),
            LogError(ref e) => write!(f, "Log error : {}", e),
            MessageOnly(ref e) => write!(f, "Error : {}", e),
        }
    }
}
