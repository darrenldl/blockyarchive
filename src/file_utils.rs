#![allow(dead_code)]

use std::fs;
use super::Error;
use super::sbx_specs::Version;
use super::sbx_specs::ver_to_data_size;
use super::sbx_specs::ver_to_block_size;
use super::sbx_specs::ver_uses_rs;

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

pub mod from_raw_file_metadata {
    use super::*;
    pub fn calc_data_chunk_count(version  : Version,
                                 metadata : &fs::Metadata) -> u64 {
        let data_size = ver_to_data_size(version) as u64;
        ((metadata.len() + (data_size - 1)) / data_size)
    }
}

pub mod from_container_metadata {
    use super::*;
    pub fn calc_total_block_count(version  : Version,
                                  metadata : &fs::Metadata) -> u64 {
        let block_size = ver_to_block_size(version) as u64;
        ((metadata.len() + (block_size - 1)) / block_size)
    }
}

pub mod from_orig_file_size {
    use super::*;
    pub fn calc_rs_enabled_total_block_count(version       : Version,
                                             data_shards   : usize,
                                             parity_shards : usize,
                                             size          : u64)
                                             -> u64 {
        assert!(ver_uses_rs(version));

        let data_shards   = data_shards   as u64;
        let parity_shards = parity_shards as u64;

        let chunk_size  = ver_to_data_size(version) as u64;
        let data_chunks = (size + (chunk_size - 1)) / chunk_size;

        let meta_block_count = 1 + parity_shards as u64;

        let data_block_set_size  = data_shards;
        let data_block_set_count =
            (data_chunks + (data_block_set_size - 1)) / data_block_set_size;

        let encoded_data_block_set_size  = data_shards + parity_shards;
        let encoded_data_block_set_count = data_block_set_count;

        meta_block_count
            + encoded_data_block_set_count * encoded_data_block_set_size
    }
}
