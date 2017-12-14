const CRC_POLY_CCITT : u16 = 0x1021;

include!(concat!(env!("OUT_DIR"), "/table.rs"));

fn crc_ccitt_generic (input : &[u8], start_val : u16) -> u16 {
    let mut crc : u16;

    const
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
