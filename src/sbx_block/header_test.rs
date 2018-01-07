#![cfg(test)]

use super::super::sbx_specs::Version;
use super::header::Header;
use super::Error;
use super::super::rand_utils::fill_random_bytes;

extern crate rand;

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

        header.to_bytes(&mut buffer).unwrap();

        assert_eq!(*b"SBx\x01\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04",
                   buffer);
    }
    {
        let mut header =
            Header::new(Version::V2,
                        [0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
        header.crc = 0xCDEF;
        header.seq_num = 0x01020304;

        let mut buffer : [u8; 16] = [0; 16];

        header.to_bytes(&mut buffer).unwrap();

        assert_eq!(*b"SBx\x02\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04",
                   buffer);
    }
    {
        let mut header =
            Header::new(Version::V3,
                        [0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
        header.crc = 0xCDEF;
        header.seq_num = 0x01020304;

        let mut buffer : [u8; 16] = [0; 16];

        header.to_bytes(&mut buffer).unwrap();

        assert_eq!(*b"SBx\x03\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04",
                   buffer);
    }
    {
        let mut header =
            Header::new(Version::V11,
                        [0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
        header.crc = 0xCDEF;
        header.seq_num = 0x01020304;

        let mut buffer : [u8; 16] = [0; 16];

        header.to_bytes(&mut buffer).unwrap();

        assert_eq!(*b"SBx\x0B\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04",
                   buffer);
    }
    {
        let mut header =
            Header::new(Version::V12,
                        [0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
        header.crc = 0xCDEF;
        header.seq_num = 0x01020304;

        let mut buffer : [u8; 16] = [0; 16];

        header.to_bytes(&mut buffer).unwrap();

        assert_eq!(*b"SBx\x0C\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04",
                   buffer);
    }
    {
        let mut header =
            Header::new(Version::V13,
                        [0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
        header.crc = 0xCDEF;
        header.seq_num = 0x01020304;

        let mut buffer : [u8; 16] = [0; 16];

        header.to_bytes(&mut buffer).unwrap();

        assert_eq!(*b"SBx\x0D\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04",
                   buffer);
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
    {
        let buffer : &[u8; 16] = b"ABx\x00\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        assert_eq!(Error::ParseError,
                   header.from_bytes(buffer).unwrap_err());
    }
    {
        let buffer : &[u8; 16] = b"SBx\x0E\xCD\xEF\x00\x01\x02\x03\x04\x05\x01\x02\x03\x04";

        assert_eq!(Error::ParseError,
                   header.from_bytes(buffer).unwrap_err());
    }
}

#[test]
fn test_to_bytes_error_handling() {
    let header = Header::new(Version::V1, [0; 6]);

    {
        let mut buffer : [u8; 15] = [0; 15];
        assert_eq!(Error::IncorrectBufferSize,
                   header.to_bytes(&mut buffer).unwrap_err());
    }
    {
        let mut buffer : [u8; 17] = [0; 17];
        assert_eq!(Error::IncorrectBufferSize,
                   header.to_bytes(&mut buffer).unwrap_err());
    }
}

#[test]
fn test_to_from_to_bytes() {
    for _ in 0..1000 {
        let mut header = Header::new(Version::V1, [0; 6]);
        header.crc = rand::random::<u16>();
        fill_random_bytes(&mut header.file_uid);
        header.seq_num = rand::random::<u32>();

        let mut expect : [u8; 16] =
            [b'S', b'B', b'x', 0x01, 0xCC, 0xCC, 0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 0xEE, 0xEE, 0xEE, 0xEE];
        expect[4] = (header.crc >> 8) as u8;
        expect[5] =  header.crc      as u8;
        for i in 0..6 {
            expect[6 + i] = header.file_uid[i];
        }
        expect[12] = (header.seq_num >> 24) as u8;
        expect[13] = (header.seq_num >> 16) as u8;
        expect[14] = (header.seq_num >>  8) as u8;
        expect[15] =  header.seq_num        as u8;
        {
            let mut buffer : [u8; 16] = [0; 16];

            header.to_bytes(&mut buffer).unwrap();

            assert_eq!(expect, buffer);

            let mut header2 = Header::new(Version::V2, [11; 6]);

            header2.from_bytes(&buffer).unwrap();

            assert_eq!(header, header2);

            let mut buffer2 : [u8; 16] = [0; 16];

            header.to_bytes(&mut buffer2).unwrap();

            assert_eq!(expect, buffer2);
        }
    }
}
