#![allow(dead_code)]

use std::fs;
use general_error::Error;
use sbx_specs::{Version,
                ver_to_data_size,
                ver_uses_rs,
                ver_to_block_size};

use file_reader::{FileReader,
                  FileReaderParam};

use std::path::Path;

use sbx_block;

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

    pub fn calc_data_only_and_parity_block_count_exc_burst_gaps(version : Version,
                                                                data_par_burst : Option<(usize, usize, usize)>,
                                                                size           : u64)
                                                                -> (u64, u64) {
        let chunk_size  = ver_to_data_size(version) as u64;
        let data_chunks = (size + (chunk_size - 1)) / chunk_size;

        match data_par_burst {
            None                    => {
                assert!(!ver_uses_rs(version));

                let data_block_count = data_chunks;

                (data_block_count, 0)
            },
            Some((data, parity, _)) => {
                assert!( ver_uses_rs(version));

                let data_shards   = data   as u64;
                let parity_shards = parity as u64;

                let data_block_set_size  = data_shards;
                let data_block_set_count =
                    (data_chunks + (data_block_set_size - 1)) / data_block_set_size;

                let encoded_data_block_set_count = data_block_set_count;

                (encoded_data_block_set_count * data_shards,
                 encoded_data_block_set_count * parity_shards)
            }
        }
    }

    pub fn calc_data_block_count_exc_burst_gaps(version        : Version,
                                                data_par_burst : Option<(usize, usize, usize)>,
                                                size           : u64)
                                                -> u64 {
        let (data, parity) =
            calc_data_only_and_parity_block_count_exc_burst_gaps(version,
                                                                 data_par_burst,
                                                                 size);

        data + parity
    }

    pub fn calc_total_block_count_exc_burst_gaps(version        : Version,
                                                 data_par_burst : Option<(usize, usize, usize)>,
                                                 size           : u64)
                                                 -> u64 {
        let data_block_count =
            calc_data_block_count_exc_burst_gaps(version,
                                                 data_par_burst,
                                                 size);

        match data_par_burst {
            None                 => {
                let meta_block_count = 1;
                let data_block_count =
                    calc_data_block_count_exc_burst_gaps(version,
                                                         data_par_burst,
                                                         size);

                meta_block_count + data_block_count
            },
            Some((_, parity, _)) => {
                let parity_shards = parity as u64;

                let meta_block_count = 1 + parity_shards as u64;

                meta_block_count + data_block_count
            }
        }
    }

    pub fn calc_container_size(version        : Version,
                               data_par_burst : Option<(usize, usize, usize)>,
                               size           : u64)
                               -> u64 {
        let block_size = ver_to_block_size(version) as u64;

        match data_par_burst {
            None                        => {
                let block_count =
                    calc_total_block_count_exc_burst_gaps(version,
                                                          data_par_burst,
                                                          size);

                block_size * block_count
            },
            Some((data, parity, burst)) => {
                if burst == 0 {
                    let block_count =
                        calc_total_block_count_exc_burst_gaps(version,
                                                              data_par_burst,
                                                              size);

                    return block_size * block_count;
                }

                let data_block_count =
                    calc_data_block_count_exc_burst_gaps(version,
                                                         data_par_burst,
                                                         size);

                if data_block_count == 0 {
                    let mut max_file_size = 0;
                    for &p in sbx_block::calc_meta_block_all_write_pos_s(version,
                                                                         data_par_burst).iter()
                    {
                        let file_size = p + block_size;

                        if file_size > max_file_size {
                            max_file_size = file_size;
                        }
                    }

                    return max_file_size;
                }

                let last_seq_num = data_block_count as u32;

                let last_data_index = last_seq_num - 1;

                let super_block_set_size = ((data + parity) * burst) as u32;

                let first_data_index_in_last_block_set =
                    (last_data_index / super_block_set_size) * super_block_set_size;

                let first_seq_num_in_last_block_set =
                    first_data_index_in_last_block_set + 1;

                let mut last_index = 0;
                let mut seq_num = first_seq_num_in_last_block_set;
                while seq_num <= last_seq_num {
                    let index =
                        sbx_block::calc_data_block_write_index(seq_num,
                                                               data_par_burst);

                    if index > last_index {
                        last_index = index;
                    }

                    seq_num += 1;
                }

                block_size * (last_index + 1)
            }
        }
    }
}

pub fn get_file_name_part_of_path(path : &str) -> String {
    let path = Path::new(path);
    path.file_name().unwrap().to_string_lossy().to_string()
}
