#![cfg(test)]

use sbx_specs::Version;
use file_utils::*;

#[test]
fn test_calc_meta_block_count_exc_burst_gaps_rs_disabled() {
    assert_eq!(0, calc_meta_block_count_exc_burst_gaps(Version::V1, Some(false), None));
    assert_eq!(0, calc_meta_block_count_exc_burst_gaps(Version::V2, Some(false), None));
    assert_eq!(0, calc_meta_block_count_exc_burst_gaps(Version::V3, Some(false), None));
}

quickcheck! {
    fn qc_calc_meta_block_count_exc_burst_gaps_rs_enabled(meta_enabled   : Option<bool>,
                                                          data_par_burst : (usize, usize, usize)) -> bool {
        let (_, parity, _) = data_par_burst;

        (1 + parity as u64) == calc_meta_block_count_exc_burst_gaps(Version::V17, meta_enabled, Some(data_par_burst))
            && (1 + parity as u64) == calc_meta_block_count_exc_burst_gaps(Version::V18, meta_enabled, Some(data_par_burst))
            && (1 + parity as u64) == calc_meta_block_count_exc_burst_gaps(Version::V19, meta_enabled, Some(data_par_burst))
    }
}

mod from_orig_file_size {
    use file_utils::from_orig_file_size::*;
    use sbx_specs::*;

    quickcheck! {
        fn qc_calc_data_only_and_parity_block_count_exc_burst_gaps_rs_disabled(size : u64) -> bool {
            ({
                let version = Version::V1;

                let (data, parity) =
                    calc_data_only_and_parity_block_count_exc_burst_gaps(version, None, size);

                let data_size = ver_to_data_size(version) as u64;

                parity == 0
                    && data * data_size >= size
                    && ((size % data_size == 0 && data * data_size == size)
                        || (size % data_size != 0 && data * data_size > size))
                    && (data == (size + (data_size - 1)) / data_size)
            })
                &&
                ({
                    let version = Version::V2;

                    let (data, parity) =
                        calc_data_only_and_parity_block_count_exc_burst_gaps(version, None, size);

                    let data_size = ver_to_data_size(version) as u64;

                    parity == 0
                        && data * data_size >= size
                        && ((size % data_size == 0 && data * data_size == size)
                            || (size % data_size != 0 && data * data_size > size))
                        && (data == (size + (data_size - 1)) / data_size)
                })
                &&
                ({
                    let version = Version::V3;

                    let (data, parity) =
                        calc_data_only_and_parity_block_count_exc_burst_gaps(version, None, size);

                    let data_size = ver_to_data_size(version) as u64;

                    parity == 0
                        && data * data_size >= size
                        && ((size % data_size == 0 && data * data_size == size)
                            || (size % data_size != 0 && data * data_size > size))
                        && (data == (size + (data_size - 1)) / data_size)
                })
        }
    }
}
