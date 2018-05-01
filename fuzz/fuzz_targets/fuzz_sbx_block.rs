#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate rsbx_lib;

use rsbx_lib::sbx_block;

fuzz_target!(|data: &[u8]| {
});
