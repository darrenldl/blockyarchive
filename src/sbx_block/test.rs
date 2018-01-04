#![cfg(test)]

use super::Block;
use super::super::sbx_specs;

#[test]
fn test_write_to_bytes() {
    let mut bytes : [u8; 512] = [0; 512];
    let file_uid : [u8; 6] = [0; 6];

    let block = Block::new(sbx_specs::Version::V1,
                           &file_uid,
    )
}
