#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
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

pub fn hash_bytes_to_bytes(hash_bytes : &HashBytes, buffer : &mut [u8]) {
    let param        = specs::Param::new(hash_bytes.0);
    let digest_bytes = &hash_bytes.1;
    for i in 0..param.hash_func_type.len() {
        buffer[i] = param.hash_func_type[i];
    }

    buffer[param.hash_func_type.len()] = param.digest_length;

    let offset = param.hash_func_type.len() + 1;

    for i in 0..param.digest_length as usize {
        buffer[i + offset] = digest_bytes[i];
    }
}

pub fn hash_bytes_into_bytes(hash_bytes : &HashBytes) -> Box<[u8]> {
    let param      = specs::Param::new(hash_bytes.0);
    let mut buffer = vec![0; param.total_length()].into_boxed_slice();

    hash_bytes_to_bytes(hash_bytes, &mut buffer);
    buffer
}

pub mod specs {
    use super::*;

    #[derive(Copy, Clone)]
    pub struct Param {
        pub hash_func_type : &'static [u8],
        pub digest_length  : u8,
    }

    macro_rules! param {
        (
            $func_type:ident; $len:expr
        ) => {
            Param { hash_func_type : &$func_type,
                    digest_length  : $len }
        }
    }

    static SHA1_HFT        : [u8; 1] = [0x11      ];
    static SHA256_HFT      : [u8; 1] = [0x12      ];
    static SHA512_HFT      : [u8; 1] = [0x13      ];
    static BLAKE2B_256_HFT : [u8; 2] = [0xb2, 0x20];
    static BLAKE2B_512_HFT : [u8; 2] = [0xb2, 0x40];
    static BLAKE2S_128_HFT : [u8; 2] = [0xb2, 0x50];
    static BLAKE2S_256_HFT : [u8; 2] = [0xb2, 0x60];

    pub static SHA1_PARAM         : Param = param!(SHA1_HFT;        0x14);
    pub static SHA256_PARAM       : Param = param!(SHA256_HFT;      0x20);
    pub static SHA2_512_256_PARAM : Param = param!(SHA512_HFT;      0x20);
    pub static SHA512_PARAM       : Param = param!(SHA512_HFT;      0x40);
    pub static BLAKE2B_256_PARAM  : Param = param!(BLAKE2B_256_HFT; 0x20);
    pub static BLAKE2B_512_PARAM  : Param = param!(BLAKE2B_512_HFT; 0x40);
    pub static BLAKE2S_128_PARAM  : Param = param!(BLAKE2S_128_HFT; 0x10);
    pub static BLAKE2S_256_PARAM  : Param = param!(BLAKE2S_256_HFT; 0x20);

    impl Param {
        pub fn new(hash_type : HashType) -> Param {
            use super::HashType::*;
            match hash_type {
                SHA1                  => SHA1_PARAM,
                SHA2_256     | SHA256 => SHA256_PARAM,
                SHA2_512_256          => SHA2_512_256_PARAM,
                SHA2_512_512 | SHA512 => SHA512_PARAM,
                BLAKE2B_256           => BLAKE2B_256_PARAM,
                BLAKE2B_512           => BLAKE2B_512_PARAM,
                BLAKE2S_128           => BLAKE2S_128_PARAM,
                BLAKE2S_256           => BLAKE2S_256_PARAM,
            }
        }

        pub fn total_length(&self) -> usize {
            self.hash_func_type.len() + 1 + self.digest_length as usize
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
                        blake2b::State::new(specs::Param::new(hash_type).digest_length as usize))),
                HashType::BLAKE2B_512                     =>
                    Some(_Ctx::BLAKE2B_512(
                        blake2b::State::new(specs::Param::new(hash_type).digest_length as usize))),
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
            let mut hashval =
                vec![0; digest_len as usize]
                .into_boxed_slice();
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

pub mod parsers {
    use super::specs;
    use super::{HashBytes, HashType};
    use super::super::misc_utils;

    macro_rules! make_hash_parser_w_len {
        (
            $name:ident, $ht:path, $param:path
        ) => {
            named!($name <HashBytes>,
                   do_parse!(
                       _total_len : tag!(&[$param.total_length() as u8]) >>
                       _id : tag!($param.hash_func_type) >>
                           _n : tag!(&[$param.digest_length]) >>
                           res : take!($param.digest_length) >>
                           (($ht, misc_utils::slice_to_vec(res).into_boxed_slice()))
                   )
            );
        }
    }

    make_hash_parser_w_len!(sha1_w_len_p,
                            HashType::SHA1,         specs::SHA1_PARAM);
    make_hash_parser_w_len!(sha256_w_len_p,
                            HashType::SHA256,       specs::SHA256_PARAM);
    make_hash_parser_w_len!(sha2_512_256_w_len_p,
                            HashType::SHA2_512_256, specs::SHA2_512_256_PARAM);
    make_hash_parser_w_len!(sha512_w_len_p,
                            HashType::SHA512,       specs::SHA512_PARAM);
    make_hash_parser_w_len!(blake2b_256_w_len_p,
                            HashType::BLAKE2B_256,  specs::BLAKE2B_256_PARAM);
    make_hash_parser_w_len!(blake2b_512_w_len_p,
                            HashType::BLAKE2B_512,  specs::BLAKE2B_512_PARAM);
    make_hash_parser_w_len!(blake2s_128_w_len_p,
                            HashType::BLAKE2S_128,  specs::BLAKE2S_128_PARAM);
    make_hash_parser_w_len!(blake2s_256_w_len_p,
                            HashType::BLAKE2S_256,  specs::BLAKE2S_256_PARAM);

    named!(pub multihash_w_len_p <HashBytes>,
           alt!(sha1_w_len_p         |
                sha256_w_len_p       |
                sha2_512_256_w_len_p |
                sha512_w_len_p       |
                blake2b_256_w_len_p  |
                blake2b_512_w_len_p  |
                blake2s_128_w_len_p  |
                blake2s_256_w_len_p
           )
    );
}
