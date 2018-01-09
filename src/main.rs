#![allow(dead_code)]

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

#[macro_use]
extern crate nom;

extern crate time;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    FileOpenFail(String),
    FileCreateFail(String),
    RSCodecCreateFail
}

fn main () {
}
