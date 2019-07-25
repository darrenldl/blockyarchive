#![allow(dead_code)]
use crate::file_reader::{FileReader, FileReaderParam};
use crate::general_error::Error;
use crate::sbx_block;
use crate::sbx_specs::{ver_to_block_size, ver_to_data_size, ver_uses_rs, Version};
use std::fs;
use std::path::Path;

pub fn get_file_metadata(file: &str) -> Result<fs::Metadata, Error> {
    let reader = FileReader::new(
        file,
        FileReaderParam {
            write: false,
            buffered: false,
        },
    )?;
    reader.metadata()
}

pub fn get_file_size(file: &str) -> Result<u64, Error> {
    let mut reader = FileReader::new(
        file,
        FileReaderParam {
            write: false,
            buffered: false,
        },
    )?;

    Ok(reader.get_file_size()?)
}

pub fn check_if_file_exists(file: &str) -> bool {
    Path::new(file).exists()
}

pub fn check_if_file_is_file(file: &str) -> bool {
    Path::new(file).is_file()
}

pub fn check_if_file_is_dir(file: &str) -> bool {
    Path::new(file).is_dir()
}

pub fn check_if_file_is_stdin(file: &str) -> bool {
    file == "-"
}

pub fn check_if_file_is_stdout(file: &str) -> bool {
    file == "-"
}

pub fn calc_meta_block_count_exc_burst_gaps(
    version: Version,
    meta_enabled: Option<bool>,
    data_par_burst: Option<(usize, usize, usize)>,
) -> u64 {
    match data_par_burst {
        None => {
            assert!(!ver_uses_rs(version));

            let meta_enabled = meta_enabled.unwrap_or(true);

            if meta_enabled {
                1
            } else {
                0
            }
        }
        Some((_, parity, _)) => {
            assert!(ver_uses_rs(version));

            1 + parity as u64
        }
    }
}

pub mod from_container_size {
    use super::*;
    pub fn calc_total_block_count(version: Version, size: u64) -> u64 {
        let block_size = ver_to_block_size(version) as u64;
        ((size + (block_size - 1)) / block_size)
    }
}

pub mod from_orig_file_size {
    use super::*;

    pub fn calc_data_chunk_count(version: Version, size: u64) -> u64 {
        let data_size = ver_to_data_size(version) as u64;
        ((size + (data_size - 1)) / data_size)
    }

    pub fn calc_data_only_and_parity_block_count_exc_burst_gaps(
        version: Version,
        data_par_burst: Option<(usize, usize, usize)>,
        size: u64,
    ) -> (u64, u64) {
        let chunk_size = ver_to_data_size(version) as u64;
        let data_chunks = (size + (chunk_size - 1)) / chunk_size;

        match data_par_burst {
            None => {
                assert!(!ver_uses_rs(version));

                let data_block_count = data_chunks;

                (data_block_count, 0)
            }
            Some((data, parity, _)) => {
                assert!(ver_uses_rs(version));

                let data_shards = data as u64;
                let parity_shards = parity as u64;

                let data_block_set_size = data_shards;
                let data_block_set_count =
                    (data_chunks + (data_block_set_size - 1)) / data_block_set_size;

                let encoded_data_block_set_count = data_block_set_count;

                (
                    encoded_data_block_set_count * data_shards,
                    encoded_data_block_set_count * parity_shards,
                )
            }
        }
    }

    pub fn calc_data_block_count_exc_burst_gaps(
        version: Version,
        data_par_burst: Option<(usize, usize, usize)>,
        size: u64,
    ) -> u64 {
        let (data, parity) =
            calc_data_only_and_parity_block_count_exc_burst_gaps(version, data_par_burst, size);

        data + parity
    }

    pub fn calc_total_block_count_exc_burst_gaps(
        version: Version,
        meta_enabled: Option<bool>,
        data_par_burst: Option<(usize, usize, usize)>,
        size: u64,
    ) -> u64 {
        let meta_block_count =
            calc_meta_block_count_exc_burst_gaps(version, meta_enabled, data_par_burst);

        let data_block_count = calc_data_block_count_exc_burst_gaps(version, data_par_burst, size);

        meta_block_count + data_block_count
    }

    pub fn calc_container_size(
        version: Version,
        meta_enabled: Option<bool>,
        data_par_burst: Option<(usize, usize, usize)>,
        size: u64,
    ) -> u64 {
        let block_size = ver_to_block_size(version) as u64;

        match data_par_burst {
            None => {
                let block_count = calc_total_block_count_exc_burst_gaps(
                    version,
                    meta_enabled,
                    data_par_burst,
                    size,
                );

                block_size * block_count
            }
            Some((data, parity, burst)) => {
                if burst == 0 {
                    let block_count = calc_total_block_count_exc_burst_gaps(
                        version,
                        meta_enabled,
                        data_par_burst,
                        size,
                    );

                    return block_size * block_count;
                }

                let data_block_count =
                    calc_data_block_count_exc_burst_gaps(version, data_par_burst, size);

                if data_block_count == 0 {
                    let write_pos_s =
                        sbx_block::calc_meta_block_all_write_pos_s(version, data_par_burst);

                    let last_write_pos = write_pos_s[write_pos_s.len() - 1];

                    return last_write_pos + block_size;
                }

                let last_seq_num = data_block_count as u32;

                let last_data_index = last_seq_num - 1;

                let super_block_set_size = ((data + parity) * burst) as u32;

                let first_data_index_in_last_block_set =
                    (last_data_index / super_block_set_size) * super_block_set_size;

                let first_seq_num_in_last_block_set = first_data_index_in_last_block_set + 1;

                let mut last_index = 0;
                let mut seq_num = first_seq_num_in_last_block_set;
                while seq_num <= last_seq_num {
                    let index = sbx_block::calc_data_block_write_index(
                        seq_num,
                        meta_enabled,
                        data_par_burst,
                    );

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

pub fn get_file_name_part_of_path(path: &str) -> Option<String> {
    let path = Path::new(path);
    match path.file_name() {
        Some(s) => Some(s.to_string_lossy().to_string()),
        None => None,
    }
}
