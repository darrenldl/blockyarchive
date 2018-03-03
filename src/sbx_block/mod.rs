mod header;
mod metadata;
mod crc;
mod test;
mod header_test;
mod metadata_test;

use self::header::Header;
pub use self::metadata::Metadata;
pub use self::metadata::MetadataID;

use super::sbx_specs::{Version,
                       SBX_HEADER_SIZE,
                       SBX_FILE_UID_LEN,
                       SBX_FIRST_DATA_SEQ_NUM,
                       ver_to_block_size,
                       ver_uses_rs};
use self::crc::*;

use super::multihash;

macro_rules! make_meta_getter {
    (
        $func_name:ident => $meta_id:ident => $ret_type:ty
    ) => {
        #[allow(non_snake_case)]
        pub fn $func_name (&self) -> Result<Option<$ret_type>, Error> {
            match self.get_meta_ref_by_id(MetadataID::$meta_id)? {
                None                             => Ok(None),
                Some(&Metadata::$meta_id(ref x)) => Ok(Some(x.clone())),
                _                                => panic!(),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockType {
    Data, Meta
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    IncorrectBlockType,
    InconsistentHeaderBlockType,
    InsufficientBufferSize,
    IncorrectBufferSize,
    TooMuchMetaData,
    InvalidCRC,
    SeqNumOverflow,
    ParseError
}

#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    Data,
    Meta(Vec<Metadata>)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    header      : Header,
    data        : Data,
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
        if $buf.len() < block_size!($self) {
            return Err(Error::InsufficientBufferSize);
        }
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

pub fn write_padding(version : Version,
                     skip    : usize,
                     buffer  : &mut [u8]) {
    for i in SBX_HEADER_SIZE + skip..ver_to_block_size(version) {
        buffer[i] = 0x1A;
    }
}

pub fn slice_buf(version : Version,
                 buffer  : & [u8]) -> & [u8] {
    &buffer[..ver_to_block_size(version)]
}

pub fn slice_buf_mut(version : Version,
                     buffer  : &mut [u8]) -> &mut [u8] {
    &mut buffer[..ver_to_block_size(version)]
}

pub fn slice_header_buf(buffer  : &[u8]) -> &[u8] {
    &buffer[..SBX_HEADER_SIZE]
}

pub fn slice_header_buf_mut(buffer  : &mut [u8]) -> &mut [u8] {
    &mut buffer[..SBX_HEADER_SIZE]
}

pub fn slice_data_buf(version : Version,
                      buffer  : &[u8]) -> &[u8] {
    &buffer[SBX_HEADER_SIZE..ver_to_block_size(version)]
}

pub fn slice_data_buf_mut(version : Version,
                          buffer  : &mut [u8]) -> &mut [u8] {
    &mut buffer[SBX_HEADER_SIZE..ver_to_block_size(version)]
}

pub fn check_if_buffer_valid(buffer : &[u8]) -> bool {
    let mut block = Block::new(Version::V1,
                               b"\x00\x00\x00\x00\x00\x00",
                               BlockType::Data);

    match block.sync_from_buffer(buffer) {
        Ok(()) => {},
        Err(_) => { return false; }
    }

    block.verify_crc(buffer).unwrap()
}

pub fn seq_num_is_parity(seq_num       : u32,
                         data_shards   : usize,
                         parity_shards : usize) -> bool {
    if        seq_num == 0 {
        false // this is metadata block
    } else {  // data sets
        let index        = seq_num - SBX_FIRST_DATA_SEQ_NUM as u32;
        let index_in_set = index % (data_shards + parity_shards) as u32;

        (data_shards as u32 <= index_in_set)
    }
}

impl Block {
    pub fn new(version    : Version,
               file_uid   : &[u8; SBX_FILE_UID_LEN],
               block_type : BlockType)
               -> Block {
        match block_type {
            BlockType::Data => {
                let seq_num = SBX_FIRST_DATA_SEQ_NUM as u32;
                Block {
                    header : Header::new(version, file_uid.clone(), seq_num),
                    data   : Data::Data,
                }
            },
            BlockType::Meta => {
                let seq_num = 0 as u32;
                Block {
                    header : Header::new(version, file_uid.clone(), seq_num),
                    data   : Data::Meta(Vec::with_capacity(10)),
                }
            }
        }
    }

    pub fn dummy() -> Block {
        let version = Version::V1;
        let seq_num = SBX_FIRST_DATA_SEQ_NUM as u32;
        Block {
            header : Header::new(version, [0; 6], seq_num),
            data   : Data::Data,
        }
    }

    pub fn get_version(&self) -> Version {
        self.header.version
    }

    pub fn set_version(&mut self,
                       version : Version) {
        self.header.version = version;
    }

    pub fn get_file_uid(&self) -> [u8; SBX_FILE_UID_LEN] {
        self.header.file_uid
    }

    pub fn set_file_uid(&mut self,
                        file_uid : [u8; SBX_FILE_UID_LEN]) {
        self.header.file_uid = file_uid;
    }

    pub fn get_crc(&self) -> u16 {
        self.header.crc
    }

    pub fn get_seq_num(&self) -> u32 {
        self.header.seq_num
    }

    pub fn set_seq_num(&mut self,
                       seq_num : u32) {
        self.header.seq_num = seq_num;

        self.switch_block_type_to_match_header();
    }

    pub fn add_seq_num(&mut self,
                       val : u32)
                       -> Result<(), Error> {
        match self.header.seq_num.checked_add(val) {
            None    => { return Err(Error::SeqNumOverflow); },
            Some(x) => { self.header.seq_num = x; }
        }

        self.switch_block_type_to_match_header();

        Ok(())
    }

    pub fn add1_seq_num(&mut self)
                        -> Result<(), Error> {
        self.add_seq_num(1)
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

    pub fn is_parity(&self,
                     data_shards   : usize,
                     parity_shards : usize) -> bool {
        ver_uses_rs(self.header.version)
            && seq_num_is_parity(self.get_seq_num(),
                                 data_shards,
                                 parity_shards)
    }

    pub fn data_index(&self,
                      dat_par_shards : Option<(usize, usize)>)
                      -> Option<u32> {
        match self.data {
            Data::Meta(_) => None,
            Data::Data    => {
                let data_index =
                    self.header.seq_num
                    - SBX_FIRST_DATA_SEQ_NUM as u32;

                match dat_par_shards {
                    None                               => Some(data_index),
                    Some((data_shards, parity_shards)) => {
                        if self.is_parity(data_shards, parity_shards) {
                            None
                        } else {
                            let set_index =
                                data_index / (data_shards + parity_shards) as u32;
                            let index_in_set =
                                data_index % (data_shards + parity_shards) as u32;

                            Some(set_index * data_shards as u32 + index_in_set)
                        }
                    }
                }
            }
        }
    }

    pub fn get_meta_ref_by_id(&self,
                              id : MetadataID)
                              -> Result<Option<&Metadata>, Error> {
        match self.data {
            Data::Data           => Err(Error::IncorrectBlockType),
            Data::Meta(ref meta) => {
                Ok(metadata::get_meta_ref_by_id(id, meta))
            }
        }
    }

    pub fn get_meta_ref_mut_by_id(&mut self,
                                  id : MetadataID)
                                  -> Result<Option<&mut Metadata>, Error> {
        match self.data {
            Data::Data               => Err(Error::IncorrectBlockType),
            Data::Meta(ref mut meta) => {
                Ok(metadata::get_meta_ref_mut_by_id(id, meta))
            }
        }
    }

    make_meta_getter!(get_FNM => FNM => Box<[u8]>);
    make_meta_getter!(get_SNM => SNM => Box<[u8]>);
    make_meta_getter!(get_FSZ => FSZ => u64);
    make_meta_getter!(get_FDT => FDT => i64);
    make_meta_getter!(get_SDT => SDT => i64);
    make_meta_getter!(get_HSH => HSH => multihash::HashBytes);
    make_meta_getter!(get_RSD => RSD => u8);
    make_meta_getter!(get_RSP => RSP => u8);

    pub fn meta(&self) -> Result<&Vec<Metadata>, Error> {
        match self.data {
            Data::Data           => Err(Error::IncorrectBlockType),
            Data::Meta(ref meta) => Ok(meta)
        }
    }

    pub fn meta_mut(&mut self) -> Result<&mut Vec<Metadata>, Error> {
        match self.data {
            Data::Data               => Err(Error::IncorrectBlockType),
            Data::Meta(ref mut meta) => Ok(meta)
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
        self.header.header_type() == self.block_type()
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
                if self.get_seq_num() == 0 {  // not a metadata parity block
                    // transform metadata to bytes
                    metadata::to_bytes(meta, slice_buf!(data_mut => self, buffer))?;
                }
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

    fn switch_block_type_to_match_header(&mut self) {
        if !self.header_type_matches_block_type() {
            self.switch_block_type();
        }
    }

    pub fn sync_from_buffer_header_only(&mut self,
                                        buffer : &[u8])
                                        -> Result<(), Error> {
        self.header.from_bytes(slice_buf!(header => self, buffer))?;

        self.switch_block_type_to_match_header();

        Ok(())
    }

    pub fn sync_from_buffer(&mut self,
                            buffer : &[u8])
                            -> Result<(), Error> {
        self.sync_from_buffer_header_only(buffer)?;

        check_buffer!(self, buffer);

        self.enforce_crc(buffer)?;

        match self.data {
            Data::Meta(ref mut meta) => {
                // parse if it is metadata and not parity block
                // or if it is a RS parity block
                if self.header.seq_num == 0
                    || ver_uses_rs(self.header.version) {
                    meta.clear();
                    let res = metadata::from_bytes(slice_buf!(data => self, buffer))?;
                    for r in res.into_iter() {
                        meta.push(r);
                    }
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

    pub fn enforce_crc(&self,
                       buffer : &[u8])
                       -> Result<(), Error> {
        if self.verify_crc(buffer)? {
            Ok(())
        } else {
            Err(Error::InvalidCRC)
        }
    }
}
