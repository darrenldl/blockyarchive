#![cfg(test)]

use crate::integer_utils::IntegerUtils;
use crate::misc_utils::*;

mod hex_tests {
    use super::*;
    use crate::rand_utils;

    #[test]
    fn hex_to_bytes_test_cases() {
        {
            let hex = "010203";
            assert_eq!(&[1u8, 2, 3], hex_string_to_bytes(hex).unwrap().as_ref());
        }
        {
            let hex = "abcdef";
            assert_eq!(
                &[0xABu8, 0xCD, 0xEF],
                hex_string_to_bytes(hex).unwrap().as_ref()
            );
        }
        {
            let hex = "ABCDEF";
            assert_eq!(
                &[0xABu8, 0xCD, 0xEF],
                hex_string_to_bytes(hex).unwrap().as_ref()
            );
        }
    }

    #[test]
    fn bytes_to_hex_test_cases() {
        {
            let bytes = [0u8, 1, 2];
            assert_eq!("000102", bytes_to_lower_hex_string(&bytes));
        }
        {
            let bytes = [0u8, 1, 2];
            assert_eq!("000102", bytes_to_upper_hex_string(&bytes));
        }
        {
            let bytes = [0xABu8, 0xCD, 0xEF];
            assert_eq!("abcdef", bytes_to_lower_hex_string(&bytes));
        }
        {
            let bytes = [0xABu8, 0xCD, 0xEF];
            assert_eq!("ABCDEF", bytes_to_upper_hex_string(&bytes));
        }
    }

    #[test]
    fn hex_to_bytes_to_hex() {
        {
            let hex = "1234567890";
            assert_eq!(
                hex,
                bytes_to_lower_hex_string(hex_string_to_bytes(hex).unwrap().as_ref())
            );
        }
        {
            let hex = "abcdef0123";
            assert_eq!(
                hex,
                bytes_to_lower_hex_string(hex_string_to_bytes(hex).unwrap().as_ref())
            );
        }
        {
            let hex = "1234567890";
            assert_eq!(
                hex,
                bytes_to_upper_hex_string(hex_string_to_bytes(hex).unwrap().as_ref())
            );
        }
        {
            let hex = "ABCDEF0123";
            assert_eq!(
                hex,
                bytes_to_upper_hex_string(hex_string_to_bytes(hex).unwrap().as_ref())
            );
        }
        {
            let mut bytes: [u8; 100] = [0; 100];
            for _ in 0..1000 {
                rand_utils::fill_random_bytes(&mut bytes);
                let hex = bytes_to_lower_hex_string(&bytes);
                assert_eq!(
                    hex,
                    bytes_to_lower_hex_string(hex_string_to_bytes(&hex).unwrap().as_ref())
                );
                let hex = bytes_to_upper_hex_string(&bytes);
                assert_eq!(
                    hex,
                    bytes_to_upper_hex_string(hex_string_to_bytes(&hex).unwrap().as_ref())
                );
            }
        }
    }

    #[test]
    fn error_handling() {
        assert_eq!(
            hex_string_to_bytes("abc").unwrap_err(),
            HexError::InvalidLen
        );
        assert_eq!(
            hex_string_to_bytes("LL").unwrap_err(),
            HexError::InvalidHexString
        );
    }
}

#[test]
fn test_f64_max_simple_cases() {
    assert_eq!(0.2, f64_max(0.1, 0.2));
    assert_eq!(10.0, f64_max(10.0, 9.0));
    assert_eq!(0.3, f64_max(0.3, 0.2));
    assert_eq!(100.0, f64_max(9.0, 100.0));
    assert_eq!(-1.0, f64_max(-1.0, -2.0));
    assert_eq!(2.0, f64_max(2.0, -10.0));
}

quickcheck! {
    fn qc_calc_required_len_and_seek_to_from_byte_range(from_byte         : Option<u64>,
                                                        to_byte_inc       : Option<u64>,
                                                        force_misalign    : bool,
                                                        bytes_so_far      : u64,
                                                        last_possible_pos : u64) -> bool {
        let to_byte = match to_byte_inc {
            None    => None,
            Some(x) => Some(RangeEnd::Inc(x)),
        };

        let RequiredLenAndSeekTo { required_len, seek_to } =
            calc_required_len_and_seek_to_from_byte_range(from_byte,
                                                          to_byte,
                                                          force_misalign,
                                                          bytes_so_far,
                                                          last_possible_pos,
                                                          None);

        let from_byte = match from_byte   {
            None => 0,
            Some(x) => {
                if force_misalign { x }
                else              { u64::round_down_to_multiple(x, 128) }
            }
        };

        let to_byte = to_byte_inc.unwrap_or(last_possible_pos);

        seek_to <= last_possible_pos
            && ((from_byte + bytes_so_far <= last_possible_pos
                 && seek_to <= from_byte + bytes_so_far)
                || (from_byte + bytes_so_far > last_possible_pos))
            && (force_misalign
                || (!force_misalign && seek_to % 128 == 0))
            && (from_byte + required_len == to_byte + 1
                || from_byte + required_len == last_possible_pos + 1
                || required_len == 1)
    }
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_make_path_simple_cases() {
    assert_eq!("abcd/efgh", make_path(&["abcd", "efgh"]));
    assert_eq!(
        "/usb/folder1/test.sbx",
        make_path(&["/usb/", "/folder1/", "/test.sbx/"])
    );
    assert_eq!("/abcd/efgh", make_path(&["/abcd/", "efgh"]));
    assert_eq!("abcd/efgh", make_path(&["abcd/", "/efgh"]));
    assert_eq!("/abcd/efgh", make_path(&["/abcd", "efgh/"]));
    assert_eq!("/efgh", make_path(&["/", "efgh/"]));
    assert_eq!("/efgh", make_path(&["/", "efgh/", "/"]));
    assert_eq!("test", make_path(&["test", "/"]));
}

#[test]
fn test_buffer_is_blank_simple_cases() {
    let buffer: [u8; 100] = [0; 100];
    assert!(buffer_is_blank(&buffer));
    assert!(buffer_is_blank(&buffer[0..10]));
    assert!(buffer_is_blank(&buffer[1..2]));
    assert!(buffer_is_blank(&buffer[15..20]));
    assert!(buffer_is_blank(&buffer[21..37]));
}

#[test]
fn test_camelcase() {
    assert_eq!("testABCD", to_camelcase("test a b c D"));
    assert_eq!("fileSize", to_camelcase("File size"));
    assert_eq!("low", to_camelcase("low"));
}
