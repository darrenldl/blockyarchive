#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidHexString,
    InvalidLen
}

fn is_valid_hex_char(chr : u8) -> bool {
    (0x30 <= chr && chr <= 0x39)
        || (0x41 <= chr && chr <= 0x46)
        || (0x61 <= chr && chr <= 0x66)
}

fn hex_char_to_value(chr : u8) -> u8 {
    if 0x30 <= chr && chr <= 0x39 {
        chr - 0x30
    }
    else if 0x41 <= chr && chr <= 0x46 {
        chr - 0x41 + 0x0A
    }
    else if 0x61 <= chr && chr <= 0x66 {
        chr - 0x61 + 0x0A
    }
    else { panic!() }
}

pub fn hex_string_to_bytes(string : &str) -> Result<Box<[u8]>, Error> {
    if string.len() % 2 == 0 {
        let string = string.as_bytes();
        for c in string {
            if !is_valid_hex_char(*c) {
                return Err(Error::InvalidHexString) }
        }

        let mut result = Vec::with_capacity(string.len() % 2);

        for i in (0..string.len()).filter(|x| x % 2 == 0) {
            let l_chr_val = hex_char_to_value(string[i]);
            let r_chr_val = hex_char_to_value(string[i+1]);

            println!("{}, {}", l_chr_val, r_chr_val);
            result.push(l_chr_val * 0x10 + r_chr_val);
        }
        Ok(result.into_boxed_slice())
    }
    else {
        Err(Error::InvalidLen)
    }
}

pub fn bytes_to_lower_hex_string(bytes : &[u8]) -> String {
    let strs: Vec<String> = bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    strs.join("")
}

pub fn bytes_to_upper_hex_string(bytes : &[u8]) -> String {
    let strs: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    strs.join("")
}

#[cfg(test)]
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
