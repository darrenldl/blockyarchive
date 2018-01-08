#![cfg(test)]

use super::metadata;
use super::metadata::Metadata;
use super::super::misc_utils::slice_to_boxed;

#[test]
fn test_to_bytes_simple_cases() {
    {
        let expect = b"FNM\x0Ahelloworld";
        let meta = [Metadata::FNM(slice_to_boxed(b"helloworld"))];

        let mut buffer : [u8; 100] = [0; 100];
        metadata::to_bytes(&meta, &mut buffer).unwrap();

        assert_eq!(*expect, *&buffer[0..expect.len()]);
    }
}
