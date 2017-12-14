use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const FIELD_SIZE : usize = 256;

const GENERATING_POLYNOMIAL : usize = 29;

fn gen_log_table(polynomial : usize) -> [u8; FIELD_SIZE] {
    let mut result : [u8; FIELD_SIZE] = [0; FIELD_SIZE];
    let mut b      : usize            = 1;

    for log in 0..FIELD_SIZE-1 {
        result[b] = log as u8;

        b = b << 1;

        if FIELD_SIZE <= b {
            b = (b - FIELD_SIZE) ^ polynomial;
        }
    }

    result
}

fn main() {
    let log_table = gen_log_table(GENERATING_POLYNOMIAL);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("table.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut log_table_str = String::from("static LOG_TABLE : [u8; 256] = [");

    for v in log_table.iter() {
        let str = format!("{}, ", v);
        log_table_str.push_str(&str);
    }

    log_table_str.push_str("];\n");

    f.write_all(log_table_str.as_bytes()).unwrap();
}
