use super::super::sbx_specs::{Version, SBX_HEADER_SIZE};

extern crate reed_solomon;

pub struct Header {
    version   : Version,
    crc_ccitt : u16,
    file_uid  : &[u8; SBX_HEADER_SIZE],
    seq_num   : u32
}
