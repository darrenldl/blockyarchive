#![allow(dead_code)]
use general_error::Error;
use file_error::FileError;
use file_error::to_err;
use std::io::Read;
use std::io::Write;
use std::io::BufWriter;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Seek;
use std::fs::Metadata;
use std::fs::OpenOptions;

macro_rules! flush {
    (
        $self:ident => $file:expr
    ) => {{
        match $file.flush() {
            Ok(_)  => {},
            Err(e) => { return Err(to_err(FileError::new(e.kind(), &$self.path))); },
        }
    }}
}

macro_rules! file_op {
    (
        $self:ident read => $input:expr
    ) => {{
        use self::FileHandle::*;
        match $self.file {
            Buffered(ref mut f)   => {
                flush!($self => f);

                f.get_mut().read($input)
            },
            Unbuffered(ref mut f) => {
                f.read($input)
            },
        }
    }};
    (
        $self:ident set_len => $input:expr
    ) => {{
        use self::FileHandle::*;
        match $self.file {
            Buffered(ref mut f)   => {
                flush!($self => f);

                f.get_ref().set_len($input)
            },
            Unbuffered(ref mut f) => {
                f.set_len($input)
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

pub struct FileWriterParam {
    pub read     : bool,
    pub append   : bool,
    pub buffered : bool,
}

enum FileHandle {
    Buffered(BufWriter<File>),
    Unbuffered(File),
}

pub struct FileWriter {
    file         : FileHandle,
    path         : String,
    read_enabled : bool,
}

impl FileWriter {
    pub fn new(path  : &str,
               param : FileWriterParam) -> Result<FileWriter, Error> {
        let read_enabled = param.read;
        let file =
            OpenOptions::new()
            .append(param.append)
            .read(read_enabled)
            .truncate(!param.append)
            .write(true)
            .create(true)
            .open(&path);
        let file = match file {
            Ok(f) => f,
            Err(e) => { return Err(to_err(FileError::new(e.kind(), path))); }
        };
        Ok (FileWriter {
            file :
            if param.buffered {
                FileHandle::Buffered(BufWriter::new(file))
            } else {
                FileHandle::Unbuffered(file)
            },
            path : String::from(path),
            read_enabled,
        })
    }

    pub fn write(&mut self, buf : &[u8]) -> Result<usize, Error> {
        match file_op!(self write => buf) {
            Ok(len_written) => Ok(len_written),
            Err(e)          => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
    }

    pub fn read(&mut self, buf : &mut [u8]) -> Result<usize, Error> {
        if !self.read_enabled {
            panic!("Read not enabled");
        }

        match file_op!(self read => buf) {
            Ok(len) => Ok(len),
            Err(e)  => Err(to_err(FileError::new(e.kind(), &self.path)))
        }
    }

    pub fn set_len(&mut self, size : u64) -> Result<(), Error> {
        match file_op!(self set_len => size) {
            Ok(_)  => Ok(()),
            Err(e) => Err(to_err(FileError::new(e.kind(), &self.path)))
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

    pub fn get_file_size(&mut self) -> Result<u64, Error> {
        let cur_pos = self.cur_pos()?;

        let last_pos = self.seek(SeekFrom::End(0))?;

        self.seek(SeekFrom::Start(cur_pos))?;

        Ok(last_pos)
    }
}
