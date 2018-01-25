mod header;
pub mod metadata;
mod crc;
mod test;
mod header_test;
mod metadata_test;

use self::header::Header;
use self::metadata::Metadata;

use super::sbx_specs::{Version,
                       SBX_HEADER_SIZE,
                       SBX_FILE_UID_LEN,
                       ver_to_block_size};
use self::crc::*;

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
    Meta(Vec<Metadata>)
}

pub struct Block {
    pub header : Header,
    data   : Data,
}

macro_rules! slice_buf {
    (
        whole => $self:ident, $buf:ident
    ) => {
        &$buf[..block_size!($self)]
    };
    (
        whole_mut => $self:ident, $buf:ident
    ) => {
        &mut $buf[..block_size!($self)]
    };
    (
        header => $self:ident, $buf:ident
    ) => {
        &$buf[..SBX_HEADER_SIZE]
    };
    (
        header_mut => $self:ident, $buf:ident
    ) => {
        &mut $buf[..SBX_HEADER_SIZE]
    };
    (
        data => $self:ident, $buf:ident
    ) => {
        &$buf[SBX_HEADER_SIZE..block_size!($self)]
    };
    (
        data_mut => $self:ident, $buf:ident
    ) => {
        &mut $buf[SBX_HEADER_SIZE..block_size!($self)]
    }
}

macro_rules! check_buffer {
    (
        $self:ident, $buf:ident
    ) => {
        if $buf.len() < block_size!($self) { panic!("Insufficient buffer size"); }
    }
}

macro_rules! block_size {
    (
        $self:ident
    ) => {
        ver_to_block_size($self.header.version)
    }
}

/*macro_rules! data_size {
    (
        $self:ident
    ) => {
        ver_to_data_size($self.header.version)
    }
}*/

pub fn slice_buf(version : Version,
                 buffer  : & [u8]) -> & [u8] {
    &buffer[..ver_to_block_size(version)]
}

pub fn slice_buf_mut(version : Version,
                     buffer  : &mut [u8]) -> &mut [u8] {
    &mut buffer[..ver_to_block_size(version)]
}

pub fn slice_header_buf(version : Version,
                        buffer  : &[u8]) -> &[u8] {
    &buffer[..SBX_HEADER_SIZE]
}

pub fn slice_header_buf_mut(version : Version,
                            buffer : &mut [u8]) -> &mut [u8] {
    &mut buffer[..SBX_HEADER_SIZE]
}

pub fn slice_data_buf(version : Version,
                      buffer : &[u8]) -> &[u8] {
    &buffer[SBX_HEADER_SIZE..ver_to_block_size(version)]
}

pub fn slice_data_buf_mut(version : Version,
                          buffer  : &mut [u8]) -> &mut [u8] {
    &mut buffer[SBX_HEADER_SIZE..ver_to_block_size(version)]
}

impl Block {
    pub fn new(version    : Version,
               file_uid   : &[u8; SBX_FILE_UID_LEN],
               block_type : BlockType)
               -> Block {
        match block_type {
            BlockType::Data => {
                Block {
                    header : Header::new(version, file_uid.clone()),
                    data   : Data::Data,
                }
            },
            BlockType::Meta => {
                Block {
                    header : Header::new(version, file_uid.clone()),
                    data   : Data::Meta(Vec::with_capacity(10)),
                }
            }
        }
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

    pub fn meta(&self) -> Result<&Vec<Metadata>, Error> {
        match self.data {
            Data::Data        => Err(Error::IncorrectBlockType),
            Data::Meta(ref x) => { Ok(x) }
        }
    }

    pub fn meta_mut(&mut self) -> Result<&mut Vec<Metadata>, Error> {
        match self.data {
            Data::Data            => Err(Error::IncorrectBlockType),
            Data::Meta(ref mut x) => { Ok(x) }
        }
    }

    pub fn calc_crc(&self, buffer : &[u8]) -> Result<u16, Error> {
        check_buffer!(self, buffer);

        self.check_header_type_matches_block_type()?;

        let crc = self.header.calc_crc();

        Ok(crc_ccitt_generic(crc, slice_buf!(data => self, buffer)))
    }

    pub fn update_crc(&mut self,
                      buffer : &[u8])
                      -> Result<(), Error> {
        self.header.crc = self.calc_crc(buffer)?;

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

    pub fn sync_to_buffer(&mut self,
                          update_crc : Option<bool>,
                          buffer     : &mut [u8])
                          -> Result<(), Error> {
        check_buffer!(self, buffer);

        self.check_header_type_matches_block_type()?;

        let update_crc = match update_crc {
            Some(v) => v,
            None    => true
        };

        match self.data {
            Data::Meta(ref meta) => {
                // transform metadata to bytes
                metadata::to_bytes(meta, slice_buf!(data_mut => self, buffer))?;
            },
            Data::Data => {}
        }

        match self.block_type() {
            BlockType::Data => if update_crc { self.update_crc(buffer)? },
            BlockType::Meta =>                 self.update_crc(buffer)?
        }

        self.header.to_bytes(slice_buf!(header_mut => self, buffer)).unwrap();

        Ok(())
    }

    fn switch_block_type(&mut self) {
        let block_type = self.block_type();

        if block_type == BlockType::Meta {
            self.data = Data::Data;
        } else {
            self.data = Data::Meta(Vec::with_capacity(10));
        }
    }

    pub fn switch_block_type_to_match_header(&mut self) {
        if !self.header_type_matches_block_type() {
            self.switch_block_type();
        }
    }

    pub fn sync_from_buffer_header_only(&mut self,
                                        buffer : &[u8])
                                        -> Result<(), Error> {
        self.header.from_bytes(slice_buf!(header => self, buffer))?;

        Ok(())
    }

    pub fn sync_from_buffer(&mut self,
                            buffer : &[u8])
                            -> Result<(), Error> {
        self.sync_from_buffer_header_only(buffer)?;

        check_buffer!(self, buffer);

        self.switch_block_type_to_match_header();

        match self.data {
            Data::Meta(ref mut meta) => {
                meta.clear();
                let res = metadata::from_bytes(slice_buf!(data => self, buffer))?;
                for r in res.into_iter() {
                    meta.push(r);
                }
            },
            Data::Data => {}
        }

        Ok(())
    }

    pub fn verify_crc(&self,
                      buffer : &[u8])
                      -> Result<bool, Error> {
        Ok(self.header.crc == self.calc_crc(buffer)?)
    }
}
