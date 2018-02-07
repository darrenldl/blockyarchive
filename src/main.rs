#![allow(dead_code)]

#[macro_use]
extern crate nom;

extern crate time;

extern crate pond;

extern crate futures;

extern crate smallvec;

#[macro_use]
extern crate reed_solomon_erasure;

#[macro_use]
mod worker_macros;

macro_rules! smallvec {
    [
        $arr:ty => $val:expr; $len:expr
    ] => {{
        let mut v : SmallVec<$arr> = SmallVec::with_capacity($len);
        for _ in 0..$len {
            v.push($val);
        }
        v
    }};
    [
        $val:expr; $len:expr
    ] => {{
        let mut v = SmallVec::with_capacity($len);
        for _ in 0..$len {
            v.push($val);
        }
        v
    }}
}

mod file_error;

mod general_error;
use general_error::Error;
use general_error::ErrorKind;

mod multihash;
mod multihash_test;
mod misc_utils;
mod misc_utils_test;
mod file_utils;
mod rand_utils;
mod time_utils;
mod integer_utils;
mod sbx_block;
mod sbx_specs;

mod rs_codec;

mod encode_core;
mod decode_core;
mod rescue_core;
mod repair_core;
mod show_core;
mod sort_core;

mod progress_report;

mod file_reader;
use file_reader::FileReader;
mod file_writer;
use file_writer::FileWriter;

mod worker;

fn main () {
    /*use encode_core::Param;
    let param = Param::new(sbx_specs::Version::V11,
                           &[0, 1, 2, 3, 4, 5],
                           10,
                           2,
                           true,
                           multihash::HashType::SHA256,
                           "test",
                           "test.sbx",
                           progress_report::SilenceLevel::L0);
    match encode_core::encode_file(&param) {
        Ok(s)  => print!("{}", s),
        Err(e) => print!("{}", e)
    }*/
    use decode_core::Param;
    let param = Param::new(false,
                           "test.sbx",
                           "test2",
                           progress_report::SilenceLevel::L0);
    match decode_core::decode_file(&param) {
        Ok(s)  => print!("{}", s),
        Err(e) => print!("{}", e)
    }
}
