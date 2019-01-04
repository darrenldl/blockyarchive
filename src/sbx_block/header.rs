use super::BlockType;
use sbx_specs;
use sbx_specs::{Version, SBX_FILE_UID_LEN, SBX_FIRST_DATA_SEQ_NUM, SBX_SIGNATURE};
use std;

use super::crc::*;

use super::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub version: Version,
    pub crc: u16,
    pub uid: [u8; SBX_FILE_UID_LEN],
    pub seq_num: u32,
}

impl Header {
    pub fn new(version: Version, uid: [u8; SBX_FILE_UID_LEN], seq_num: u32) -> Header {
        Header {
            version,
            crc: 0,
            uid,
            seq_num,
        }
    }

    pub fn to_bytes(&self, buffer: &mut [u8]) {
        if buffer.len() != 16 {
            panic!("Incorrect buffer size");
            //return Err(Error::IncorrectBufferSize);
        }

        {
            // signature
            buffer[0..3].copy_from_slice(SBX_SIGNATURE);
        }
        {
            // version byte
            buffer[3] = sbx_specs::ver_to_usize(self.version) as u8;
        }
        {
            // crc ccitt
            let crc: [u8; 2] = unsafe { std::mem::transmute::<u16, [u8; 2]>(self.crc.to_be()) };
            buffer[4..6].copy_from_slice(&crc);
        }
        {
            // file uid
            buffer[6..12].copy_from_slice(&self.uid);
        }
        {
            // seq num
            let seq_num: [u8; 4] =
                unsafe { std::mem::transmute::<u32, [u8; 4]>(self.seq_num.to_be()) };
            buffer[12..16].copy_from_slice(&seq_num);
        }
    }

    pub fn from_bytes(&mut self, buffer: &[u8]) -> Result<(), Error> {
        use super::Error;
        if buffer.len() != 16 {
            return Err(Error::IncorrectBufferSize);
        }

        match parsers::header_p(buffer) {
            Ok((_, header)) => {
                *self = header;
                Ok(())
            }
            _ => Err(Error::ParseError),
        }
    }

    pub fn calc_crc(&self) -> u16 {
        let crc = sbx_crc_ccitt(self.version, &self.uid);
        let seq_num: [u8; 4] = unsafe { std::mem::transmute::<u32, [u8; 4]>(self.seq_num.to_be()) };
        crc_ccitt_generic(crc, &seq_num)
    }

    pub fn header_type(&self) -> BlockType {
        if self.seq_num < SBX_FIRST_DATA_SEQ_NUM as u32 {
            BlockType::Meta
        } else {
            BlockType::Data
        }
    }
}

mod parsers {
    use super::Header;
    use super::Version;
    use nom::{be_u16, be_u32};

    named!(sig_p, tag!(b"SBx"));

    named!(
        ver_p<Version>,
        alt_complete!(
            do_parse!(_v: tag!(&[1]) >> (Version::V1))
                | do_parse!(_v: tag!(&[2]) >> (Version::V2))
                | do_parse!(_v: tag!(&[3]) >> (Version::V3))
                | do_parse!(_v: tag!(&[17]) >> (Version::V17))
                | do_parse!(_v: tag!(&[18]) >> (Version::V18))
                | do_parse!(_v: tag!(&[19]) >> (Version::V19))
        )
    );

    named!(uid_p, take!(6));

    named!(pub header_p <Header>,
           do_parse!(
               _sig : sig_p >>
                   version : ver_p >>
                   crc     : be_u16 >>
                   uid_raw : uid_p >>
                   seq_num : be_u32 >>
                   ({
                       let mut uid : [u8; 6] = [0; 6];
                       uid.copy_from_slice(uid_raw);
                       Header {
                           version,
                           crc,
                           uid,
                           seq_num
                       }
                   })
           )
    );
}
