#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate blkar_lib;

use blkar_lib::sbx_block::Block;

use blkar_lib::sbx_specs;

fuzz_target!(|data: &[u8]| {
    let mut block = Block::dummy();
    let mut block2 = Block::dummy();

    let mut buffer : [u8; 4096] = [0; 4096];
    let mut buffer2 : [u8; 4096] = [0; 4096];

    if data.len() >= 4096 {
        if let Ok(()) = block.sync_from_buffer(data, None) {
            let block_size = sbx_specs::ver_to_block_size(block.get_version());

            buffer.copy_from_slice(&data[0..block_size]);
            buffer2.copy_from_slice(&data[0..block_size]);

            for i in 0..16 {
                buffer[i]  = 0;
                buffer2[i] = 0;
            }

            block.sync_to_buffer(None, &mut buffer).unwrap();

            block2.sync_from_buffer(&buffer, None).unwrap();

            block2.sync_to_buffer(None, &mut buffer2).unwrap();

            if block.get_seq_num() > 0 {
                assert_eq!(&buffer[0..block_size], &data[0..block_size]);
                assert_eq!(&buffer[0..block_size], &buffer2[0..block_size]);
            }

            assert_eq!(&buffer[0..4],  &buffer2[0..4]);
            assert_eq!(&buffer[6..16], &buffer2[6..16]);
        }
    }
});
