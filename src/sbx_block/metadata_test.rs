#![cfg(test)]

use super::metadata;
use super::metadata::Metadata;
use super::super::misc_utils::slice_to_boxed;
use super::super::multihash;

#[test]
fn test_to_bytes_simple_cases() {
    {
        let expect = b"FNM\x0Ahelloworld";
        let meta = [Metadata::FNM(slice_to_boxed(b"helloworld"))];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"SNM\x07cheerio";
        let meta = [Metadata::SNM(slice_to_boxed(b"cheerio"))];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"FSZ\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF";
        let meta = [Metadata::FSZ(0x01234567_89ABCDEF)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"FDT\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF";
        let meta = [Metadata::FDT(0x01234567_89ABCDEF)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"SDT\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF";
        let meta = [Metadata::SDT(0x01234567_89ABCDEF)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"HSH\x16\x11\x14\xaa\xf4\xc6\x1d\xdc\xc5\xe8\xa2\xda\xbe\xde\x0f\x3b\x48\x2c\xd9\xae\xa9\x43\x4d";

        let mut ctx = multihash::hash::Ctx::new(multihash::HashType::SHA1).unwrap();
        ctx.update(b"hello");
        let hbytes = ctx.finish_into_hash_bytes();

        let meta = [Metadata::HSH(hbytes)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
}

#[test]
fn test_to_bytes_simple_cases_with_padding() {
    {
        let expect = b"FNM\x0Ahelloworld\x1A\x1A\x1A\x1A";
        let meta = [Metadata::FNM(slice_to_boxed(b"helloworld"))];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"SNM\x07cheerio\x1A\x1A\x1A\x1A";
        let meta = [Metadata::SNM(slice_to_boxed(b"cheerio"))];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"FSZ\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF\x1A\x1A\x1A\x1A";
        let meta = [Metadata::FSZ(0x01234567_89ABCDEF)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"FDT\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF\x1A\x1A\x1A\x1A";
        let meta = [Metadata::FDT(0x01234567_89ABCDEF)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"SDT\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF\x1A\x1A\x1A\x1A";
        let meta = [Metadata::SDT(0x01234567_89ABCDEF)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
    {
        let expect = b"HSH\x16\x11\x14\xaa\xf4\xc6\x1d\xdc\xc5\xe8\xa2\xda\xbe\xde\x0f\x3b\x48\x2c\xd9\xae\xa9\x43\x4d";

        let mut ctx = multihash::hash::Ctx::new(multihash::HashType::SHA1).unwrap();
        ctx.update(b"hello");
        let hbytes = ctx.finish_into_hash_bytes();

        let meta = [Metadata::HSH(hbytes)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
}

#[test]
fn test_from_bytes_simple_cases() {
    {
        let input = b"FNM\x0Ahelloworld";
        let expect = Metadata::FNM(slice_to_boxed(b"helloworld"));

        let metas = metadata::from_bytes(input).unwrap();
        assert_eq!(1, metas.len());

        assert_eq!(expect, metas[0]);
    }
    {
        let input = b"SNM\x0Ahelloworld";
        let expect = Metadata::SNM(slice_to_boxed(b"helloworld"));

        let metas = metadata::from_bytes(input).unwrap();
        assert_eq!(1, metas.len());

        assert_eq!(expect, metas[0]);
    }
    {
        let input = b"FSZ\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF";
        let expect = Metadata::FSZ(0x01234567_89ABCDEF);

        let metas = metadata::from_bytes(input).unwrap();
        assert_eq!(1, metas.len());

        assert_eq!(expect, metas[0]);
    }
    {
        let input = b"FDT\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF";
        let expect = Metadata::FDT(0x01234567_89ABCDEF);

        let metas = metadata::from_bytes(input).unwrap();
        assert_eq!(1, metas.len());

        assert_eq!(expect, metas[0]);
    }
    {
        let input = b"SDT\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF";
        let expect = Metadata::SDT(0x01234567_89ABCDEF);

        let metas = metadata::from_bytes(input).unwrap();
        assert_eq!(1, metas.len());

        assert_eq!(expect, metas[0]);
    }
    {
        let input = b"HSH\x16\x11\x14\xaa\xf4\xc6\x1d\xdc\xc5\xe8\xa2\xda\xbe\xde\x0f\x3b\x48\x2c\xd9\xae\xa9\x43\x4d";
        let mut ctx = multihash::hash::Ctx::new(multihash::HashType::SHA1).unwrap();
        ctx.update(b"hello");
        let hbytes = ctx.finish_into_hash_bytes();

        let expect = Metadata::HSH(hbytes);

        let metas = metadata::from_bytes(input).unwrap();
        assert_eq!(1, metas.len());

        assert_eq!(expect, metas[0]);
    }
}
