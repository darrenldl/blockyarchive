use super::Error;
use super::file_error::FileError;
use super::file_error::to_err;
use std::io::Write;
use std::io::BufWriter;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Seek;

macro_rules! file_op {
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

enum FileHandle {
    Buffered(BufWriter<File>),
    Unbuffered(File),
}

pub struct FileWriter {
    file : FileHandle,
    path : String,
}

impl FileWriter {
    pub fn new(path : &str, buffered : bool) -> Result<FileWriter, Error> {
        let file = match File::create(&path) {
            Ok(f) => f,
            Err(e) => { return Err(to_err(FileError::new(e.kind(), path))); }
        };
        Ok (FileWriter {
            file :
            if buffered {
                FileHandle::Buffered(BufWriter::new(file))
            } else {
                FileHandle::Unbuffered(file)
            },
            path : String::from(path)
        })
    }

    pub fn write(&mut self, buf : &[u8]) -> Result<usize, Error> {
        match file_op!(self write => buf) {
            Ok(len_wrote) => Ok(len_wrote),
            Err(e)        => Err(to_err(FileError::new(e.kind(), &self.path)))
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
