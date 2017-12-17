#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum HashType {
    SHA1,
    SHA2_256,     SHA256,
    SHA2_512_256,
    SHA2_512_512, SHA512,
    BLAKE2B_256,
    BLAKE2B_512,
    BLAKE2S_128,
    BLAKE2S_256
}

pub type HashBytes = (HashType, Box<[u8]>);

pub mod specs {
    use super::*;

    pub struct Param {
        pub hash_func_type : Box<[u8]>,
        pub digest_length  : usize,
        pub total_length   : usize
    }

    macro_rules! param {
        (
            [ $( $val:expr ),* ]; $len:expr
        ) => {
            Param { hash_func_type : Box::new([ $( $val ),* ]),
                    digest_length  : $len,
                    total_length   : $len + [ $( $val ),* ].len() }
        }
    }

    impl Param {
        pub fn new(hash_type : HashType) -> Param {
            use super::HashType::*;
            match hash_type {
                SHA1                  => param!([0x11      ]; 0x14),
                SHA2_256     | SHA256 => param!([0x12      ]; 0x20),
                SHA2_512_256          => param!([0x13      ]; 0x20),
                SHA2_512_512 | SHA512 => param!([0x13      ]; 0x40),
                BLAKE2B_256           => param!([0xb2, 0x20]; 0x20),
                BLAKE2B_512           => param!([0xb2, 0x40]; 0x40),
                BLAKE2S_128           => param!([0xb2, 0x50]; 0x10),
                BLAKE2S_256           => param!([0xb2, 0x60]; 0x20),
            }
        }
    }
}

pub mod hash {
    extern crate ring;
    extern crate blake2_c;

    use self::blake2_c::blake2b;

    use super::*;

    pub struct Ctx {
        ctx : _Ctx
    }

    #[allow(non_camel_case_types)]
    enum _Ctx {
        SHA1(ring::digest::Context),
        SHA256(ring::digest::Context),
        SHA512(ring::digest::Context),
        BLAKE2B_256(blake2b::State),
        BLAKE2B_512(blake2b::State)
    }

    impl Ctx {
        pub fn new(hash_type : HashType) -> Result<Ctx, ()> {
            let ctx = match hash_type {
                HashType::SHA1                            =>
                    Some(_Ctx::SHA1(
                        ring::digest::Context::new(&ring::digest::SHA1))),
                HashType::SHA2_256     | HashType::SHA256 =>
                    Some(_Ctx::SHA256(
                        ring::digest::Context::new(&ring::digest::SHA256))),
                HashType::SHA2_512_256                    => None,
                HashType::SHA2_512_512 | HashType::SHA512 =>
                    Some(_Ctx::SHA512(
                        ring::digest::Context::new(&ring::digest::SHA512))),
                HashType::BLAKE2B_256                     =>
                    Some(_Ctx::BLAKE2B_256(
                        blake2b::State::new(specs::Param::new(hash_type).digest_length))),
                HashType::BLAKE2B_512                     =>
                    Some(_Ctx::BLAKE2B_512(
                        blake2b::State::new(specs::Param::new(hash_type).digest_length))),
                HashType::BLAKE2S_128                     => None,
                HashType::BLAKE2S_256                     => None,
            };
            match ctx {
                Some(ctx) => Ok(Ctx { ctx }),
                None      => Err(())
            }
        }

        pub fn hash_type(&self) -> HashType {
            match self.ctx {
                _Ctx::SHA1(_)        => HashType::SHA1,
                _Ctx::SHA256(_)      => HashType::SHA256,
                _Ctx::SHA512(_)      => HashType::SHA512,
                _Ctx::BLAKE2B_256(_) => HashType::BLAKE2B_256,
                _Ctx::BLAKE2B_512(_) => HashType::BLAKE2B_512
            }
        }

        pub fn hash_type_is_supported(hash_type : HashType) -> bool {
            match Self::new(hash_type) {
                Ok(_)  => true,
                Err(_) => false
            }
        }

        pub fn update(&mut self, data : &[u8]) {
            match self.ctx {
                _Ctx::SHA1(ref mut ctx)        =>
                    ctx.update(data),
                _Ctx::SHA256(ref mut ctx)      =>
                    ctx.update(data),
                _Ctx::SHA512(ref mut ctx)      =>
                    ctx.update(data),
                _Ctx::BLAKE2B_256(ref mut ctx) => {
                    ctx.update(data); },
                _Ctx::BLAKE2B_512(ref mut ctx) => {
                    ctx.update(data); },
            }
        }

