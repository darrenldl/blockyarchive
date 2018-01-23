use std::io::ErrorKind;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct FileError {
    kind : ErrorKind,
    path : String,
}

impl FileError {
    pub fn new(kind : ErrorKind, path : &str) -> FileError {
        FileError {
            kind,
            path : String::from(path)
        }
    }
}

use std::error::Error;

impl fmt::Display for FileError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;
        match self.kind {
            NotFound          => write!(f, "file \"{}\" not found", self.path),
            PermissionDenied  => write!(f, "file \"{}\" permission denied", self.path),
            ConnectionRefused => panic!("Invalid error"),
            ConnectionReset   => panic!("Invalid error"),
            ConnectionAborted => panic!("Invalid error"),
            NotConnected      => panic!("Invalid error"),
            AddrInUse         => panic!("Invalid error"),
            AddrNotAvailable  => panic!("Invalid error"),
            BrokenPipe        => panic!("Invalid error"),
            AlreadyExists     => panic!("Invalid error"),
            WouldBlock        => panic!("Invalid error"),
            InvalidInput      => panic!("Invalid parameters"),
            InvalidData       => panic!("Invalid parameters"),
            TimedOut          => write!(f, "file \"{}\" operation timed out", &self.path),
            WriteZero         => write!(f, "file \"{}\" failed write", &self.path),
            Interrupted       => write!(f, "file \"{}\" operation interrupted", &self.path),
            Other             => write!(f, "file \"{}\" unknown error", &self.path),
            UnexpectedEof     => write!(f, "file \"{}\" unexpected EOF", &self.path),
            _                 => write!(f, "Unknown error")
        }
    }
}
