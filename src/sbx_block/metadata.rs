use super::Error;
use crate::misc_utils;
use crate::multihash;
use crate::sbx_specs::{ver_to_data_size, Version};
use crate::time_utils;
use std;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Metadata {
    FNM(String),
    SNM(String),
    FSZ(u64),
    FDT(i64),
    SDT(i64),
    HSH(multihash::HashBytes),
    RSD(u8),
    RSP(u8),
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Metadata::*;

        match self {
            FNM(s) => write!(f, "{}", s),
            SNM(s) => write!(f, "{}", s),
            FSZ(x) => write!(f, "{}", *x),
            FDT(x) | SDT(x) => {
                match (
                    time_utils::i64_secs_to_date_time_string(*x, time_utils::TimeMode::UTC),
                    time_utils::i64_secs_to_date_time_string(*x, time_utils::TimeMode::Local),
                ) {
                    (Some(u), Some(l)) => write!(f, "{} (UTC)  {} (Local)", u, l),
                    _ => write!(f, "Invalid recorded date time"),
                }
            }
            HSH(h) => write!(
                f,
                "{} - {}",
                multihash::hash_type_to_string(h.0),
                misc_utils::bytes_to_lower_hex_string(&h.1)
            ),
            RSD(x) => write!(f, "{}", *x),
            RSP(x) => write!(f, "{}", *x),
        }
    }
}

