use std::fmt;
pub use std::io::ErrorKind;

#[derive(Clone, Debug)]
pub struct FileError {
    pub kind: ErrorKind,
    path: String,
}

impl FileError {
    pub fn new(kind: ErrorKind, path: &str) -> FileError {
        FileError {
            kind,
            path: String::from(path),
        }
    }
}

pub fn to_err(e: FileError) -> super::Error {
    use super::{Error, ErrorKind};
    Error::new(ErrorKind::FileError(e))
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;
        match self.kind {
            NotFound => write!(f, "File \"{}\" not found", self.path),
            PermissionDenied => write!(f, "File \"{}\" permission denied", self.path),
            ConnectionRefused => panic!("Invalid error"),
            ConnectionReset => panic!("Invalid error"),
            ConnectionAborted => panic!("Invalid error"),
            NotConnected => panic!("Invalid error"),
            AddrInUse => panic!("Invalid error"),
            AddrNotAvailable => panic!("Invalid error"),
            BrokenPipe => panic!("Invalid error"),
            AlreadyExists => panic!("Invalid error"),
            WouldBlock => panic!("Invalid error"),
            InvalidInput => panic!("Invalid parameters"),
            InvalidData => panic!("Invalid parameters"),
            TimedOut => write!(f, "File \"{}\" operation timed out", &self.path),
            WriteZero => write!(f, "File \"{}\" failed write", &self.path),
            Interrupted => write!(f, "File \"{}\" operation interrupted", &self.path),
            Other => write!(f, "File \"{}\" unknown error", &self.path),
            UnexpectedEof => write!(f, "File \"{}\" unexpected EOF", &self.path),
            _ => write!(f, "Unknown error"),
        }
    }
}
