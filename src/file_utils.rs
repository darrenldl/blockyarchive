use std::fs;
use super::Error;
use super::sbx_specs::Version;
use super::sbx_specs::ver_to_data_size;

use super::file_reader::FileReader;

pub fn get_file_metadata(file : &str) -> Result<fs::Metadata, Error> {
    let reader = FileReader::new(file)?;
    reader.metadata()
}

pub fn calc_block_count(version  : Version,
                        metadata : &fs::Metadata) -> u64 {
    let data_size = ver_to_data_size(version) as u64;
    ((metadata.len() + (data_size - 1)) / data_size)
}