pub enum UncheckedMetadata {
    FNM(Vec<u8>),
    SNM(Vec<u8>),
    FSZ(u64),
    FDT(i64),
    SDT(i64),
    HSH(multihash::HashBytes),
    RSD(u8),
    RSP(u8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MetadataID {
    FNM,
    SNM,
    FSZ,
    FDT,
    SDT,
    HSH,
    RSD,
    RSP,
}

static PREAMBLE_LEN: usize = 3 + 1;

fn single_info_size(meta: &Metadata) -> usize {
    use self::Metadata::*;
    use std::mem;
    match *meta {
        FNM(ref x) | SNM(ref x) => x.len(),
        FSZ(_) | FDT(_) | SDT(_) => mem::size_of::<u64>(),
        HSH(ref x) => multihash::specs::Param::new(x.0).total_length(),
        RSD(_) | RSP(_) => mem::size_of::<u8>(),
    }
}

fn single_meta_size(meta: &Metadata) -> usize {
    PREAMBLE_LEN + single_info_size(meta)
}

pub fn id_to_bytes(id: MetadataID) -> [u8; 3] {
    use self::MetadataID::*;
    match id {
        FNM => [b'F', b'N', b'M'],
        SNM => [b'S', b'N', b'M'],
        FSZ => [b'F', b'S', b'Z'],
        FDT => [b'F', b'D', b'T'],
        SDT => [b'S', b'D', b'T'],
        HSH => [b'H', b'S', b'H'],
        RSD => [b'R', b'S', b'D'],
        RSP => [b'R', b'S', b'P'],
    }
}

pub fn id_to_str(id: MetadataID) -> &'static str {
    use self::MetadataID::*;
    match id {
        FNM => "FNM",
        SNM => "SNM",
        FSZ => "FSZ",
        FDT => "FDT",
        SDT => "SDT",
        HSH => "HSH",
        RSD => "RSD",
        RSP => "RSP",
    }
}

pub fn meta_to_id(meta: &Metadata) -> MetadataID {
    match *meta {
        Metadata::FNM(_) => MetadataID::FNM,
        Metadata::SNM(_) => MetadataID::SNM,
        Metadata::FSZ(_) => MetadataID::FSZ,
        Metadata::FDT(_) => MetadataID::FDT,
        Metadata::SDT(_) => MetadataID::SDT,
        Metadata::HSH(_) => MetadataID::HSH,
        Metadata::RSD(_) => MetadataID::RSD,
        Metadata::RSP(_) => MetadataID::RSP,
    }
}

fn single_to_bytes(meta: &Metadata, buffer: &mut [u8]) -> Result<usize, ()> {
    let total_size = single_meta_size(meta);
    let info_size = single_info_size(meta);

    if buffer.len() < total_size {
        return Err(());
    }

    use self::Metadata::*;

    // write id
    let id = id_to_bytes(meta_to_id(meta));
    for i in 0..id.len() {
        buffer[i] = id[i];
    }

    // write length
    buffer[3] = info_size as u8;

    let dst = &mut buffer[PREAMBLE_LEN..PREAMBLE_LEN + info_size];

    // write info
    match *meta {
        FNM(ref x) | SNM(ref x) => {
            dst.copy_from_slice(x.as_bytes());
        }
        FSZ(x) => {
            let be_bytes: [u8; 8] = unsafe { std::mem::transmute::<u64, [u8; 8]>(x.to_be()) };
            dst.copy_from_slice(&be_bytes);
        }
        FDT(x) | SDT(x) => {
            let be_bytes: [u8; 8] = unsafe { std::mem::transmute::<i64, [u8; 8]>(x.to_be()) };
            dst.copy_from_slice(&be_bytes);
        }
        HSH(ref x) => {
            multihash::hash_bytes_to_bytes(x, dst);
        }
        RSD(x) | RSP(x) => {
            dst[0] = x;
        }
    }

    Ok(total_size)
}

pub fn to_bytes(meta: &[Metadata], buffer: &mut [u8]) -> Result<(), Error> {
    let mut cur_pos = 0;
    for m in meta.iter() {
        let size_written = match single_to_bytes(m, &mut buffer[cur_pos..]) {
            Ok(x) => x,
            Err(()) => {
                return Err(Error::TooMuchMetadata(meta.to_vec()));
            }
        };

        cur_pos += size_written;
    }

    // fill the rest with padding 0x1A
    for i in cur_pos..buffer.len() {
        buffer[i] = 0x1A;
    }

    Ok(())
}

pub fn make_too_much_meta_err_string(version: Version, meta: &[Metadata]) -> String {
    let msg = make_distribution_string(version, meta);

    format!("Too much metadata, distribution :\n{}", &msg)
}

pub fn make_distribution_string(version: Version, metas: &[Metadata]) -> String {
    let mut string = String::with_capacity(1000);
    string.push_str("|  ID | Info length | Total length |\n");

    let mut overall_total = 0;
    let max_size = ver_to_data_size(version);

    for i in 0..metas.len() {
        let id_str = id_to_str(meta_to_id(&metas[i]));
        let total_size = single_meta_size(&metas[i]);
        let info_size = single_info_size(&metas[i]);

        overall_total += total_size;

        string.push_str(&format!(
            "| {} |      {:6} |       {:6} |\n",
            id_str, info_size, total_size
        ));
    }
    string.push_str("\n");
    string.push_str(&format!("Overall total length : {:6}\n", overall_total));
    string.push_str(&format!("Maximum total length : {:6}", max_size));
    string
}

mod parsers {
    use super::super::super::misc_utils;
    use super::super::super::multihash::parsers::multihash_w_len_p;
    use super::UncheckedMetadata;
    use super::UncheckedMetadata::*;

    use nom::number::complete::be_i64;
    use nom::number::complete::be_u64;
    use nom::number::complete::be_u8;

    macro_rules! make_meta_parser {
        (
            $name:ident, $id:expr, $constructor:path
                => num, $n_must_be:expr, $res_parser:ident
        ) => {
            named!(
                $name<UncheckedMetadata>,
                do_parse!(
                    _id: tag!($id)
                        >> n: be_u8
                        >> res: map_opt!(cond!(n >= 1 && n == $n_must_be, $res_parser), move |x| x)
                        >> ($constructor(res))
                )
            );
        };
        (
            $name:ident, $id:expr, $constructor:path => str
        ) => {
            named!(
                $name<UncheckedMetadata>,
                do_parse!(
                    tag!($id)
                        >> n: be_u8
                        >> res: map_opt!(cond!(n >= 1, take!(n)), move |x| x)
                        >> ($constructor(misc_utils::slice_to_vec(res)))
                )
            );
        };
    }

