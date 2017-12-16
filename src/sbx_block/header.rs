use super::super::sbx_specs::{Version, SBX_HEADER_SIZE};

pub struct Header {
    version   : Version,
    crc_ccitt : u16,
    file_uid  : [u8; SBX_HEADER_SIZE],
    seq_num   : u32
}
