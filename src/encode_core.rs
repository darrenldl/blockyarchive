use super::Error;
use super::sbx_specs;

use super::time;

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    sbx_version         : sbx_specs::Version,
    meta_blocks_written : u64,
    data_blocks_written : u64,
    data_bytes_encoded  : u64,
    start_time          : u64,
}

pub fn encode_file() -> Result<(), Error> {
    Ok(())
}
