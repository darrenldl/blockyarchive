#![allow(dead_code)]
//use std::sync::mpsc;
//use std::{thread, time};
//use std::time::SystemTime;

extern crate crcccitt;
extern crate reed_solomon_erasure;

mod multihash;
mod misc_utils;
mod rand_utils;
mod sbx_block;
mod sbx_specs;

use multihash::*;

//use reed_solomon::ReedSolomon;

//extern crate hex_slice;
//use hex_slice::AsHex;

fn main() {
    let mut ctx = hash::Ctx::new(HashType::SHA1).unwrap();

    ctx.update("abcd".as_bytes());

    let result = ctx.finish_into_bytes();

    println!("{}", misc_utils::bytes_to_upper_hex_string(&result));
    println!("{:?}", misc_utils::hex_string_to_bytes("0102").unwrap())
}
