mod header;
mod metadata;
mod crc;
mod test;
mod header_test;
mod metadata_test;

use self::header::Header;
use self::metadata::Metadata;

use super::sbx_specs::{Version,
                       SBX_HEADER_SIZE,
                       SBX_FILE_UID_LEN,
                       SBX_LARGEST_BLOCK_SIZE};
extern crate reed_solomon_erasure;
extern crate smallvec;
use self::smallvec::SmallVec;

use self::crc::*;

use super::sbx_specs;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockType {
    Data, Meta
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    IncorrectBlockType,
    InconsistentHeaderBlockType,
    IncorrectBufferSize,
    TooMuchMetaData,
    ParseError
}

#[derive(Debug, PartialEq)]
pub enum Data {
    Data,
    Meta(SmallVec<[Metadata; 16]>)
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub header : Header,
    data       : Data,
    buffer     : SmallVec<[u8; SBX_LARGEST_BLOCK_SIZE]>,
}

macro_rules! get_buf {
    (
        header => $self:ident
    ) => {
        &$self.buffer[..SBX_HEADER_SIZE]
    };
    (
        header_mut => $self:ident
    ) => {
        &mut $self.buffer[..SBX_HEADER_SIZE]
    };
    (
        data => $self:ident
    ) => {
        &$self.buffer[SBX_HEADER_SIZE..]
    };
    (
        data_mut => $self:ident
    ) => {
        &mut $self.buffer[SBX_HEADER_SIZE..]
    };
}

impl Block {
    pub fn new(version    : Version,
               file_uid   : &[u8; SBX_FILE_UID_LEN],
               block_type : BlockType)
               -> Result<Block, Error> {
        Ok(match block_type {
            BlockType::Data => {
                Block {
                    header : Header::new(version, file_uid.clone()),
                    data   : Data::Data,
                    buffer : SmallVec::new()
                }
            },
            BlockType::Meta => {
                Block {
                    header : Header::new(version, file_uid.clone()),
                    data   : Data::Meta(SmallVec::new()),
                    buffer : SmallVec::new()
                }
            }
        })
    }

    pub fn header_data_buf(&self) -> (&[u8], &[u8]) {
        self.buffer.split_at(SBX_HEADER_SIZE)
    }

    pub fn header_data_buf_mut(&mut self) -> (&mut [u8], &mut [u8]) {
        self.buffer.split_at_mut(SBX_HEADER_SIZE)
    }

    pub fn header_buf(&self) -> &[u8] {
        self.header_data_buf().0
    }

    pub fn header_buf_mut(&mut self) -> &mut [u8] {
        self.header_data_buf_mut().0
    }

    pub fn data_buf(&self) -> &[u8] {
        self.header_data_buf().1
    }

    pub fn data_buf_mut(&mut self) -> &mut [u8] {
        self.header_data_buf_mut().1
    }

    pub fn block_type(&self) -> BlockType {
        match self.data {
            Data::Data    => BlockType::Data,
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

    pub fn get_meta_ref(&self) -> Result<&SmallVec<[Metadata; 16]>, Error> {
        match self.data {
            Data::Data        => Err(Error::IncorrectBlockType),
            Data::Meta(ref x) => { Ok(x) }
        }
    }

    pub fn get_meta_ref_mut(&mut self) -> Result<&mut SmallVec<[Metadata; 16]>, Error> {
        match self.data {
            Data::Data            => Err(Error::IncorrectBlockType),
            Data::Meta(ref mut x) => { Ok(x) }
        }
    }

    pub fn calc_crc(&self) -> Result<u16, Error> {
        self.check_header_type_matches_block_type()?;

        let crc = self.header.calc_crc();

        Ok(crc_ccitt_generic(crc, self.data_buf()))
    }

    pub fn update_crc(&mut self) -> Result<(), Error> {
        self.header.crc = self.calc_crc()?;

        Ok(())
    }

    fn header_type_matches_block_type(&self) -> bool {
        self.header.is_meta() == self.is_meta()
    }

    fn check_header_type_matches_block_type(&self) -> Result<(), Error> {
        if self.header_type_matches_block_type() {
            Ok(())
        } else {
            Err(Error::InconsistentHeaderBlockType)
        }
    }

    pub fn sync_to_buffer(&mut self) -> Result<(), Error> {
        self.check_header_type_matches_block_type()?;

        match self.data {
            Data::Meta(ref meta) => {
                // transform metadata to bytes
                metadata::to_bytes(meta, get_buf!(data_mut => self))?;
            },
            Data::Data => {}
        }

        self.update_crc()?;

        self.header.to_bytes(get_buf!(header_mut => self)).unwrap();

        Ok(())
    }

    fn switch_block_type(&mut self) {
        let block_type = self.block_type();

        if block_type == BlockType::Meta {
            self.data = Data::Data;
        } else {
            self.data = Data::Meta(SmallVec::new());
        }
    }

    pub fn switch_block_type_to_match_header(&mut self) {
        if !self.header_type_matches_block_type() {
            self.switch_block_type();
        }
    }

    pub fn sync_from_buffer_header_only(&mut self) -> Result<(), Error> {
        self.header.from_bytes(get_buf!(header_mut => self))?;

        self.switch_block_type_to_match_header();

        Ok(())
    }

    pub fn sync_from_buffer(&mut self) -> Result<(), Error> {
        self.sync_from_buffer_header_only()?;

        match self.data {
            Data::Meta(ref mut meta) => {
                meta.clear();
                let res = metadata::from_bytes(get_buf!(header => self))?;
                for r in res.into_iter() {
                    meta.push(r);
                }
            },
            Data::Data => {}
        }

        Ok(())
    }

    pub fn verify_crc(&self) -> Result<bool, Error> {
        Ok(self.header.crc == self.calc_crc()?)
    }
}
