#![cfg(test)]

use file_utils::*;
use sbx_specs::Version;

#[test]
fn test_calc_meta_block_count_exc_burst_gaps_rs_disabled() {
    assert_eq!(
        0,
        calc_meta_block_count_exc_burst_gaps(Version::V1, Some(false), None)
    );
    assert_eq!(
        0,
        calc_meta_block_count_exc_burst_gaps(Version::V2, Some(false), None)
    );
    assert_eq!(
        0,
        calc_meta_block_count_exc_burst_gaps(Version::V3, Some(false), None)
    );
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
    use file_utils::*;
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

        fn qc_calc_data_only_and_parity_block_count_exc_burst_gaps_rs_enabled(data_par_burst : (usize, usize, usize),
                                                                              size           : u64)
                                                                              -> bool {
            let mut data_par_burst = data_par_burst;
            data_par_burst.0 = if data_par_burst.0 == 0 { 1 } else { data_par_burst.0 };
            data_par_burst.1 = if data_par_burst.1 == 0 { 1 } else { data_par_burst.1 };

            let (data, parity, _) = data_par_burst;
            let data = data as u64;
            let parity = parity as u64;

            ({
                let version = Version::V17;

                let (data_total, parity_total) =
                    calc_data_only_and_parity_block_count_exc_burst_gaps(version, Some(data_par_burst), size);

                let data_size = ver_to_data_size(version) as u64;

                let data_chunks = (size + (data_size - 1)) / data_size;

                let block_set_size = data + parity;

                let block_set_count = (data_chunks + (block_set_size - 1)) / block_set_size;

                data_total * data_size >= size
                    && data_total % data == 0
                    && parity_total % parity == 0
                    && data_total / data == parity_total / parity
                    && data_total / data == block_set_count
            })
                &&
                ({
                    let version = Version::V18;

                    let (data_total, parity_total) =
                        calc_data_only_and_parity_block_count_exc_burst_gaps(version, Some(data_par_burst), size);

                    let data_size = ver_to_data_size(version) as u64;

                    let data_chunks = (size + (data_size - 1)) / data_size;

                    let block_set_size = data + parity;

                    let block_set_count = (data_chunks + (block_set_size - 1)) / block_set_size;

                    data_total * data_size >= size
                        && data_total % data == 0
                        && parity_total % parity == 0
                        && data_total / data == parity_total / parity
                        && data_total / data == block_set_count
                })
                &&
                ({
                    let version = Version::V19;

                    let (data_total, parity_total) =
                        calc_data_only_and_parity_block_count_exc_burst_gaps(version, Some(data_par_burst), size);

                    let data_size = ver_to_data_size(version) as u64;

                    let data_chunks = (size + (data_size - 1)) / data_size;

                    let block_set_size = data + parity;

                    let block_set_count = (data_chunks + (block_set_size - 1)) / block_set_size;

                    data_total * data_size >= size
                        && data_total % data == 0
                        && parity_total % parity == 0
                        && data_total / data == parity_total / parity
                        && data_total / data == block_set_count
                })
        }
        fn qc_calc_data_block_count_exc_burst_gaps_consistent_rs_disabled(size : u64)
                                                                          -> bool {
            ({
                let version = Version::V1;

                let (data_total, parity_total) =
                    calc_data_only_and_parity_block_count_exc_burst_gaps(version, None, size);

                let data_all_total =
                    calc_data_block_count_exc_burst_gaps(version, None, size);

                data_all_total == data_total + parity_total
            })
                &&
                ({
                    let version = Version::V2;

                    let (data_total, parity_total) =
                        calc_data_only_and_parity_block_count_exc_burst_gaps(version, None, size);

                    let data_all_total =
                        calc_data_block_count_exc_burst_gaps(version, None, size);

                    data_all_total == data_total + parity_total
                })
                &&
                ({
                    let version = Version::V3;

                    let (data_total, parity_total) =
                        calc_data_only_and_parity_block_count_exc_burst_gaps(version, None, size);

                    let data_all_total =
                        calc_data_block_count_exc_burst_gaps(version, None, size);

                    data_all_total == data_total + parity_total
                })
        }

        fn qc_calc_data_block_count_exc_burst_gaps_consistent_rs_enabled(data_par_burst : (usize, usize, usize),
                                                                         size           : u64)
                                                                         -> bool {
            let mut data_par_burst = data_par_burst;
            data_par_burst.0 = if data_par_burst.0 == 0 { 1 } else { data_par_burst.0 };
            data_par_burst.1 = if data_par_burst.1 == 0 { 1 } else { data_par_burst.1 };

            ({
                let version = Version::V17;

                let (data_total, parity_total) =
                    calc_data_only_and_parity_block_count_exc_burst_gaps(version, Some(data_par_burst), size);

                let data_all_total =
                    calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size);

                data_all_total == data_total + parity_total
            })
                &&
                ({
                    let version = Version::V18;

                    let (data_total, parity_total) =
                        calc_data_only_and_parity_block_count_exc_burst_gaps(version, Some(data_par_burst), size);

                    let data_all_total =
                        calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size);

                    data_all_total == data_total + parity_total
                })
                &&
                ({
                    let version = Version::V19;

                    let (data_total, parity_total) =
                        calc_data_only_and_parity_block_count_exc_burst_gaps(version, Some(data_par_burst), size);

                    let data_all_total =
                        calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size);

                    data_all_total == data_total + parity_total
                })
        }

        fn qc_calc_total_block_count_exc_burst_gaps_consistent_rs_disabled(size : u64) -> bool {
            ({
                let version = Version::V1;
                calc_meta_block_count_exc_burst_gaps(version, Some(false), None)
                    + calc_data_block_count_exc_burst_gaps(version, None, size)
                    == calc_total_block_count_exc_burst_gaps(version, Some(false), None, size)
                    &&
                    calc_meta_block_count_exc_burst_gaps(version, Some(true), None)
                    + calc_data_block_count_exc_burst_gaps(version, None, size)
                    == calc_total_block_count_exc_burst_gaps(version, Some(true), None, size)
                    &&
                    calc_meta_block_count_exc_burst_gaps(version, None, None)
                    + calc_data_block_count_exc_burst_gaps(version, None, size)
                    == calc_total_block_count_exc_burst_gaps(version, None, None, size)
                    &&
                    calc_total_block_count_exc_burst_gaps(version, Some(true), None, size)
                    == calc_total_block_count_exc_burst_gaps(version, None, None, size)
                    &&
                    calc_total_block_count_exc_burst_gaps(version, Some(false), None, size) + 1
                    == calc_total_block_count_exc_burst_gaps(version, Some(true), None, size)
            })
                &&
                ({
                    let version = Version::V2;
                    calc_meta_block_count_exc_burst_gaps(version, Some(false), None)
                        + calc_data_block_count_exc_burst_gaps(version, None, size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(false), None, size)
                        &&
                        calc_meta_block_count_exc_burst_gaps(version, Some(true), None)
                        + calc_data_block_count_exc_burst_gaps(version, None, size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(true), None, size)
                        &&
                        calc_meta_block_count_exc_burst_gaps(version, None, None)
                        + calc_data_block_count_exc_burst_gaps(version, None, size)
                        == calc_total_block_count_exc_burst_gaps(version, None, None, size)
                        &&
                        calc_total_block_count_exc_burst_gaps(version, Some(true), None, size)
                        == calc_total_block_count_exc_burst_gaps(version, None, None, size)
                        &&
                        calc_total_block_count_exc_burst_gaps(version, Some(false), None, size) + 1
                        == calc_total_block_count_exc_burst_gaps(version, Some(true), None, size)
                })
                &&
                ({
                    let version = Version::V3;
                    calc_meta_block_count_exc_burst_gaps(version, Some(false), None)
                        + calc_data_block_count_exc_burst_gaps(version, None, size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(false), None, size)
                        &&
                        calc_meta_block_count_exc_burst_gaps(version, Some(true), None)
                        + calc_data_block_count_exc_burst_gaps(version, None, size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(true), None, size)
                        &&
                        calc_meta_block_count_exc_burst_gaps(version, None, None)
                        + calc_data_block_count_exc_burst_gaps(version, None, size)
                        == calc_total_block_count_exc_burst_gaps(version, None, None, size)
                        &&
                        calc_total_block_count_exc_burst_gaps(version, Some(true), None, size)
                        == calc_total_block_count_exc_burst_gaps(version, None, None, size)
                        &&
                        calc_total_block_count_exc_burst_gaps(version, Some(false), None, size) + 1
                        == calc_total_block_count_exc_burst_gaps(version, Some(true), None, size)
                })
        }

        fn qc_calc_total_block_count_exc_burst_gaps_consistent_rs_enabled(data_par_burst : (usize, usize, usize),
                                                                          size           : u64)
                                                                          -> bool {
            let mut data_par_burst = data_par_burst;
            data_par_burst.0 = if data_par_burst.0 == 0 { 1 } else { data_par_burst.0 };
            data_par_burst.1 = if data_par_burst.1 == 0 { 1 } else { data_par_burst.1 };

            ({
                let version = Version::V17;
                calc_meta_block_count_exc_burst_gaps(version, Some(false), Some(data_par_burst))
                    + calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size)
                    == calc_total_block_count_exc_burst_gaps(version, Some(false), Some(data_par_burst), size)
                    &&
                    calc_meta_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst))
                    + calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size)
                    == calc_total_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst), size)
                    &&
                    calc_meta_block_count_exc_burst_gaps(version, None, Some(data_par_burst))
                    + calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size)
                    == calc_total_block_count_exc_burst_gaps(version, None, Some(data_par_burst), size)
                    &&
                    calc_total_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst), size)
                    == calc_total_block_count_exc_burst_gaps(version, None, Some(data_par_burst), size)
                    &&
                    calc_total_block_count_exc_burst_gaps(version, Some(false), Some(data_par_burst), size)
                    == calc_total_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst), size)
            })
                &&
                ({
                    let version = Version::V18;
                    calc_meta_block_count_exc_burst_gaps(version, Some(false), Some(data_par_burst))
                        + calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(false), Some(data_par_burst), size)
                        &&
                        calc_meta_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst))
                        + calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst), size)
                        &&
                        calc_meta_block_count_exc_burst_gaps(version, None, Some(data_par_burst))
                        + calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, None, Some(data_par_burst), size)
                        &&
                        calc_total_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, None, Some(data_par_burst), size)
                        &&
                        calc_total_block_count_exc_burst_gaps(version, Some(false), Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst), size)
                })
                &&
                ({
                    let version = Version::V19;
                    calc_meta_block_count_exc_burst_gaps(version, Some(false), Some(data_par_burst))
                        + calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(false), Some(data_par_burst), size)
                        &&
                        calc_meta_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst))
                        + calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst), size)
                        &&
                        calc_meta_block_count_exc_burst_gaps(version, None, Some(data_par_burst))
                        + calc_data_block_count_exc_burst_gaps(version, Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, None, Some(data_par_burst), size)
                        &&
                        calc_total_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, None, Some(data_par_burst), size)
                        &&
                        calc_total_block_count_exc_burst_gaps(version, Some(false), Some(data_par_burst), size)
                        == calc_total_block_count_exc_burst_gaps(version, Some(true), Some(data_par_burst), size)
                })
        }
    }

    #[test]
    fn test_calc_container_size_simple_cases() {
        assert_eq!(0, calc_container_size(Version::V1, Some(false), None, 0));
        assert_eq!(0, calc_container_size(Version::V2, Some(false), None, 0));
        assert_eq!(0, calc_container_size(Version::V3, Some(false), None, 0));
        assert_eq!(512, calc_container_size(Version::V1, Some(true), None, 0));
        assert_eq!(128, calc_container_size(Version::V2, Some(true), None, 0));
        assert_eq!(4096, calc_container_size(Version::V3, Some(true), None, 0));
        assert_eq!(512, calc_container_size(Version::V1, None, None, 0));
        assert_eq!(128, calc_container_size(Version::V2, None, None, 0));
        assert_eq!(4096, calc_container_size(Version::V3, None, None, 0));

        assert_eq!(512, calc_container_size(Version::V1, Some(false), None, 1));
        assert_eq!(128, calc_container_size(Version::V2, Some(false), None, 1));
        assert_eq!(4096, calc_container_size(Version::V3, Some(false), None, 1));
        assert_eq!(
            512 * 2,
            calc_container_size(Version::V1, Some(true), None, 1)
        );
        assert_eq!(
            128 * 2,
            calc_container_size(Version::V2, Some(true), None, 1)
        );
        assert_eq!(
            4096 * 2,
            calc_container_size(Version::V3, Some(true), None, 1)
        );
        assert_eq!(512 * 2, calc_container_size(Version::V1, None, None, 1));
        assert_eq!(128 * 2, calc_container_size(Version::V2, None, None, 1));
        assert_eq!(4096 * 2, calc_container_size(Version::V3, None, None, 1));

        assert_eq!(
            512 * 2,
            calc_container_size(Version::V17, None, Some((1, 1, 0)), 0)
        );
        assert_eq!(
            128 * 2,
            calc_container_size(Version::V18, None, Some((1, 1, 0)), 0)
        );
        assert_eq!(
            4096 * 2,
            calc_container_size(Version::V19, None, Some((1, 1, 0)), 0)
        );

        assert_eq!(
            512 * (2 + 1),
            calc_container_size(Version::V17, None, Some((1, 1, 1)), 0)
        );
        assert_eq!(
            128 * (2 + 1),
            calc_container_size(Version::V18, None, Some((1, 1, 1)), 0)
        );
        assert_eq!(
            4096 * (2 + 1),
            calc_container_size(Version::V19, None, Some((1, 1, 1)), 0)
        );

        assert_eq!(
            512 * (2 + 2),
            calc_container_size(Version::V17, None, Some((1, 1, 2)), 0)
        );
        assert_eq!(
            128 * (2 + 2),
            calc_container_size(Version::V18, None, Some((1, 1, 2)), 0)
        );
        assert_eq!(
            4096 * (2 + 2),
            calc_container_size(Version::V19, None, Some((1, 1, 2)), 0)
        );

        assert_eq!(
            512 * (2 + 3),
            calc_container_size(Version::V17, None, Some((1, 1, 3)), 0)
        );
        assert_eq!(
            128 * (2 + 3),
            calc_container_size(Version::V18, None, Some((1, 1, 3)), 0)
        );
        assert_eq!(
            4096 * (2 + 3),
            calc_container_size(Version::V19, None, Some((1, 1, 3)), 0)
        );

        assert_eq!(
            1536,
            calc_container_size(Version::V17, None, Some((1, 1, 1)), 0)
        );
        assert_eq!(
            384,
            calc_container_size(Version::V18, None, Some((1, 1, 1)), 0)
        );
        assert_eq!(
            12288,
            calc_container_size(Version::V19, None, Some((1, 1, 1)), 0)
        );

        assert_eq!(
            2048,
            calc_container_size(Version::V17, None, Some((1, 1, 1)), 1)
        );
        assert_eq!(
            512,
            calc_container_size(Version::V18, None, Some((1, 1, 1)), 1)
        );
        assert_eq!(
            16384,
            calc_container_size(Version::V19, None, Some((1, 1, 1)), 1)
        );

        assert_eq!(
            4096,
            calc_container_size(Version::V17, None, Some((1, 1, 1)), 1024)
        );
        assert_eq!(
            2816,
            calc_container_size(Version::V18, None, Some((1, 1, 1)), 1024)
        );
        assert_eq!(
            16384,
            calc_container_size(Version::V19, None, Some((1, 1, 1)), 1024)
        );

        assert_eq!(
            12288,
            calc_container_size(Version::V17, None, Some((1, 1, 1)), 5000)
        );
        assert_eq!(
            11776,
            calc_container_size(Version::V18, None, Some((1, 1, 1)), 5000)
        );
        assert_eq!(
            24576,
            calc_container_size(Version::V19, None, Some((1, 1, 1)), 5000)
        );

        assert_eq!(
            2066432,
            calc_container_size(Version::V17, None, Some((1, 1, 1)), 1_000_000)
        );
        assert_eq!(
            2286080,
            calc_container_size(Version::V18, None, Some((1, 1, 1)), 1_000_000)
        );
        assert_eq!(
            2023424,
            calc_container_size(Version::V19, None, Some((1, 1, 1)), 1_000_000)
        );

        assert_eq!(
            12800,
            calc_container_size(Version::V17, None, Some((11, 3, 7)), 0)
        );
        assert_eq!(
            3200,
            calc_container_size(Version::V18, None, Some((11, 3, 7)), 0)
        );
        assert_eq!(
            102400,
            calc_container_size(Version::V19, None, Some((11, 3, 7)), 0)
        );

        assert_eq!(
            49152,
            calc_container_size(Version::V17, None, Some((11, 3, 7)), 1)
        );
        assert_eq!(
            12288,
            calc_container_size(Version::V18, None, Some((11, 3, 7)), 1)
        );
        assert_eq!(
            393216,
            calc_container_size(Version::V19, None, Some((11, 3, 7)), 1)
        );

        assert_eq!(
            49152,
            calc_container_size(Version::V17, None, Some((11, 3, 7)), 1024)
        );
        assert_eq!(
            12288,
            calc_container_size(Version::V18, None, Some((11, 3, 7)), 1024)
        );
        assert_eq!(
            393216,
            calc_container_size(Version::V19, None, Some((11, 3, 7)), 1024)
        );

        assert_eq!(
            49152,
            calc_container_size(Version::V17, None, Some((11, 3, 7)), 5000)
        );
        assert_eq!(
            12800,
            calc_container_size(Version::V18, None, Some((11, 3, 7)), 5000)
        );
        assert_eq!(
            393216,
            calc_container_size(Version::V19, None, Some((11, 3, 7)), 5000)
        );

        assert_eq!(
            50176,
            calc_container_size(Version::V17, None, Some((11, 3, 7)), 13000)
        );
        assert_eq!(
            25216,
            calc_container_size(Version::V18, None, Some((11, 3, 7)), 13000)
        );
        assert_eq!(
            393216,
            calc_container_size(Version::V19, None, Some((11, 3, 7)), 13000)
        );

        assert_eq!(
            1354240,
            calc_container_size(Version::V17, None, Some((11, 3, 7)), 1_000_000)
        );
        assert_eq!(
            1455616,
            calc_container_size(Version::V18, None, Some((11, 3, 7)), 1_000_000)
        );
        assert_eq!(
            1601536,
            calc_container_size(Version::V19, None, Some((11, 3, 7)), 1_000_000)
        );

        assert_eq!(
            38133760,
            calc_container_size(Version::V17, None, Some((11, 3, 7)), 29_000_000)
        );
        assert_eq!(
            42185728,
            calc_container_size(Version::V18, None, Some((11, 3, 7)), 29_000_000)
        );
        assert_eq!(
            37330944,
            calc_container_size(Version::V19, None, Some((11, 3, 7)), 29_000_000)
        );
    }

    quickcheck! {
        fn qc_calc_container_size_rs_enabled_no_data(data_par_burst : (usize, usize, usize)) -> bool {
            let mut data_par_burst = data_par_burst;
            data_par_burst.0 = if data_par_burst.0 == 0 { 1 } else { data_par_burst.0 };
            data_par_burst.1 = if data_par_burst.1 == 0 { 1 } else { data_par_burst.1 };

            let (_, parity, burst) = data_par_burst;

            (ver_to_block_size(Version::V17) * ((1 + parity) + parity * burst)) as u64 == calc_container_size(Version::V17, None, Some(data_par_burst), 0)
                && (ver_to_block_size(Version::V18) * ((1 + parity) + parity * burst)) as u64 == calc_container_size(Version::V18, None, Some(data_par_burst), 0)
                && (ver_to_block_size(Version::V19) * ((1 + parity) + parity * burst)) as u64 == calc_container_size(Version::V19, None, Some(data_par_burst), 0)
        }

        fn qc_calc_container_size_rs_enabled_not_too_off(data_par_burst : (usize, usize, usize),
                                                         size           : u64) -> bool {
            let size = if size == 0 { 1 } else { size };

            let mut data_par_burst = data_par_burst;
            data_par_burst.0 = if data_par_burst.0 == 0 { 1 } else { data_par_burst.0 };
            data_par_burst.1 = if data_par_burst.1 == 0 { 1 } else { data_par_burst.1 };
            data_par_burst.2 = if data_par_burst.2 == 0 { 1 } else { data_par_burst.2 };

            let (data, parity, burst) = data_par_burst;

            let super_block_set_size = ((data + parity) * burst) as u64;

            ({
                let version = Version::V17;
                let block_size = ver_to_block_size(version) as u64;
                let container_size = calc_container_size(version, None, Some(data_par_burst), size);
                let super_block_set_count_lower_bound = size / (super_block_set_size * block_size);
                let super_block_set_count_upper_bound = (size + ((super_block_set_size * block_size) - 1)) / (super_block_set_size * block_size);
                let container_size_lower_bound = ((1 + parity as u64) + super_block_set_count_lower_bound * super_block_set_size) * block_size;
                let container_size_upper_bound = ((1 + parity as u64) + super_block_set_count_upper_bound * super_block_set_size) * block_size;

                container_size % block_size == 0
                    && container_size >= container_size_lower_bound
                    && container_size <= container_size_upper_bound
            })
        }
    }
}

#[test]
fn test_get_file_name_part_of_path_simple_cases() {
    assert_eq!("abcd", get_file_name_part_of_path("test/abcd"));
    assert_eq!("test.sbx", get_file_name_part_of_path("test/test.sbx"));
    assert_eq!("abcd", get_file_name_part_of_path("/root/test/abcd"));
    assert_eq!(
        "abcd_defg.sbx",
        get_file_name_part_of_path("/root/test/abcd_defg.sbx")
    );
    assert_eq!("abcd", get_file_name_part_of_path("abcd"));
    assert_eq!("test", get_file_name_part_of_path("/test"));
}
