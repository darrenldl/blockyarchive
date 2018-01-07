#![cfg(test)]
mod test_vectors {
    // SHA1, SHA256, SHA512 test vectors are copied from : https://www.di-mgt.com.au/sha_testvectors.html
    // BLAKE2B_512 test vectors are copied from : https://raw.githubusercontent.com/BLAKE2/BLAKE2/master/testvectors/blake2-kat.json

    use super::super::misc_utils;
    use super::*;

    enum Data<'a> {
        Raw(&'a str),
        Hex(&'a str)
    }

    use self::Data::{Raw, Hex};
    //use std::string::String;

    fn test_single_vector(hash_type : HashType, input : Data, expect : Data) {
        let input  : Box<[u8]> = match input {
            Raw(x) => x.to_string().into_bytes().into_boxed_slice(),
            Hex(x) => misc_utils::hex_string_to_bytes(x).unwrap()
        };
        let expect : Box<[u8]> = match expect {
            Raw(x) => x.to_string().into_bytes().into_boxed_slice(),
            Hex(x) => misc_utils::hex_string_to_bytes(x).unwrap()
        };
        let mut ctx = hash::Ctx::new(hash_type).unwrap();
        ctx.update(input.as_ref());
        let result = ctx.finish_into_bytes();

        assert_eq!(expect, result);
    }

    fn test_single_multihash(hash_type : HashType,
                             input     : Data,
                             expect    : Data) {
        let input  : Box<[u8]> = match input {
            Raw(x) => x.to_string().into_bytes().into_boxed_slice(),
            Hex(x) => misc_utils::hex_string_to_bytes(x).unwrap()
        };
        let expect : Box<[u8]> = match expect {
            Raw(x) => x.to_string().into_bytes().into_boxed_slice(),
            Hex(x) => misc_utils::hex_string_to_bytes(x).unwrap()
        };
        let mut ctx = hash::Ctx::new(hash_type).unwrap();
        ctx.update(input.as_ref());
        let result_bytes =
            hash_bytes_into_bytes(&ctx.finish_into_hash_bytes());

        assert_eq!(expect, result_bytes);
    }

    fn test_repetition(hash_type : HashType, input : Data, repeat : usize, expect : Data) {
        let input  = match input {
            Raw(x) => x.to_string().into_bytes().into_boxed_slice(),
            Hex(x) => misc_utils::hex_string_to_bytes(x).unwrap()
        };
        let expect = match expect {
            Raw(x) => x.to_string().into_bytes().into_boxed_slice(),
            Hex(x) => misc_utils::hex_string_to_bytes(x).unwrap()
        };
        let mut ctx = hash::Ctx::new(hash_type).unwrap();
        for _ in 0..repeat {
            ctx.update(input.as_ref());
        }
        let result = ctx.finish_into_bytes();

        assert_eq!(expect, result);
    }

    #[test]
    fn multihash_sha1_bytes() {
        test_single_multihash(HashType::SHA1,
                              Raw(""),
                              Hex("1114da39a3ee5e6b4b0d3255bfef95601890afd80709"));
    }

    #[test]
    fn sha1_test_vectors() {
        test_single_vector(HashType::SHA1,
                           Raw("abc"),
                           Hex("a9993e364706816aba3e25717850c26c9cd0d89d"));
        test_single_vector(HashType::SHA1,
                           Raw(""),
                           Hex("da39a3ee5e6b4b0d3255bfef95601890afd80709"));
        test_single_vector(HashType::SHA1,
                           Raw("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
                           Hex("84983e441c3bd26ebaae4aa1f95129e5e54670f1"));
        test_single_vector(HashType::SHA1,
                           Raw("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"),
                           Hex("a49b2446a02c645bf419f995b67091253a04a259"));
        test_repetition(HashType::SHA1,
                        Raw("a"),
                        1_000_000,
                        Hex("34aa973cd4c4daa4f61eeb2bdbad27316534016f"));
    }

    #[test]
    fn multihash_sha256_bytes() {
        test_single_multihash(HashType::SHA256,
                              Raw(""),
                              Hex("1220e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"));
    }

    #[test]
    fn sha256_test_vectors() {
        test_single_vector(HashType::SHA256,
                           Raw("abc"),
                           Hex("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
        test_single_vector(HashType::SHA256,
                           Raw(""),
                           Hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"));
        test_single_vector(HashType::SHA256,
                           Raw("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
                           Hex("248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"));
        test_single_vector(HashType::SHA256,
                           Raw("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"),
                           Hex("cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"));
        test_repetition(HashType::SHA256,
                        Raw("a"),
                        1_000_000,
                        Hex("cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"));
    }

    #[test]
    fn multihash_sha512_bytes() {
        test_single_multihash(HashType::SHA512,
                              Raw(""),
                              Hex("1340cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"));
    }

    #[test]
    fn sha512_test_vectors() {
        test_single_vector(HashType::SHA512,
                           Raw("abc"),
                           Hex("ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"));
        test_single_vector(HashType::SHA512,
                           Raw(""),
                           Hex("cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"));
        test_single_vector(HashType::SHA512,
                           Raw("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
                           Hex("204a8fc6dda82f0a0ced7beb8e08a41657c16ef468b228a8279be331a703c33596fd15c13b1b07f9aa1d3bea57789ca031ad85c7a71dd70354ec631238ca3445"));
        test_single_vector(HashType::SHA512,
                           Raw("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"),
                           Hex("8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018501d289e4900f7e4331b99dec4b5433ac7d329eeb6dd26545e96e55b874be909"));
        test_repetition(HashType::SHA512,
                        Raw("a"),
                        1_000_000,
                        Hex("e718483d0ce769644e2e42c7bc15b4638e1f98b13b2044285632a803afa973ebde0ff244877ea60a4cb0432ce577c31beb009c5c2c49aa2e4eadb217ad8cc09b"));
    }

    /*#[test]
    fn blake2b_256_test_vectors() {
        test_single_vector(HashType::BLAKE2B_256,
                           Hex(""),
                           Hex(""));
    }*/

    #[test]
    fn multihash_blake2b_512_bytes() {
        test_single_multihash(HashType::BLAKE2B_512,
                              Hex(""),
                              Hex("b24040786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce"));
    }

    #[test]
    fn blake2b_512_test_vectors() {
        test_single_vector(HashType::BLAKE2B_512,
                           Hex(""),
                           Hex("786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("00"),
                           Hex("2fa3f686df876995167e7c2e5d74c4c7b6e48f8068fe0e44208344d480f7904c36963e44115fe3eb2a3ac8694c28bcb4f5a0f3276f2e79487d8219057a506e4b"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("0001"),
                           Hex("1c08798dc641aba9dee435e22519a4729a09b2bfe0ff00ef2dcd8ed6f8a07d15eaf4aee52bbf18ab5608a6190f70b90486c8a7d4873710b1115d3debbb4327b5"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("000102"),
                           Hex("40a374727302d9a4769c17b5f409ff32f58aa24ff122d7603e4fda1509e919d4107a52c57570a6d94e50967aea573b11f86f473f537565c66f7039830a85d186"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("00010203"),
                           Hex("77ddf4b14425eb3d053c1e84e3469d92c4cd910ed20f92035e0c99d8a7a86cecaf69f9663c20a7aa230bc82f60d22fb4a00b09d3eb8fc65ef547fe63c8d3ddce"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfe"),
                           Hex("5b21c5fd8868367612474fa2e70e9cfa2201ffeee8fafab5797ad58fefa17c9b5b107da4a3db6320baaf2c8617d5a51df914ae88da3867c2d41f0cc14fa67928"));
    }
}
