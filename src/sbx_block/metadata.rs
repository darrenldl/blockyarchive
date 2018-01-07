use super::super::multihash;
use std;
use super::Error;

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
        HSH(ref x) => multihash::specs::Param::new(x.0).total_length()
    }
}

fn single_to_bytes(meta   : &Metadata,
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

pub fn to_bytes(meta   : &[Metadata],
                buffer : &mut [u8])
                -> Result<(), Error> {
    let mut cur_pos = 0;
    for m in meta.iter() {
        let size_written = single_to_bytes(m, &mut buffer[cur_pos..])?;

        cur_pos += size_written;
    }

    Ok(())
}

mod parsers {
    use super::Metadata;
    use super::Metadata::*;
    use super::super::super::misc_utils;
    use super::super::super::multihash::parsers::multihash_w_len_p;

    use nom::be_u8;
    use nom::be_u64;

    macro_rules! make_meta_parser {
        (
            $name:ident, $id:expr, $constructor:path
                => num, $res_parser:ident
        ) => {
            named!($name <Metadata>,
                   do_parse!(
                       _id : tag!($id) >>
                           n : be_u8 >>
                           res : $res_parser >>
                           ($constructor(res))
                   )
            );
        };
        (
            $name:ident, $id:expr, $constructor:path => str
        ) => {
            named!($name <Metadata>,
                   do_parse!(
                       _id : tag!($id) >>
                           n : be_u8 >>
                           res : take!(n) >>
                           ($constructor(misc_utils::slice_to_vec(res)
                                         .into_boxed_slice()))
                   )
            );
        };
    }

    make_meta_parser!(fnm_p, b"FNM", FNM => str);
    make_meta_parser!(snm_p, b"SNM", SNM => str);
    make_meta_parser!(fsz_p, b"FSZ", FSZ => num, be_u64);
    make_meta_parser!(fdt_p, b"FDT", FDT => num, be_u64);
    make_meta_parser!(sdt_p, b"SDT", SDT => num, be_u64);

    named!(hsh_p <Metadata>,
           do_parse!(
               _id : tag!(b"HSH") >>
                   res : multihash_w_len_p >>
                   (HSH(res))
           )
    );

    named!(pub meta_p <Vec<Metadata>>,
           many0!(
               alt!(fnm_p  |
                    snm_p  |
                    fsz_p  |
                    fdt_p  |
                    sdt_p  |
                    hsh_p
               )
           )
    );
}

pub fn from_bytes(bytes : &[u8])
                  -> Result<Vec<Metadata>, Error> {
    use nom::IResult;
    match parsers::meta_p(bytes) {
        IResult::Done(_, res) => Ok(res),
        _                     => Err(Error::ParseError)
    }
}
