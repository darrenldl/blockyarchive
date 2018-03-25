#![cfg(test)]

use super::metadata;
use super::metadata::Metadata;
use super::metadata::MetadataID;
use multihash;

#[test]
fn test_to_bytes_simple_cases() {
    {
        let expect = b"FNM\x0Ahelloworld";
        let meta = [Metadata::FNM("helloworld".to_string())];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);

        for i in expect.len()..buffer.len() {
            assert_eq!(buffer[i], 0x1A);
        }
    }
    {
        let expect = b"SNM\x07cheerio";
        let meta = [Metadata::SNM("cheerio".to_string())];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);

        for i in expect.len()..buffer.len() {
            assert_eq!(buffer[i], 0x1A);
        }
    }
    {
        let expect = b"FSZ\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF";
        let meta = [Metadata::FSZ(0x01234567_89ABCDEF)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);

        for i in expect.len()..buffer.len() {
            assert_eq!(buffer[i], 0x1A);
        }
    }
    {
        let expect = b"FDT\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF";
        let meta = [Metadata::FDT(0x01234567_89ABCDEF)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);

        for i in expect.len()..buffer.len() {
            assert_eq!(buffer[i], 0x1A);
        }
    }
    {
        let expect = b"SDT\x08\x01\x23\x45\x67\x89\xAB\xCD\xEF";
        let meta = [Metadata::SDT(0x01234567_89ABCDEF)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);

        for i in expect.len()..buffer.len() {
            assert_eq!(buffer[i], 0x1A);
        }
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

        for i in expect.len()..buffer.len() {
            assert_eq!(buffer[i], 0x1A);
        }
    }
    {
        let expect = b"RSD\x01\x12";
        let meta = [Metadata::RSD(0x12)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);

        for i in expect.len()..buffer.len() {
            assert_eq!(buffer[i], 0x1A);
        }
    }
    {
        let expect = b"RSP\x01\x12";
        let meta = [Metadata::RSP(0x12)];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);

        for i in expect.len()..buffer.len() {
            assert_eq!(buffer[i], 0x1A);
        }
    }
}

