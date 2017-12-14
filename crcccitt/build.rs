use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const CRC_POLY_CCITT : u16 = 0x1021;

fn make_crcccitt_tab() -> [u16; 256] {
    let mut crc : u16;
    let mut c : u16;

    let mut table : [u16; 256] = [0; 256];

    for i in 0u16..256u16 {

        crc = 0;
        c   = i << 8;

        for _ in 0..8 {

            if ((crc ^ c) & 0x8000u16) != 0 {
                crc = ( crc << 1 ) ^ CRC_POLY_CCITT; }
            else {
                crc =   crc << 1; }

            c = c << 1;
        }

        table[i as usize] = crc;
    }

    table
}

fn main() {
    let table = make_crcccitt_tab();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("table.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut table_str = String::from("static CRCCCITT_TABLE : [u16; 256] = [");

    for v in table.iter() {
        let str = format!("{}, ", v);
        table_str.push_str(&str);
    }

    table_str.push_str("];");

    f.write_all(table_str.as_bytes()).unwrap();
}
