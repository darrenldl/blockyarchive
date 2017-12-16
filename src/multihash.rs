#[allow(non_camel_case_types)]
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
        hash_func_type : Box<[u8]>,
        digest_length  : usize
    }

    macro_rules! param {
        (
            [ $( $val:expr ),* ]; $len:expr
        ) => {
            Param { hash_func_type : Box::new([ $( $val ),* ]),
                    digest_length  : $len }
        }
    }

    pub fn hash_type_to_param (hash_type : &HashType) -> Param {
        use super::HashType::*;
        match *hash_type {
            SHA1                  => param!([0x11]; 0x14),
            SHA2_256     | SHA256 => param!([0x12]; 0x20),
            SHA2_512_256          => param!([0x13]; 0x20),
            SHA2_512_512 | SHA512 => param!([0x13]; 0x40),
            BLAKE2B_256           => param!([0xb2, 0x20]; 0x20),
            BLAKE2B_512           => param!([0xb2, 0x40]; 0x40),
            BLAKE2S_128           => param!([0xb2, 0x50]; 0x10),
            BLAKE2S_256           => param!([0xb2, 0x60]; 0x20),
        }
    }
}
