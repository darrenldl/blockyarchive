use super::Error;
use super::file_error::FileError;
use super::file_error::to_err;
use std::io::Read;
use std::io::Write;
use std::io::BufReader;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Seek;
use std::fs::Metadata;
use std::fs::OpenOptions;

const READ_RETRIES : usize = 5;

macro_rules! file_op {
    (
        $self:ident write => $input:expr
    ) => {{
        use self::FileHandle::*;
        match $self.file {
            Buffered(ref mut f)   => {
                // drop the internal buffer by seeking
                match f.seek(SeekFrom::Current(0)) {
                    Ok(_)  => {},
                    Err(e) => { return Err(to_err(FileError::new(e.kind(), &$self.path))); },
                }

                f.get_mut().write($input)
            },
            Unbuffered(ref mut f) => {
                f.write($input)
            },
        }
    }};
    (
        $self:ident $op:ident => $input:expr
    ) => {{
        use self::FileHandle::*;
        match $self.file {
            Buffered(ref mut f)   => f.$op($input),
            Unbuffered(ref mut f) => f.$op($input),
        }
    }};
    (
        $self:ident get_metadata
    ) => {{
        use self::FileHandle::*;
        match $self.file {
            Buffered(ref f)   => f.get_ref().metadata(),
            Unbuffered(ref f) => f.metadata(),
        }
    }}
}

pub struct FileReaderParam {
    pub write    : bool,
    pub buffered : bool,
}

enum FileHandle {
    Buffered(BufReader<File>),
    Unbuffered(File),
}

pub struct ReadResult {
    pub len_read : usize,
    pub eof_seen : bool,
}

pub struct FileReader {
    file          : FileHandle,
    path          : String,
    write_enabled : bool,
}

impl FileReader {
    pub fn new(path  : &str,
               param : FileReaderParam) -> Result<FileReader, Error> {
        let write_enabled = param.write;
        let file =
            OpenOptions::new()
            .write(write_enabled)
            .read(true)
            .open(&path);
        let file = match file {
            Ok(f)  => f,
            Err(e) => { return Err(to_err(FileError::new(e.kind(), path))); }
        };
        Ok (FileReader {
            file :
            if param.buffered {
                FileHandle::Buffered(BufReader::new(file))
            } else {
                FileHandle::Unbuffered(file)
            },
            path : String::from(path),
            write_enabled,
        })
    }

    pub fn read(&mut self, buf : &mut [u8]) -> Result<ReadResult, Error> {
        let mut len_read = 0;
        let mut tries    = 0;
        while len_read < buf.len() && tries < READ_RETRIES {
            match file_op!(self read => &mut buf[len_read..]) {
                Ok(len) => { len_read += len; },
                Err(e)  => { return Err(to_err(FileError::new(e.kind(), &self.path))); }
            }

            tries += 1;
        }

        Ok(ReadResult { len_read,
                        eof_seen : len_read < buf.len() })
    }

    pub fn write(&mut self, buf : &[u8]) -> Result<usize, Error> {
        if !self.write_enabled {
            panic!("Write not enabled");
        }

        match file_op!(self write => buf) {
            Ok(len) => Ok(len),
            Err(e)  => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
    }

    pub fn seek(&mut self, pos : SeekFrom)
                -> Result<u64, Error> {
        match file_op!(self seek => pos) {
            Ok(pos) => Ok(pos),
            Err(e)  => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
    }

    pub fn cur_pos(&mut self) -> Result<u64, Error> {
        match file_op!(self seek => SeekFrom::Current(0)) {
            Ok(pos) => Ok(pos),
            Err(e)  => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
    }

    pub fn metadata(&self) -> Result<Metadata, Error> {
        match file_op!(self get_metadata) {
            Ok(data) => Ok(data),
            Err(e)   => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
    }
}
