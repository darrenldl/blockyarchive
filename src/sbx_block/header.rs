use super::super::sbx_specs::{Version, SBX_HEADER_SIZE};

struct raw_header {
    version   : Version,
    crc_ccitt : u16,
    file_uid  : &[u8; SBX_HEADER_SIZE],
}
