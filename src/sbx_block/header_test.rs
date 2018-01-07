#![cfg(test)]

use super::super::sbx_specs::Version;
use super::header::Header;
use super::Error;

#[test]
fn test_from_bytes_versions() {
    let mut header = Header::new(Version::V1, [0; 6]);

    {
        let buffer : &[u8; 16] = b"SBx\x01\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        header.from_bytes(buffer).unwrap();

        assert_eq!(header.version, Version::V1);
        assert_eq!(header.crc, 0xCDEF);
        assert_eq!(header.file_uid, *b"\x00\x01\x02\x03\x04\x05");
        assert_eq!(header.seq_num, 0x01020304);
    }
    {
        let buffer : &[u8; 16] = b"SBx\x02\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        header.from_bytes(buffer).unwrap();

        assert_eq!(header.version, Version::V2);
        assert_eq!(header.crc, 0xCDEF);
        assert_eq!(header.file_uid, *b"\x00\x01\x02\x03\x04\x05");
        assert_eq!(header.seq_num, 0x01020304);
    }
    {
        let buffer : &[u8; 16] = b"SBx\x03\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        header.from_bytes(buffer).unwrap();

        assert_eq!(header.version, Version::V3);
        assert_eq!(header.crc, 0xCDEF);
        assert_eq!(header.file_uid, *b"\x00\x01\x02\x03\x04\x05");
        assert_eq!(header.seq_num, 0x01020304);
    }
    {
        let buffer : &[u8; 16] = b"SBx\x0B\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        header.from_bytes(buffer).unwrap();

        assert_eq!(header.version, Version::V11);
        assert_eq!(header.crc, 0xCDEF);
        assert_eq!(header.file_uid, *b"\x00\x01\x02\x03\x04\x05");
        assert_eq!(header.seq_num, 0x01020304);
    }
    {
        let buffer : &[u8; 16] = b"SBx\x0C\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        header.from_bytes(buffer).unwrap();

        assert_eq!(header.version, Version::V12);
        assert_eq!(header.crc, 0xCDEF);
        assert_eq!(header.file_uid, *b"\x00\x01\x02\x03\x04\x05");
        assert_eq!(header.seq_num, 0x01020304);
    }
    {
        let buffer : &[u8; 16] = b"SBx\x0D\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        header.from_bytes(buffer).unwrap();

        assert_eq!(header.version, Version::V13);
        assert_eq!(header.crc, 0xCDEF);
        assert_eq!(header.file_uid, *b"\x00\x01\x02\x03\x04\x05");
        assert_eq!(header.seq_num, 0x01020304);
    }
}

#[test]
fn test_to_bytes_versions() {
    {
        let mut header =
            Header::new(Version::V1,
                        [0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
        header.crc = 0xCDEF;
        header.seq_num = 0x01020304;

        let mut buffer : [u8; 16] = [0; 16];

        header.to_bytes(&mut buffer);
    }
}

#[test]
fn test_from_bytes_error_handling() {
    let mut header = Header::new(Version::V1, [0; 6]);

    {
        let buffer : [u8; 15] = [0; 15];
        assert_eq!(Error::IncorrectBufferSize,
                   header.from_bytes(&buffer).unwrap_err());
    }
    {
        let buffer : [u8; 17] = [0; 17];
        assert_eq!(Error::IncorrectBufferSize,
                   header.from_bytes(&buffer).unwrap_err());
    }
    {
        let buffer : &[u8; 16] = b"SBx\x00\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        assert_eq!(Error::ParseError,
                   header.from_bytes(buffer).unwrap_err());
    }
}

#[test]
fn test_from_bytes() {
    let mut header = Header::new(Version::V1, [0; 6]);

    {
        let buffer : &[u8; 16] = b"SBx\x0B\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        header.from_bytes(buffer).unwrap();

        assert_eq!(header.version, Version::V11);
        assert_eq!(header.crc, 0xCDEF);
        assert_eq!(header.file_uid, *b"\x00\x01\x02\x03\x04\x05");
        assert_eq!(header.seq_num, 0x01020304);
    }

    {
        let buffer : &[u8; 16] = b"SBx\x01\xCD\xEF\xBE\x0A\x02\x03\x04\x05\x01\x02\x03\x04";

        header.from_bytes(buffer).unwrap();

        assert_eq!(header.version, Version::V1);
        assert_eq!(header.crc, 0xCDEF);
        assert_eq!(header.file_uid, *b"\xBE\x0A\x02\x03\x04\x05");
        assert_eq!(header.seq_num, 0x01020304);
    }
    {
        let buffer : &[u8; 16] = b"SBx\x0B\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        header.from_bytes(buffer).unwrap();

        assert_eq!(header.version, Version::V11);
        assert_eq!(header.crc, 0xCDEF);
        assert_eq!(header.file_uid, *b"\x00\x01\x02\x03\x04\x05");
        assert_eq!(header.seq_num, 0x01020304);
    }
}
