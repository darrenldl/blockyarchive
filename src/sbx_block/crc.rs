extern crate crc_ccitt;
use self::crc_ccitt::crc_ccitt_generic;

use super::super::sbx_specs;
use super::super::sbx_specs::{Version};

pub fn crc_ccitt(version : Version,
                 buffer  : &[u8]) -> u16 {
    crc_ccitt_generic(buffer, sbx_specs::ver_to_usize(version) as u16)
}