        pub fn finish_to_bytes(self, hashval : &mut [u8]) {
            match self.ctx {
                _Ctx::SHA1(ctx)            =>
                    hashval.copy_from_slice(ctx.finish().as_ref()),
                _Ctx::SHA256(ctx)          =>
                    hashval.copy_from_slice(ctx.finish().as_ref()),
                _Ctx::SHA512(ctx)          =>
                    hashval.copy_from_slice(ctx.finish().as_ref()),
                _Ctx::BLAKE2B_256(mut ctx) =>
                    hashval.copy_from_slice(ctx.finalize().bytes.as_slice()),
                _Ctx::BLAKE2B_512(mut ctx) =>
                    hashval.copy_from_slice(ctx.finalize().bytes.as_slice()),
            }
        }

        pub fn finish_into_bytes(self) -> Box<[u8]> {
            let hash_type   = self.hash_type();
            let param       = specs::Param::new(hash_type);
            let digest_len  = param.digest_length;
            let mut hashval = vec![0; digest_len].into_boxed_slice();
            self.finish_to_bytes(&mut hashval);
            hashval
        }

        pub fn finish_to_hash_bytes(self, hash_bytes : &mut HashBytes) {
            hash_bytes.0  = self.hash_type();
            self.finish_to_bytes(&mut hash_bytes.1);
        }

        pub fn finish_into_hash_bytes(self) -> HashBytes {
            (self.hash_type(), self.finish_into_bytes())
        }
    }
}

#[cfg(test)]
mod test_vectors {
    // SHA1, SHA256, SHA512 test vectors are copied from : https://www.di-mgt.com.au/sha_testvectors.html
    // BLAKE2B_256 test vectors are copied from : https://github.com/froydnj/ironclad/blob/master/testing/test-vectors/blake2-256.testvec
    // BLAKE2B_512 test vectors are copied from : https://github.com/froydnj/ironclad/blob/master/testing/test-vectors/blake2.testvec

    use super::super::misc_utils;
    use super::*;

    enum Data<'a> {
        Raw(&'a str),
        Hex(&'a str)
    }

    use self::Data::{Raw, Hex};
    use std::string::String;

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

    #[test]
    fn blake2b_256_test_vectors() {
        test_single_vector(HashType::BLAKE2B_256,
                           Raw("The lazy fox jumps over the lazy dog."),
                           Hex("0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8"));
    }

    #[test]
    fn blake2b_512_test_vectors() {
        test_single_vector(HashType::BLAKE2B_512,
                           Hex(""),
                           Hex("10ebb67700b1868efb4417987acf4690ae9d972fb7a590c2f02871799aaa4786b5e996e8f0f4eb981fc214b005f42d2ff4233499391653df7aefcbc13fc51568"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("00"),
                           Hex("961f6dd1e4dd30f63901690c512e78e4b45e4742ed197c3c5e45c549fd25f2e4187b0bc9fe30492b16b0d0bc4ef9b0f34c7003fac09a5ef1532e69430234cebd"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("0001"),
                           Hex("da2cfbe2d8409a0f38026113884f84b50156371ae304c4430173d08a99d9fb1b983164a3770706d537f49e0c916d9f32b95cc37a95b99d857436f0232c88a965"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("000102"),
                           Hex("33d0825dddf7ada99b0e7e307104ad07ca9cfd9692214f1561356315e784f3e5a17e364ae9dbb14cb2036df932b77f4b292761365fb328de7afdc6d8998f5fc1"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("00010203"),
                           Hex("beaa5a3d08f3807143cf621d95cd690514d0b49efff9c91d24b59241ec0eefa5f60196d407048bba8d2146828ebcb0488d8842fd56bb4f6df8e19c4b4daab8ac"));
        test_single_vector(HashType::BLAKE2B_512,
                           Hex("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfe"),
                           Hex("142709d62e28fcccd0af97fad0f8465b971e82201dc51070faa0372aa43e92484be1c1e73ba10906d5d1853db6a4106e0a7bf9800d373d6dee2d46d62ef2a461"));
    }
}
