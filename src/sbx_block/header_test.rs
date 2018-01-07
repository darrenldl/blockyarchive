#![cfg(test)]

use super::super::sbx_specs::Version;
use super::header::Header;

#[test]
fn test_from_bytes() {
    let mut header = Header::new(Version::V1,
                             [0; 6]);

    let buffer : &[u8; 16] = b"SBx\x01\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

    header.from_bytes(buffer).unwrap();

    assert_eq!(header.version, Version::V1);
    assert_eq!(header.crc, 0xCDEF);
    assert_eq!(header.file_uid, *b"\x00\x01\x02\x03\x04\x05");
    assert_eq!(header.seq_num, 0x01020304);
}
