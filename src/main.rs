#![allow(dead_code)]

macro_rules! worker_stop {
    (
        graceful => $tx_error:path, [ $( $c:path ),* ]
    ) => {{
        $tx_error.send(None).unwrap();
        $( $c.send(None).unwrap(); )*;
        break;
    }};
    (
        with_error => $tx_error:path, $error:expr, [ $( $c:path ),* ]
    ) => {{
        $tx_error.send(Some($error)).unwrap();
        $( $c.send(None).unwrap(); )*;
        break;
    }};
    (
        with_error_ret => $tx_error:path, $error:expr, [ $( $c:path ),* ]
    ) => {{
        $tx_error.send(Some($error)).unwrap();
        $( $c.send(None).unwrap(); )*;
        return;
    }}
}

mod file_error;
use file_error::FileError;

mod general_error;
use general_error::Error;
use general_error::ErrorKind;

mod multihash;
mod multihash_test;
mod misc_utils;
mod misc_utils_test;
mod rand_utils;
mod sbx_block;
mod sbx_specs;

mod encode_core;
mod decode_core;
mod rescue_core;
mod repair_core;
mod show_core;
mod sort_core;

mod file_reader;
use file_reader::FileReader;
mod file_writer;
use file_writer::FileWriter;

mod worker;


#[macro_use]
extern crate nom;

extern crate time;

extern crate scoped_threadpool;

fn main () {
    use encode_core::Param;
    let param = Param {
        version : sbx_specs::Version::V1,
        file_uid : [0, 1, 2, 3, 4, 5],
        rs_enabled : true,
        rs_data    : 10,
        rs_parity  : 2,
        hash_enabled : true,
        hash_type  : multihash::HashType::SHA256,
        in_file    : String::from("test"),
        out_file   : String::from("text.sbx")
    };
    encode_core::encode_file(&param).unwrap();
}
