#![cfg(test)]

use crate::sbx_specs::*;

#[test]
fn test_string_to_ver() {
    assert_eq!(Version::V1, string_to_ver("1").unwrap());
    assert_eq!(Version::V2, string_to_ver("2").unwrap());
    assert_eq!(Version::V3, string_to_ver("3").unwrap());
    assert_eq!(Version::V17, string_to_ver("17").unwrap());
    assert_eq!(Version::V18, string_to_ver("18").unwrap());
    assert_eq!(Version::V19, string_to_ver("19").unwrap());
    assert_eq!(Err(()), string_to_ver("0"));
    assert_eq!(Err(()), string_to_ver("4"));
    assert_eq!(Err(()), string_to_ver("16"));
    assert_eq!(Err(()), string_to_ver("20"));
}

#[test]
fn test_ver_to_block_size() {
    assert_eq!(512, ver_to_block_size(Version::V1));
    assert_eq!(128, ver_to_block_size(Version::V2));
    assert_eq!(4096, ver_to_block_size(Version::V3));
    assert_eq!(512, ver_to_block_size(Version::V17));
    assert_eq!(128, ver_to_block_size(Version::V18));
    assert_eq!(4096, ver_to_block_size(Version::V19));
}

#[test]
fn test_ver_to_data_size() {
    assert_eq!(496, ver_to_data_size(Version::V1));
    assert_eq!(112, ver_to_data_size(Version::V2));
    assert_eq!(4080, ver_to_data_size(Version::V3));
    assert_eq!(496, ver_to_data_size(Version::V17));
    assert_eq!(112, ver_to_data_size(Version::V18));
    assert_eq!(4080, ver_to_data_size(Version::V19));
}

#[test]
fn test_ver_uses_rs() {
    assert!(!ver_uses_rs(Version::V1));
    assert!(!ver_uses_rs(Version::V2));
    assert!(!ver_uses_rs(Version::V3));
    assert!(ver_uses_rs(Version::V17));
    assert!(ver_uses_rs(Version::V18));
    assert!(ver_uses_rs(Version::V19));
}

#[test]
fn test_ver_forces_meta_enabled() {
    assert!(!ver_forces_meta_enabled(Version::V1));
    assert!(!ver_forces_meta_enabled(Version::V2));
    assert!(!ver_forces_meta_enabled(Version::V3));
    assert!(ver_forces_meta_enabled(Version::V17));
    assert!(ver_forces_meta_enabled(Version::V18));
    assert!(ver_forces_meta_enabled(Version::V19));
}

#[test]
fn test_ver_to_max_block_set_count() {
    {
        assert_eq!(None, ver_to_max_block_set_count(Version::V1, None));
        assert_eq!(None, ver_to_max_block_set_count(Version::V2, None));
        assert_eq!(None, ver_to_max_block_set_count(Version::V3, None));
    }
    {
        assert_eq!(Some((2u64.pow(32) - 1) / (10 + 2)), ver_to_max_block_set_count(Version::V17, Some((10, 2, 1))).map(|x| x as u64));
        assert_eq!(Some((2u64.pow(32) - 1) / (10 + 2)), ver_to_max_block_set_count(Version::V17, Some((10, 2, 11))).map(|x| x as u64));
        assert_eq!(Some((2u64.pow(32) - 1) / (10 + 2)), ver_to_max_block_set_count(Version::V17, Some((10, 2, 111))).map(|x| x as u64));
        assert_eq!(Some((2u64.pow(32) - 1) / (10 + 2)), ver_to_max_block_set_count(Version::V18, Some((10, 2, 1))).map(|x| x as u64));
        assert_eq!(Some((2u64.pow(32) - 1) / (10 + 2)), ver_to_max_block_set_count(Version::V18, Some((10, 2, 11))).map(|x| x as u64));
        assert_eq!(Some((2u64.pow(32) - 1) / (10 + 2)), ver_to_max_block_set_count(Version::V18, Some((10, 2, 111))).map(|x| x as u64));
        assert_eq!(Some((2u64.pow(32) - 1) / (10 + 2)), ver_to_max_block_set_count(Version::V19, Some((10, 2, 1))).map(|x| x as u64));
        assert_eq!(Some((2u64.pow(32) - 1) / (10 + 2)), ver_to_max_block_set_count(Version::V19, Some((10, 2, 11))).map(|x| x as u64));
        assert_eq!(Some((2u64.pow(32) - 1) / (10 + 2)), ver_to_max_block_set_count(Version::V19, Some((10, 2, 111))).map(|x| x as u64));
    }
}

