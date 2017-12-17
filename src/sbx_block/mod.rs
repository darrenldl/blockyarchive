mod helper;
mod header;
mod metadata;

use super::sbx_specs;
extern crate reed_solomon_erasure;

pub enum Data {
    Raw(Box<[u8]>),
    Shard(reed_solomon_erasure::Shard),
    Meta(Vec<metadata::Metadata>)
}

pub struct Block {
    header : header::Header,
    data   : Data
}

impl Block {
    /*pub fn new(version : Version,
               crc_ccitt : u16)*/
}
