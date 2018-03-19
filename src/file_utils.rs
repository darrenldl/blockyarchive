#![allow(dead_code)]

use std::fs;
use general_error::Error;
use sbx_specs::{Version,
                ver_to_data_size,
                ver_to_block_size};

use file_reader::{FileReader,
                  FileReaderParam};

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
    pub fn calc_total_block_count_exc_burst_gaps(version        : Version,
                                                 data_par_burst : Option<(usize, usize, usize)>,
                                                 size           : u64)
                                                 -> u64 {
        let chunk_size  = ver_to_data_size(version) as u64;
        let data_chunks = (size + (chunk_size - 1)) / chunk_size;

        match data_par_burst {
            None                    => {
                let meta_block_count = 1;
                let data_block_count = data_chunks;

                meta_block_count + data_block_count
            },
            Some((data, parity, _)) => {
                let data_shards   = data   as u64;
                let parity_shards = parity as u64;

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
    }

    pub fn calc_container_size(version        : Version,
                               data_par_burst : Option<(usize, usize, usize)>,
                               size           : u64)
                               -> u64 {
        0
    }
}

pub fn get_file_name_part_of_path(path : &str) -> String {
    let path = Path::new(path);
    path.file_name().unwrap().to_string_lossy().to_string()
}
