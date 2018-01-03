use std;
use super::super::sbx_specs::{Version, SBX_HEADER_SIZE, SBX_SIGNATURE};
use super::super::sbx_specs;

use super::crc::*;

#[derive(Debug, Clone)]
pub struct Header {
    pub version  : Version,
    pub crc      : u16,
    pub file_uid : [u8; SBX_HEADER_SIZE],
    pub seq_num  : u32
}

impl Header {
    pub fn new(version   : Version,
               file_uid : [u8; SBX_HEADER_SIZE]) -> Header {
        Header {
            version,
            crc       : 0,
            file_uid,
            seq_num   : 0
        }
    }

    pub fn write_to_bytes(&self, buffer : &mut [u8]) {
        { // signature
            buffer[0..3].copy_from_slice(SBX_SIGNATURE); }
        { // version byte
            buffer[3] = sbx_specs::ver_to_usize(self.version) as u8; }
        { // crc ccitt
            let crc : [u8; 2] =
                unsafe { std::mem::transmute::<u16, [u8; 2]>(self.crc) };
            buffer[4..6].copy_from_slice(&crc); }
        { // file uid
            buffer[6..12].copy_from_slice(&self.file_uid); }
        { // seq num
            let seq_num : [u8; 4] =
                unsafe { std::mem::transmute::<u32, [u8; 4]>(self.seq_num) };
            buffer[12..16].copy_from_slice(&seq_num); }
    }

    pub fn crc_ccitt(&self) -> u16 {
        let crc = sbx_crc_ccitt(self.version, &self.file_uid);
        let seq_num : [u8; 4] =
            unsafe { std::mem::transmute::<u32, [u8; 4]>(self.seq_num) };
        crc_ccitt_generic(crc, &seq_num)
    }
}
