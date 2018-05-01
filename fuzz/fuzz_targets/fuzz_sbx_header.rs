#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate rsbx_lib;

fuzz_target!(|data: &[u8]| {
});
