mod helper;
mod header;
mod metadata;

use super::sbx_specs;

pub enum Data {
    Raw(Box<[u8]>),
    Shard(reed_solomon::Shard),
    Meta(Vec<metadata::Metadata>)
}

pub type Block = (Header, Data)