quickcheck! {
    fn qc_ver_to_max_block_set_count(data: usize,
                                     parity: usize,
                                     burst: usize) -> bool {
        let data = 1 + data % 128;
        let parity = 1 + parity % 128;

        Some((2u64.pow(32) - 1) / (data + parity) as u64) == ver_to_max_block_set_count(Version::V17, Some((data, parity, burst))).map(|x| x as u64)
            && Some((2u64.pow(32) - 1) / (data + parity) as u64) == ver_to_max_block_set_count(Version::V18, Some((data, parity, burst))).map(|x| x as u64)
            && Some((2u64.pow(32) - 1) / (data + parity) as u64) == ver_to_max_block_set_count(Version::V19, Some((data, parity, burst))).map(|x| x as u64)
    }
}

#[test]
fn test_ver_to_last_data_seq_num_exc_parity() {
    {
        assert_eq!(SBX_LAST_SEQ_NUM, ver_to_last_data_seq_num_exc_parity(Version::V1, None));
        assert_eq!(SBX_LAST_SEQ_NUM, ver_to_last_data_seq_num_exc_parity(Version::V2, None));
        assert_eq!(SBX_LAST_SEQ_NUM, ver_to_last_data_seq_num_exc_parity(Version::V3, None));
    }
    {
        assert_eq!((2u64.pow(32) - 1) / (10 + 2) * (10 + 2), ver_to_last_data_seq_num_exc_parity(Version::V17, Some((10, 2, 1))) as u64);
        assert_eq!((2u64.pow(32) - 1) / (10 + 2) * (10 + 2), ver_to_last_data_seq_num_exc_parity(Version::V17, Some((10, 2, 11))) as u64);
        assert_eq!((2u64.pow(32) - 1) / (10 + 2) * (10 + 2), ver_to_last_data_seq_num_exc_parity(Version::V17, Some((10, 2, 111))) as u64);
        assert_eq!((2u64.pow(32) - 1) / (10 + 2) * (10 + 2), ver_to_last_data_seq_num_exc_parity(Version::V18, Some((10, 2, 1))) as u64);
        assert_eq!((2u64.pow(32) - 1) / (10 + 2) * (10 + 2), ver_to_last_data_seq_num_exc_parity(Version::V18, Some((10, 2, 11))) as u64);
        assert_eq!((2u64.pow(32) - 1) / (10 + 2) * (10 + 2), ver_to_last_data_seq_num_exc_parity(Version::V18, Some((10, 2, 111))) as u64);
        assert_eq!((2u64.pow(32) - 1) / (10 + 2) * (10 + 2), ver_to_last_data_seq_num_exc_parity(Version::V19, Some((10, 2, 1))) as u64);
        assert_eq!((2u64.pow(32) - 1) / (10 + 2) * (10 + 2), ver_to_last_data_seq_num_exc_parity(Version::V19, Some((10, 2, 11))) as u64);
        assert_eq!((2u64.pow(32) - 1) / (10 + 2) * (10 + 2), ver_to_last_data_seq_num_exc_parity(Version::V19, Some((10, 2, 111))) as u64);
    }
}

#[test]
fn ver_to_max_data_file_size() {
    
}
