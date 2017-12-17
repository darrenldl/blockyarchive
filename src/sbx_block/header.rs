extern crate crc_ccitt;

use super::super::sbx_specs::{Version, SBX_HEADER_SIZE};

pub struct Header {
    version   : Version,
    crc_ccitt : u16,
    file_uid  : [u8; SBX_HEADER_SIZE],
    seq_num   : u32
}

impl Header {
    pub fn new(version   : Version,
               crc_ccitt : u16,
               file_uid  : &[u8; SBX_HEADER_SIZE],
               seq_num : u32) -> Header {
        Header {
            version,
            crc_ccitt,
            file_uid : file_uid.clone(),
            seq_num
        }
    }
}
