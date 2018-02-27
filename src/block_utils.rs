use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_block::Block;
use super::file_reader::FileReader;

pub fn find_block<T>(pred   : T,
                     buf    : Option<&mut [u8; SBX_LARGEST_BLOCK_SIZE]>,
                     reader : &mut FileReader) -> Block
where T : Fn(&Block) -> bool
{
    let block = Block::dummy();

    let mut internal_buf : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    let buf = buf.unwrap_or(&mut internal_buf);

    block
}
