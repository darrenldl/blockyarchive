#![cfg(test)]

use super::super::sbx_specs::Version;
use super::header::Header;

#[test]
fn test_from_bytes() {
    let mut header = Header::new(Version::V1,
                             [0; 6]);

    let buffer : &[u8; 16] = b"SBx\x01\xCD\xEF\x03\x03\x03\x03\x03\x03\x00\x00\x00\x00";

    header.from_bytes(buffer).unwrap();

    assert_eq!(header.version, Version::V1);
    assert_eq!(header.crc, 0xCDEF);
}
