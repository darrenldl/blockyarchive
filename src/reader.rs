use file_reader::FileReader;
use general_error::Error;
use std::io::Read;
use std::fs::Metadata;
use stdin_error::{StdinError,
                  to_err};

const READ_RETRIES : usize = 5;

pub struct ReadResult {
    pub len_read : usize,
    pub eof_seen : bool,
}

pub enum ReaderType {
    File(FileReader),
    Stdin(std::io::Stdin),
}

pub struct Reader {
    reader : ReaderType,
}

impl Reader {
    pub fn new(reader : ReaderType) -> Reader {
        Reader {
            reader
        }
    }

    pub fn read(&mut self, buf : &mut [u8]) -> Result<ReadResult, Error> {
        match self.reader {
            ReaderType::File(ref mut f)  => f.read(buf),
            ReaderType::Stdin(ref mut s) => {
                let mut len_read = 0;
                let mut tries    = 0;
                while len_read < buf.len() && tries < READ_RETRIES {
                    match s.read(&mut buf[len_read..]) {
                        Ok(len) => { len_read += len; },
                        Err(e)  => { return Err(to_err(StdinError::new(e.kind()))); }
                    }

                    tries += 1;
                }

                Ok(ReadResult { len_read,
                                eof_seen : len_read < buf.len() })
            }
        }
    }

    pub fn metadata(&self) -> Option<Result<Metadata, Error>> {
        match self.reader {
            ReaderType::File(f)  => Some(f.metadata()),
            ReaderType::Stdin(_) => None,
        }
    }

    pub fn get_file_size(&mut self) -> Option<Result<u64, Error>> {
        match self.reader {
            ReaderType::File(ref mut f)  => Some(f.get_file_size()),
            ReaderType::Stdin(_)         => None,
        }
    }
}
