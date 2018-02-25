use super::Error;
use super::file_error::FileError;
use super::file_error::to_err;
use std::io::Read;
use std::io::BufReader;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Seek;
use std::fs::Metadata;
use std::fs::OpenOptions;

const READ_RETRIES : usize = 5;

macro_rules! file_op {
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

pub struct FileReader {
    file : FileHandle,
    path : String,
}

impl FileReader {
    pub fn new(path  : &str,
               param : FileReaderParam) -> Result<FileReader, Error> {
        let file =
            OpenOptions::new()
            .write(param.write)
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
            path : String::from(path)
        })
    }

    pub fn read(&mut self, buf : &mut [u8]) -> Result<usize, Error> {
        let mut len_read = 0;
        let mut tries    = 0;
        while len_read < buf.len() && tries < READ_RETRIES {
            match file_op!(self read => &mut buf[len_read..]) {
                Ok(len) => { len_read += len; },
                Err(e)  => { return Err(to_err(FileError::new(e.kind(), &self.path))); }
            }

            tries += 1;
        }

        Ok(len_read)
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
