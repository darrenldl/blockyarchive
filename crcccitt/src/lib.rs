include!(concat!(env!("OUT_DIR"), "/table.rs"));

pub fn crc_ccitt_generic (input : &[u8], start_val : u16) -> u16 {
    let mut crc : u16 = start_val;

    for c in input {
        let c_u16 :u16 = *c as u16;

        crc =
            (!crc << 8)
            ^
            CRCCCITT_TABLE[ (((!crc >> 8) ^ c_u16) & 0x00FFu16) as usize ];
    }

    crc
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
