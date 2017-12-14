include!(concat!(env!("OUT_DIR"), "/table.rs"));

pub fn crc_ccitt_generic (input : &[u8], start_val : u16) -> u16 {
    let mut crc : u16 = start_val;

    for c in input {
        let c_u16 :u16 = *c as u16;

        crc =
            (crc << 8)
            ^
            CRCCCITT_TABLE[ (((crc >> 8) ^ c_u16) & 0x00FFu16) as usize ];
    }

    crc
}

#[cfg(test)]
mod tests {
    use super::crc_ccitt_generic;

    #[test]
    fn basic_value_tests_0xffff() {
        assert_eq!(crc_ccitt_generic(b"a", 0xFFFF), 0x9D77);
        assert_eq!(crc_ccitt_generic(b"abcd", 0xFFFF), 0x2CF6);
        assert_eq!(crc_ccitt_generic(b"0", 0xFFFF), 0xD7A3);
        assert_eq!(crc_ccitt_generic(b"0123", 0xFFFF), 0x3F7B);
    }

    #[test]
    fn basic_value_tests_0x1d0f() {
        assert_eq!(crc_ccitt_generic(b"a", 0x1D0f), 0xB01B);
        assert_eq!(crc_ccitt_generic(b"abcd", 0x1D0f), 0xA626);
        assert_eq!(crc_ccitt_generic(b"0", 0x1D0F), 0xFACF);
        assert_eq!(crc_ccitt_generic(b"0123", 0x1D0F), 0xB5AB);
    }
}
