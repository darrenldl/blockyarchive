#![cfg(test)]

use sbx_specs::*;

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