#[test]
fn test_from_bytes_simple_cases() {
    {
        let input = b"FNM\x0Ahelloworld";
        let expect = Metadata::FNM("helloworld".to_string());

        let metas = metadata::from_bytes(input).unwrap();
        assert_eq!(1, metas.len());

        assert_eq!(expect, metas[0]);
    }
    {
        let input = b"SNM\x0Ahelloworld";
        let expect = Metadata::SNM("helloworld".to_string());

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
    {
        let input  = b"RSD\x01\x20";
        let expect = Metadata::RSD(0x20);

        let metas = metadata::from_bytes(input).unwrap();
        assert_eq!(1, metas.len());

        assert_eq!(expect, metas[0]);
    }
    {
        let input  = b"RSP\x01\x20";
        let expect = Metadata::RSP(0x20);

        let metas = metadata::from_bytes(input).unwrap();
        assert_eq!(1, metas.len());

        assert_eq!(expect, metas[0]);
    }
}

#[test]
fn test_id_to_str() {
    use super::metadata::MetadataID::*;

    assert_eq!(metadata::id_to_str(FNM), "FNM");
    assert_eq!(metadata::id_to_str(SNM), "SNM");
    assert_eq!(metadata::id_to_str(FSZ), "FSZ");
    assert_eq!(metadata::id_to_str(FDT), "FDT");
    assert_eq!(metadata::id_to_str(SDT), "SDT");
    assert_eq!(metadata::id_to_str(HSH), "HSH");
    assert_eq!(metadata::id_to_str(RSD), "RSD");
    assert_eq!(metadata::id_to_str(RSP), "RSP");
}

#[test]
fn test_meta_to_id() {
    assert_eq!(metadata::meta_to_id(&Metadata::FNM("".to_string())), MetadataID::FNM);
    assert_eq!(metadata::meta_to_id(&Metadata::SNM("".to_string())), MetadataID::SNM);
    assert_eq!(metadata::meta_to_id(&Metadata::FSZ(0)),  MetadataID::FSZ);
    assert_eq!(metadata::meta_to_id(&Metadata::FDT(0)),  MetadataID::FDT);
    assert_eq!(metadata::meta_to_id(&Metadata::SDT(0)),  MetadataID::SDT);
    assert_eq!(metadata::meta_to_id(&Metadata::HSH((multihash::HashType::SHA1, Box::new([])))), MetadataID::HSH);
    assert_eq!(metadata::meta_to_id(&Metadata::RSD(0)),  MetadataID::RSD);
    assert_eq!(metadata::meta_to_id(&Metadata::RSP(0)),  MetadataID::RSP);
}

#[test]
fn test_get_meta_ref_by_id() {
    let metas = [Metadata::FNM("".to_string()),
                 Metadata::SNM("".to_string()),
                 Metadata::FSZ(0),
                 Metadata::FDT(0),
                 Metadata::SDT(0),
                 Metadata::HSH((multihash::HashType::SHA1, Box::new([]))),
                 Metadata::RSD(0),
                 Metadata::RSP(0)];

    assert_eq!(&Metadata::FNM("".to_string()), metadata::get_meta_ref_by_id(MetadataID::FNM, &metas).unwrap());
    assert_eq!(&Metadata::SNM("".to_string()), metadata::get_meta_ref_by_id(MetadataID::SNM, &metas).unwrap());
    assert_eq!(&Metadata::FSZ(0),  metadata::get_meta_ref_by_id(MetadataID::FSZ, &metas).unwrap());
    assert_eq!(&Metadata::FDT(0),  metadata::get_meta_ref_by_id(MetadataID::FDT, &metas).unwrap());
    assert_eq!(&Metadata::SDT(0),  metadata::get_meta_ref_by_id(MetadataID::SDT, &metas).unwrap());
    assert_eq!(&Metadata::HSH((multihash::HashType::SHA1, Box::new([]))), metadata::get_meta_ref_by_id(MetadataID::HSH, &metas).unwrap());
    assert_eq!(&Metadata::RSD(0),  metadata::get_meta_ref_by_id(MetadataID::RSD, &metas).unwrap());
    assert_eq!(&Metadata::RSP(0),  metadata::get_meta_ref_by_id(MetadataID::RSP, &metas).unwrap());
}

#[test]
fn test_get_meta_ref_mut_by_id() {
    let mut metas = [Metadata::FNM("".to_string()),
                     Metadata::SNM("".to_string()),
                     Metadata::FSZ(0),
                     Metadata::FDT(0),
                     Metadata::SDT(0),
                     Metadata::HSH((multihash::HashType::SHA1, Box::new([]))),
                     Metadata::RSD(0),
                     Metadata::RSP(0)];

    assert_eq!(&mut Metadata::FNM("".to_string()), metadata::get_meta_ref_mut_by_id(MetadataID::FNM, &mut metas).unwrap());
    assert_eq!(&mut Metadata::SNM("".to_string()), metadata::get_meta_ref_mut_by_id(MetadataID::SNM, &mut metas).unwrap());
    assert_eq!(&mut Metadata::FSZ(0),  metadata::get_meta_ref_mut_by_id(MetadataID::FSZ, &mut metas).unwrap());
    assert_eq!(&mut Metadata::FDT(0),  metadata::get_meta_ref_mut_by_id(MetadataID::FDT, &mut metas).unwrap());
    assert_eq!(&mut Metadata::SDT(0),  metadata::get_meta_ref_mut_by_id(MetadataID::SDT, &mut metas).unwrap());
    assert_eq!(&mut Metadata::HSH((multihash::HashType::SHA1, Box::new([]))), metadata::get_meta_ref_mut_by_id(MetadataID::HSH, &mut metas).unwrap());
    assert_eq!(&mut Metadata::RSD(0),  metadata::get_meta_ref_mut_by_id(MetadataID::RSD, &mut metas).unwrap());
    assert_eq!(&mut Metadata::RSP(0),  metadata::get_meta_ref_mut_by_id(MetadataID::RSP, &mut metas).unwrap());
}
