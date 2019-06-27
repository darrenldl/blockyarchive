use std::fmt;
pub use std::io::ErrorKind;

#[derive(Clone, Debug)]
pub struct StdoutError {
    pub kind: ErrorKind,
}

impl StdoutError {
    pub fn new(kind: ErrorKind) -> StdoutError {
        StdoutError { kind }
    }
}

pub fn to_err(e: StdoutError) -> super::Error {
    use super::{Error, ErrorKind};
    Error::new(ErrorKind::StdoutError(e))
}

impl fmt::Display for StdoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;
        match self.kind {
            NotFound => panic!("Invalid error"),
            PermissionDenied => panic!("Invalid error"),
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
            TimedOut => write!(f, "stdout operation timed out"),
            WriteZero => panic!("Invalid error"),
            Interrupted => write!(f, "stdout operation interrupted"),
            Other => write!(f, "stdout unknown error"),
            UnexpectedEof => write!(f, "stdout unexpected EOF"),
            _ => write!(f, "Unknown error"),
        }
    }
}
