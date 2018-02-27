use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_specs::SBX_SCAN_BLOCK_SIZE;
use super::sbx_block::Block;
use super::file_reader::FileReader;
use super::sbx_specs::ver_to_block_size;
use super::Error;

pub struct ReadResult {
    pub len_read : usize,
    pub usable   : bool,
    pub eof      : bool,
}

/*pub fn find_block<T>(pred   : T,
                     buf    : Option<&mut [u8; SBX_LARGEST_BLOCK_SIZE]>,
                     reader : &mut FileReader) -> Block
where T : Fn(&Block) -> bool
{
    let block = Block::dummy();

    let mut internal_buf : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    let buf = buf.unwrap_or(&mut internal_buf);

    block
}*/

pub fn read_block_lazily(block  : &mut Block,
                         buffer : &mut [u8; SBX_LARGEST_BLOCK_SIZE],
                         reader : &mut FileReader)
                         -> Result<ReadResult, Error> {
    let mut total_len_read = 0;

    { // scan at 128 chunk size
        total_len_read += reader.read(&mut buffer[0..SBX_SCAN_BLOCK_SIZE])?;

        if total_len_read < SBX_SCAN_BLOCK_SIZE {
            return Ok(ReadResult { len_read : total_len_read,
                                   usable   : false,
                                   eof      : true            });
        }

        match block.sync_from_buffer_header_only(&buffer[0..SBX_SCAN_BLOCK_SIZE]) {
            Ok(()) => {},
            Err(_) => { return Ok(ReadResult { len_read : total_len_read,
                                               usable   : false,
                                               eof      : false           }); }
        }
    }

    { // get remaining bytes of block if necessary
        let block_size = ver_to_block_size(block.get_version());

        total_len_read +=
            reader.read(&mut buffer[SBX_SCAN_BLOCK_SIZE..block_size])?;

        if total_len_read < block_size {
            return Ok(ReadResult { len_read : total_len_read,
                                   usable   : false,
                                   eof      : true            });
        }

        match block.sync_from_buffer(&buffer[0..block_size]) {
            Ok(()) => {},
            Err(_) => { return Ok(ReadResult { len_read : total_len_read,
                                               usable   : false,
                                               eof      : false           }); }
        }
    }

    Ok(ReadResult { len_read : total_len_read,
                    usable   : true,
                    eof      : false           })
}
