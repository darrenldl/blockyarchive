#![allow(dead_code)]
use crate::integer_utils::IntegerUtils;
use crate::sbx_specs::SBX_SCAN_BLOCK_SIZE;
use smallvec::SmallVec;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeEnd<T> {
    Inc(T),
    Exc(T),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PositionOrLength<T> {
    Pos(T),
    Len(T),
}

#[derive(Debug, PartialEq)]
pub enum HexError {
    InvalidHexString,
    InvalidLen,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MultiPassType {
    OverwriteAll,
    SkipGood,
}

fn is_valid_hex_char(chr: u8) -> bool {
    (b'0' <= chr && chr <= b'9') || (b'A' <= chr && chr <= b'F') || (b'a' <= chr && chr <= b'f')
}

fn hex_char_to_value(chr: u8) -> u8 {
    if b'0' <= chr && chr <= b'9' {
        chr - b'0'
    } else if b'A' <= chr && chr <= b'F' {
        chr - b'A' + 0x0A
    } else if b'a' <= chr && chr <= b'f' {
        chr - b'a' + 0x0A
    } else {
        panic!()
    }
}

pub fn hex_string_to_bytes(string: &str) -> Result<Box<[u8]>, HexError> {
    if string.len() % 2 == 0 {
        let string = string.as_bytes();
        for c in string {
            if !is_valid_hex_char(*c) {
                return Err(HexError::InvalidHexString);
            }
        }

        let mut result = Vec::with_capacity(string.len() % 2);

        for i in (0..string.len()).filter(|x| x % 2 == 0) {
            let l_chr_val = hex_char_to_value(string[i]);
            let r_chr_val = hex_char_to_value(string[i + 1]);

            result.push(l_chr_val * 0x10 + r_chr_val);
        }
        Ok(result.into_boxed_slice())
    } else {
        Err(HexError::InvalidLen)
    }
}

pub fn bytes_to_lower_hex_string(bytes: &[u8]) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    strs.join("")
}

pub fn bytes_to_upper_hex_string(bytes: &[u8]) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs.join("")
}

pub fn slice_to_vec<T>(slice: &[T]) -> Vec<T>
where
    T: Clone,
{
    let mut v: Vec<T> = Vec::with_capacity(slice.len());

    for s in slice.iter() {
        v.push(s.clone());
    }

    v
}

pub fn slice_to_boxed<T>(slice: &[T]) -> Box<[T]>
where
    T: Clone,
{
    slice_to_vec(slice).into_boxed_slice()
}

pub fn ignore<T1, T2>(_: Result<T1, T2>) {}

pub fn f64_max(v1: f64, v2: f64) -> f64 {
    if v1 < v2 {
        v2
    } else {
        v1
    }
}

pub fn f64_min(v1: f64, v2: f64) -> f64 {
    if v1 < v2 {
        v1
    } else {
        v2
    }
}

pub struct RequiredLenAndSeekTo {
    pub required_len: u64,
    pub seek_to: u64,
}

pub fn calc_required_len_and_seek_to_from_byte_range(
    from_byte: Option<u64>,
    to_byte: Option<RangeEnd<u64>>,
    force_misalign: bool,
    bytes_so_far: u64,
    last_possible_pos_or_len: PositionOrLength<u64>,
    multiple_of: Option<u64>,
) -> RequiredLenAndSeekTo {
    let last_possible_pos = match last_possible_pos_or_len {
        PositionOrLength::Pos(x) => x,
        PositionOrLength::Len(x) => {
            if x == 0 {
                0
            } else {
                x - 1
            }
        }
    };

    let multiple_of = match multiple_of {
        Some(x) => x,
        None => SBX_SCAN_BLOCK_SIZE as u64,
    };
    let align = |x: u64| -> u64 {
        if force_misalign {
            x
        } else {
            u64::round_down_to_multiple(x, multiple_of)
        }
    };
    let from_byte = match from_byte {
        None => 0,
        Some(n) => align(u64::ensure_at_most(n, last_possible_pos)),
    };
    let to_byte_inc = match to_byte {
        None => last_possible_pos,
        Some(n) => match n {
            RangeEnd::Inc(n) => {
                u64::ensure_at_most(u64::ensure_at_least(n, from_byte), last_possible_pos)
            }
            RangeEnd::Exc(n) => {
                u64::ensure_at_most(u64::ensure_at_least(n - 1, from_byte), last_possible_pos)
            }
        },
    };
    // bytes_so_far only affects seek_to
    let seek_to = u64::ensure_at_most(align(from_byte + bytes_so_far), last_possible_pos);
    RequiredLenAndSeekTo {
        required_len: to_byte_inc - from_byte + 1,
        seek_to,
    }
}

pub fn make_path(path_parts: &[&str]) -> String {
    let mut path_buf = PathBuf::new();

    for part in path_parts.into_iter() {
        path_buf.push(part);
    }

    path_buf.to_string_lossy().to_string()
}

pub fn buffer_is_blank(buf: &[u8]) -> bool {
    for p in buf.iter() {
        if *p != 0 {
            return false;
        }
    }

    true
}

pub fn fill_zeros(buf: &mut [u8]) {
    for p in buf.iter_mut() {
        *p = 0;
    }
}

pub fn to_camelcase(string: &str) -> String {
    let mut res = String::with_capacity(100);

    let split: SmallVec<[&str; 16]> = string.split(' ').collect();

    res.push_str(&split[0].to_lowercase());
    for s in &split[1..] {
        let s = strip_front_end_chars(s, " ()");
        let mut s = s.chars();
        match s.next() {
            None => {}
            Some(c) => {
                let x: String = c.to_uppercase().chain(s).collect();
                res.push_str(&x);
            }
        };
    }

    res
}

pub fn strip_front_end_chars<'a, 'b>(string: &'a str, chars: &'b str) -> &'a str {
    let mut start = None;
    let mut end_inc = 0;
    for (i, c) in string.chars().enumerate() {
        if let None = chars.find(c) {
            if let None = start {
                start = Some(i);
            }
            end_inc = i;
        }
    }

    if end_inc < string.len() {
        &string[start.unwrap_or(0)..end_inc + 1]
    } else {
        &string[start.unwrap_or(0)..string.len()]
    }
}

pub fn escape_quotes(string: &str) -> String {
    let mut res = String::with_capacity(100);
    for c in string.chars() {
        if c == '"' {
            res.push('\\');
        }
        res.push(c);
    }
    res
}
