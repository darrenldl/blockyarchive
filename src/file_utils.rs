use std::fs;
use super::Error;
use super::sbx_specs::Version;
use super::sbx_specs::ver_to_data_size;
use super::sbx_specs::ver_to_block_size;

use super::file_reader::FileReader;
use super::file_reader::FileReaderParam;

use std::path::Path;

pub fn get_file_metadata(file : &str) -> Result<fs::Metadata, Error> {
    let reader = FileReader::new(file,
                                 FileReaderParam { write    : false,
                                                   buffered : false  })?;
    reader.metadata()
}

pub fn get_file_size(file : &str) -> Result<u64, Error> {
    let metadata = get_file_metadata(file)?;

    Ok(metadata.len())
}

pub fn check_if_file_exists(file : &str) -> bool {
    Path::new(file).exists()
}

pub fn check_if_file_is_dir(file : &str) -> bool {
    Path::new(file).is_dir()
}

pub fn calc_data_chunk_count(version  : Version,
                             metadata : &fs::Metadata) -> u64 {
    let data_size = ver_to_data_size(version) as u64;
    ((metadata.len() + (data_size - 1)) / data_size)
}

pub fn calc_total_block_count(version  : Version,
                              metadata : &fs::Metadata) -> u64 {
    let block_size = ver_to_block_size(version) as u64;
    ((metadata.len() + (block_size - 1)) / block_size)
}