    make_meta_parser!(fnm_p, b"FNM", FNM => str);
    make_meta_parser!(snm_p, b"SNM", SNM => str);
    make_meta_parser!(fsz_p, b"FSZ", FSZ => num, 8, be_u64);
    make_meta_parser!(fdt_p, b"FDT", FDT => num, 8, be_i64);
    make_meta_parser!(sdt_p, b"SDT", SDT => num, 8, be_i64);
    make_meta_parser!(rsd_p, b"RSD", RSD => num, 1, be_u8);
    make_meta_parser!(rsp_p, b"RSP", RSP => num, 1, be_u8);

    named!(
        hsh_p<UncheckedMetadata>,
        do_parse!(_id: tag!(b"HSH") >> res: multihash_w_len_p >> (HSH(res)))
    );

    named!(pub meta_p <Vec<UncheckedMetadata>>,
           many0!(
               alt!(
                   complete!(fnm_p)
                       | complete!(snm_p)
                       | complete!(fsz_p)
                       | complete!(fdt_p)
                       | complete!(sdt_p)
                       | complete!(hsh_p)
                       | complete!(rsd_p)
                       | complete!(rsp_p)
               )
           )
    );
}

pub fn filter_invalid_metadata(input: Vec<UncheckedMetadata>) -> Vec<Metadata> {
    use self::UncheckedMetadata::*;
    let mut res = Vec::with_capacity(input.len());

    let mut rsd: Option<usize> = None;
    let mut rsp: Option<usize> = None;

    for meta in input.into_iter() {
        let possibly_push: Option<Metadata> = match meta {
            FNM(x) => match String::from_utf8(x) {
                Ok(x) => Some(Metadata::FNM(x)),
                Err(_) => None,
            },
            SNM(x) => match String::from_utf8(x) {
                Ok(x) => Some(Metadata::SNM(x)),
                Err(_) => None,
            },
            FSZ(x) => Some(Metadata::FSZ(x)),
            FDT(x) => Some(Metadata::FDT(x)),
            SDT(x) => Some(Metadata::SDT(x)),
            HSH(h) => Some(Metadata::HSH(h)),
            RSD(d) => {
                if 1 <= d {
                    // only record first occurance
                    if let None = rsd {
                        rsd = Some(d as usize);
                    }
                    Some(Metadata::RSD(d))
                } else {
                    None
                }
            }
            RSP(p) => {
                if 1 <= p {
                    // only record first occurance
                    if let None = rsp {
                        rsp = Some(p as usize);
                    }
                    Some(Metadata::RSP(p))
                } else {
                    None
                }
            }
        };

        match possibly_push {
            None => {}
            Some(x) => res.push(x),
        }
    }

    let res = match (rsd, rsp) {
        (Some(d), Some(p)) if d + p > 256 => {
            // remove all RSD and RSP fields
            res.into_iter()
                .filter(|x| meta_to_id(x) != MetadataID::RSD && meta_to_id(x) != MetadataID::RSP)
                .collect()
        }
        (..) => res,
    };

    res
}

pub fn from_bytes(bytes: &[u8]) -> Result<Vec<Metadata>, Error> {
    match parsers::meta_p(bytes) {
        Ok((_, res)) => Ok(filter_invalid_metadata(res)),
        _ => Err(Error::ParseError),
    }
}

pub fn get_meta_ref_by_id(metas: &[Metadata], id: MetadataID) -> Option<&Metadata> {
    for m in metas.iter() {
        if meta_to_id(m) == id {
            return Some(m);
        }
    }
    None
}

pub fn get_meta_ref_mut_by_id(metas: &mut [Metadata], id: MetadataID) -> Option<&mut Metadata> {
    for m in metas.iter_mut() {
        if meta_to_id(m) == id {
            return Some(m);
        }
    }

    None
}
