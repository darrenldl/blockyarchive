pub const SBX_LARGEST_BLOCK_SIZE   : usize = 4096;

pub const SBX_FIRST_DATA_SEQ_NUM   : usize = 1;

pub const SBX_METADATA_BLOCK_COUNT : usize = 1;

pub const SBX_SCAN_BLOCK_SIZE      : usize = 128;

pub const SBX_FILE_UID_LEN         : usize = common_params::FILE_UID_LEN;

pub const SBX_SIGNATURE            : &[u8] = common_params::SIGNATURE;

pub const SBX_HEADER_SIZE          : usize = common_params::HEADER_SIZE;

pub const SBX_MAX_DATA_BLOCK_COUNT : u32   =
    u32::max_value() - SBX_METADATA_BLOCK_COUNT as u32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Version {
    V1,
    V2,
    V3,
    V11,
    V12,
    V13,
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

pub fn ver_to_usize(version : Version) -> usize {
    use self::Version::*;
    match version {
        V1  => 1,
        V2  => 2,
        V3  => 3,
        V11 => 11,
        V12 => 12,
        V13 => 13,
    }
}

pub fn string_to_ver(string : &str) -> Result<Version, ()> {
    use self::Version::*;
    match string {
        "1"  => Ok(V1),
        "2"  => Ok(V2),
        "3"  => Ok(V3),
        "11" => Ok(V11),
        "12" => Ok(V12),
        "13" => Ok(V13),
        _    => Err(()),
    }
}

pub fn ver_to_block_size(version : Version) -> usize {
    use self::Version::*;
    match version {
        V1  => params_for_v1::BLOCK_SIZE,
        V2  => params_for_v2::BLOCK_SIZE,
        V3  => params_for_v3::BLOCK_SIZE,
        V11 => params_for_v11::BLOCK_SIZE,
        V12 => params_for_v12::BLOCK_SIZE,
        V13 => params_for_v13::BLOCK_SIZE,
    }
}

pub fn ver_to_data_size(version : Version) -> usize {
    use self::Version::*;
    match version {
        V1  => params_for_v1::DATA_SIZE,
        V2  => params_for_v2::DATA_SIZE,
        V3  => params_for_v3::DATA_SIZE,
        V11 => params_for_v11::DATA_SIZE,
        V12 => params_for_v12::DATA_SIZE,
        V13 => params_for_v13::DATA_SIZE,
    }
}

pub fn ver_uses_rs(version : Version) -> bool {
    use self::Version::*;
    match version {
        V1  | V2  | V3  => false,
        V11 | V12 | V13 => true,
    }
}

pub fn ver_forces_meta_enabled(version : Version) -> bool {
    use self::Version::*;
    match version {
        V1  | V2  | V3  => false,
        V11 | V12 | V13 => true,
    }
}

pub fn ver_to_max_data_file_size(version : Version) -> u64 {
    let data_block_size =
        ver_to_data_size(version) as u64;

    SBX_MAX_DATA_BLOCK_COUNT as u64 * data_block_size
}
