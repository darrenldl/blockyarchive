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

pub type Block = (header::Header, Data);

