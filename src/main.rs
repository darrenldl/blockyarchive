#![allow(dead_code)]
//use std::sync::mpsc;
//use std::{thread, time};
//use std::time::SystemTime;

#[macro_use(shards)]
extern crate reed_solomon_erasure;

mod multihash;
mod misc_utils;
mod rand_utils;
mod sbx_block;
mod sbx_specs;

use multihash::*;

use reed_solomon_erasure::*;

fn main () {
    let r = ReedSolomon::new(3, 2); // 3 data shards, 2 parity shards

    let mut master_copy = shards!([0, 1,  2,  3],
                                  [4, 5,  6,  7],
                                  [8, 9, 10, 11],
                                  [0, 0,  0,  0], // last 2 rows are parity shards
                                  [0, 0,  0,  0]);

    r.encode_parity(&mut master_copy, None, None);
    
    let mut shards = shards_into_option_shards(master_copy.clone());

    // We can remove up to 2 shards, which may be data or parity shards
    shards[0] = None;
    shards[4] = None;
    
    // Try to reconstruct missing shards
    r.decode_missing(&mut shards, None, None).unwrap();
    
    // Convert back to normal shard arrangement
    let result = option_shards_into_shards(shards);
    
    assert!(r.is_parity_correct(&result, None, None));
    assert_eq!(master_copy, result);
}
