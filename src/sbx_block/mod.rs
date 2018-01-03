mod helper;
mod header;
mod metadata;
mod crc;

use self::header::Header;
use self::metadata::Metadata;

use super::sbx_specs::{Version, SBX_HEADER_SIZE};
extern crate reed_solomon_erasure;
extern crate smallvec;
use self::smallvec::SmallVec;

use self::crc::crc_ccitt;

#[derive(Clone, Copy, Debug)]
pub enum BlockType {
    Data, Meta
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    WrongBlockType,
    Metadata(metadata::Error)
}

#[derive(Debug)]
pub enum Data<'a> {
    Data(&'a [u8]),
    Meta(SmallVec<[Metadata; 16]>, &'a mut [u8])
}

#[derive(Debug)]
pub struct Block<'a> {
    header     : Header,
    data       : Data<'a>,
    header_buf : &'a mut [u8],
}

impl<'a> Block<'a> {
    pub fn new(version    : Version,
               file_uid   : [u8; SBX_HEADER_SIZE],
               block_type : BlockType,
               buffer     : &'a mut [u8])
               -> Block {
        match block_type {
            BlockType::Data => {
                let (header_buf, data_buf) = buffer.split_at_mut(16);
                Block {
                    header     : Header::new(version, file_uid),
                    data       : Data::Data(data_buf),
                    header_buf : header_buf,
                }
            },
            BlockType::Meta => {
                let (header_buf, data_buf) = buffer.split_at_mut(16);
                Block {
                    header     : Header::new(version, file_uid),
                    data       : Data::Meta(SmallVec::new(), data_buf),
                    header_buf : header_buf,
                }
            }
        }
    }

    pub fn block_type(&self) -> BlockType {
        match self.data {
            Data::Data(_)    => BlockType::Data,
            Data::Meta(_, _) => BlockType::Meta
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

    pub fn add_meta(&mut self,
                     meta : Metadata) -> Result<(), Error> {
        match self.data {
            Data::Data(_) => Err(Error::WrongBlockType),
            Data::Meta(ref mut x, _) => {
                x.push(meta);
                Ok(())
            }
        }
    }

    pub fn sync_everything(&mut self) -> Result<(), Error> {
        match self.data {
            Data::Meta(ref meta, ref mut buf) => {
                if let Err(x) = metadata::write_to_bytes(meta, buf) {
                    return Err(Error::Metadata(x));
                }
                let crc = crc_ccitt(self.header.version, buf);
                let crc = crc_ccitt()
            },
            Data::Data(buf) => {
                self.header.crc = crc_ccitt(self.header.version, buf);
            }
        }

        Ok(())
    }
}
