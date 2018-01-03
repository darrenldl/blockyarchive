use super::super::multihash;
use std;

#[derive(Clone, Debug, Copy)]
pub enum Error {
    TooMuchMetaData
}

#[derive(Clone, Debug)]
pub enum Metadata {
    FNM(Box<[u8]>),
    SNM(Box<[u8]>),
    FSZ(u64),
    FDT(u64),
    SDT(u64),
    HSH(multihash::HashBytes)
}

fn single_meta_size(meta : &Metadata) -> usize {
    use self::Metadata::*;
    match *meta {
        FNM(ref x) | SNM(ref x) => x.len(),
        FSZ(_) | FDT(_) | SDT(_) => 8,
        HSH(ref x) => multihash::specs::Param::new(x.0).total_length
    }
}

fn single_write_to_bytes(meta   : &Metadata,
                         buffer : &mut [u8]) -> Result<usize, Error> {
    let size = single_meta_size(meta);

    if buffer.len() < size {
        return Err(Error::TooMuchMetaData);
    }

    use self::Metadata::*;
    match *meta {
        FNM(ref x) | SNM(ref x) => {
            &buffer[0..x.len()].copy_from_slice(x);
        },
        FSZ(x) | FDT(x) | SDT(x) => {
            let be_bytes : [u8; 8] =
                unsafe { std::mem::transmute::<u64, [u8; 8]>(x) };
            &buffer[0..8].copy_from_slice(&be_bytes);
        },
        HSH(ref x) => {
            multihash::hash_bytes_to_bytes(x, &mut buffer[0..size]);
        }
    }

    Ok(size)
}

pub fn write_to_bytes(meta   : &[Metadata],
                      buffer : &mut [u8])
                      -> Result<(), Error> {
    let mut cur_pos = 0;
    for m in meta.iter() {
        let size_written = single_write_to_bytes(m, &mut buffer[cur_pos..])?;

        cur_pos += size_written;
    }

    Ok(())
}
