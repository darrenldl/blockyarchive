extern crate crcccitt;

use super::super::sbx_specs::{Version, ver_to_usize};

use crcccitt::crc_ccitt_generic;

pub fn crc_ccitt_sbx(ver : Version, input : &[u8]) -> u16 {
    crc_ccitt_generic(input, ver_to_usize(ver) as u16)
}
