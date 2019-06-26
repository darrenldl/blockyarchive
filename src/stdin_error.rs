use std::fmt;
pub use std::io::ErrorKind;

#[derive(Clone, Debug)]
pub struct StdinError {
    pub kind: ErrorKind,
}

impl StdinError {
    pub fn new(kind: ErrorKind) -> StdinError {
        StdinError { kind }
    }
}

pub fn to_err(e: StdinError) -> super::Error {
    use super::{Error, ErrorKind};
    Error::new(ErrorKind::StdinError(e))
}

impl fmt::Display for StdinError {
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
            TimedOut => write!(f, "stdin operation timed out"),
            WriteZero => panic!("Invalid error"),
            Interrupted => write!(f, "stdin operation interrupted"),
            Other => write!(f, "stdin unknown error"),
            UnexpectedEof => write!(f, "stdin unexpected EOF"),
            _ => write!(f, "Unknown error"),
        }
    }
}
