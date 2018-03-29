#![cfg(test)]
use super::encoder::*;
use sbx_block;
use sbx_specs::{Version,
                SBX_LARGEST_BLOCK_SIZE,
                ver_to_block_size,
                ver_uses_rs};

#[test]
fn test_encoder_encode_correctly_simple_cases() {
    {
        let mut rs_codec = RSEncoder::new(Version::V17, 1, 1);

        assert!(!rs_codec.active());
        assert_eq!(1, rs_codec.unfilled_slot_count());
        assert_eq!(1, rs_codec.total_slot_count());
    }
}
