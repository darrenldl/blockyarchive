#[derive(Clone, Copy, Debug)]
pub enum Version {
    V1,
    V2,
    V3
}

mod common_params {
    use std::u32;

    pub const FILE_UID_LEN   : usize = 6;
    pub const SIGNATURE      : &[u8] = b"SBx";
    pub const HEADER_SIZE    : usize = 16;
    pub const MAX_BLOCKS_NUM : u64   = u32::MAX as u64;
}

mod params_for_v1 {
    use super::common_params;

    pub const BLOCK_SIZE : usize = 512;
    pub const DATA_SIZE  : usize = BLOCK_SIZE - common_params::HEADER_SIZE;
}

mod params_for_v2 {
    use super::common_params;

    pub const BLOCK_SIZE : usize = 128;
    pub const DATA_SIZE  : usize = BLOCK_SIZE - common_params::HEADER_SIZE;
}

mod params_for_v3 {
    use super::common_params;

    pub const BLOCK_SIZE : usize = 4096;
    pub const DATA_SIZE  : usize = BLOCK_SIZE - common_params::HEADER_SIZE;
}

mod params_for_v11 {
    use super::params_for_v1;

    pub const BLOCK_SIZE : usize = params_for_v1::BLOCK_SIZE;
    pub const DATA_SIZE  : usize = params_for_v1::DATA_SIZE;
}

mod params_for_v12 {
    use super::params_for_v2;

    pub const BLOCK_SIZE : usize = params_for_v2::BLOCK_SIZE;
    pub const DATA_SIZE  : usize = params_for_v2::DATA_SIZE;
}

mod params_for_v13 {
    use super::params_for_v3;

    pub const BLOCK_SIZE : usize = params_for_v3::BLOCK_SIZE;
    pub const DATA_SIZE  : usize = params_for_v3::DATA_SIZE;
}

pub const SBX_FILE_UID_LEN : usize = common_params::FILE_UID_LEN;

pub const SBX_SIGNATURE    : &[u8] = common_params::SIGNATURE;

pub const SBX_HEADER_SIZE  : usize = common_params::HEADER_SIZE;

pub fn ver_to_usize (ver : Version) -> usize {
    match ver {
        Version::V1 => 1,
        Version::V2 => 2,
        Version::V3 => 3,
    }
}
