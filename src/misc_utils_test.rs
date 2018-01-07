#![cfg(test)]

use super::misc_utils::*;

mod hex_tests {
    use super::*;
    use super::super::rand_utils;

    #[test]
    fn hex_to_bytes_test_cases() {
        {
            let hex = "010203";
            assert_eq!(&[1u8, 2, 3],
                       hex_string_to_bytes(hex).unwrap().as_ref());
        }
        {
            let hex = "abcdef";
            assert_eq!(&[0xABu8, 0xCD, 0xEF],
                       hex_string_to_bytes(hex).unwrap().as_ref());
        }
        {
            let hex = "ABCDEF";
            assert_eq!(&[0xABu8, 0xCD, 0xEF],
                       hex_string_to_bytes(hex).unwrap().as_ref());
        }
    }

    #[test]
    fn bytes_to_hex_test_cases() {
        {
            let bytes = [0u8, 1, 2];
            assert_eq!("000102",
                       bytes_to_lower_hex_string(&bytes));
        }
        {
            let bytes = [0u8, 1, 2];
            assert_eq!("000102",
                       bytes_to_upper_hex_string(&bytes));
        }
        {
            let bytes = [0xABu8, 0xCD, 0xEF];
            assert_eq!("abcdef",
                       bytes_to_lower_hex_string(&bytes));
        }
        {
            let bytes = [0xABu8, 0xCD, 0xEF];
            assert_eq!("ABCDEF",
                       bytes_to_upper_hex_string(&bytes));
        }
    }

    #[test]
    fn hex_to_bytes_to_hex() {
        {
            let hex = "1234567890";
            assert_eq!(hex,
                       bytes_to_lower_hex_string(
                           hex_string_to_bytes(hex).unwrap().as_ref()));
        }
        {
            let hex = "abcdef0123";
            assert_eq!(hex,
                       bytes_to_lower_hex_string(
                           hex_string_to_bytes(hex).unwrap().as_ref()));
        }
        {
            let hex = "1234567890";
            assert_eq!(hex,
                       bytes_to_upper_hex_string(
                           hex_string_to_bytes(hex).unwrap().as_ref()));
        }
        {
            let hex = "ABCDEF0123";
            assert_eq!(hex,
                       bytes_to_upper_hex_string(
                           hex_string_to_bytes(hex).unwrap().as_ref()));
        }
        {
            let mut bytes : [u8; 100] = [0; 100];
            for _ in 0..1000 {
                rand_utils::fill_random_bytes(&mut bytes);
                let hex = bytes_to_lower_hex_string(&bytes);
                assert_eq!(hex,
                           bytes_to_lower_hex_string(
                               hex_string_to_bytes(&hex).unwrap().as_ref()));
                let hex = bytes_to_upper_hex_string(&bytes);
                assert_eq!(hex,
                           bytes_to_upper_hex_string(
                               hex_string_to_bytes(&hex).unwrap().as_ref()));
            }
        }
    }

    #[test]
    fn error_handling() {
        assert_eq!(hex_string_to_bytes("abc").unwrap_err(),
                   Error::InvalidLen);
        assert_eq!(hex_string_to_bytes("LL").unwrap_err(),
                   Error::InvalidHexString);
    }
}
