use super::Error;
use super::file_error::FileError;
use super::file_error::to_err;
use std::io::Read;
use std::io::Write;
use std::io::BufWriter;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Seek;
use std::fs::OpenOptions;

macro_rules! file_op {
    (
        $self:ident read => $input:expr
    ) => {{
        use self::FileHandle::*;
        match $self.file {
            Buffered(ref mut f)   => {
                // write buffered content
                match f.flush() {
                    Ok(_)  => {},
                    Err(e) => { return Err(to_err(FileError::new(e.kind(), &$self.path))); },
                }

                f.get_mut().read($input)
            },
            Unbuffered(ref mut f) => {
                f.read($input)
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
}
