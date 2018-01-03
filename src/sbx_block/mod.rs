mod helper;
mod header;
mod metadata;

use self::header::Header;
use self::metadata::Metadata;

use super::sbx_specs::{Version, SBX_HEADER_SIZE};
extern crate reed_solomon_erasure;
extern crate smallvec;
use self::smallvec::SmallVec;

#[derive(Clone, Copy, Debug)]
pub enum BlockType {
    Data, Meta
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    WrongBlockType
}

#[derive(Clone, Debug)]
pub enum Data {
    Data(SmallVec<[u8; 4096]>),
    Meta(SmallVec<[Metadata; 16]>)
}

#[derive(Clone, Debug)]
pub struct Block {
    header : Header,
    data   : Data
}

impl Block {
    pub fn new(version    : Version,
               file_uid   : [u8; SBX_HEADER_SIZE],
               block_type : BlockType)
               -> Block {
        Block {
            header : Header::new(version, file_uid),
            data   : match block_type {
                BlockType::Data => Data::Data(SmallVec::new()),
                BlockType::Meta => Data::Meta(SmallVec::new())
            }
        }
    }

    pub fn block_type(&self) -> BlockType {
        match self.data {
            Data::Data(_) => BlockType::Data,
            Data::Meta(_) => BlockType::Meta
        }
    }

    pub fn is_meta(&self) -> bool {
        match self.block_type() {
            BlockType::Data => false,
            BlockType::Meta => true
        }
    }

    pub fn is_data(&self) -> bool {
        match self.block_type() {
            BlockType::Data => true,
            BlockType::Meta => false
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn push_meta(&mut self,
                     meta : Metadata) -> Result<(), Error> {
        match self.data {
            Data::Data(_) => Err(Error::WrongBlockType),
            Data::Meta(ref mut x) => {
                x.push(meta);
                Ok(())
            }
        }
    }
}
