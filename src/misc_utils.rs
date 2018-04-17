#![allow(dead_code)]

use sbx_specs::{SBX_SCAN_BLOCK_SIZE};
use integer_utils::IntegerUtils;

use std::path::PathBuf;

use smallvec::SmallVec;

#[derive(Debug, PartialEq)]
pub enum HexError {
    InvalidHexString,
    InvalidLen
}

fn is_valid_hex_char(chr : u8) -> bool {
    (b'0' <= chr && chr <= b'9')
        || (b'A' <= chr && chr <= b'F')
        || (b'a' <= chr && chr <= b'f')
}

fn hex_char_to_value(chr : u8) -> u8 {
    if      b'0' <= chr && chr <= b'9' {
        chr - b'0'
    }
    else if b'A' <= chr && chr <= b'F' {
        chr - b'A' + 0x0A
    }
    else if b'a' <= chr && chr <= b'f' {
        chr - b'a' + 0x0A
    }
    else { panic!() }
}

pub fn hex_string_to_bytes(string : &str) -> Result<Box<[u8]>, HexError> {
    if string.len() % 2 == 0 {
        let string = string.as_bytes();
        for c in string {
            if !is_valid_hex_char(*c) {
                return Err(HexError::InvalidHexString); }
        }

        let mut result = Vec::with_capacity(string.len() % 2);

        for i in (0..string.len()).filter(|x| x % 2 == 0) {
            let l_chr_val = hex_char_to_value(string[i]);
            let r_chr_val = hex_char_to_value(string[i+1]);

            result.push(l_chr_val * 0x10 + r_chr_val);
        }
        Ok(result.into_boxed_slice())
    }
    else {
        Err(HexError::InvalidLen)
    }
}

pub fn bytes_to_lower_hex_string(bytes : &[u8]) -> String {
    let strs: Vec<String> = bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    strs.join("")
}

pub fn bytes_to_upper_hex_string(bytes : &[u8]) -> String {
    let strs: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    strs.join("")
}

pub fn slice_to_vec<T> (slice : &[T]) -> Vec<T>
    where T : Clone
{
    let mut v : Vec<T> = Vec::with_capacity(slice.len());

    for s in slice.iter() {
        v.push(s.clone());
    }

    v
}

pub fn slice_to_boxed<T> (slice : &[T]) -> Box<[T]>
    where T : Clone
{
    slice_to_vec(slice).into_boxed_slice()
}

pub fn ignore<T1, T2>(_ : Result<T1, T2>) {}

pub fn f64_max (v1 : f64, v2 : f64) -> f64 {
    if v1 < v2 { v2 }
    else       { v1 }
}

pub struct RequiredLenAndSeekTo {
    pub required_len : u64,
    pub seek_to      : u64,
}

pub fn calc_required_len_and_seek_to_from_byte_range_inc
    (from_byte         : Option<u64>,
     to_byte_inc       : Option<u64>,
     force_misalign    : bool,
     bytes_so_far      : u64,
     last_possible_pos : u64) -> RequiredLenAndSeekTo
{
    let multiple_of = SBX_SCAN_BLOCK_SIZE as u64;
    let align = |x : u64| -> u64 {
        if force_misalign { x }
        else              { u64::round_down_to_multiple(x,
                                                        multiple_of) }
    };
    let from_byte = match from_byte {
        None    => 0,
        Some(n) => align(u64::ensure_at_most(n,
                                             last_possible_pos))
    };
    let to_byte = match to_byte_inc {
        None    => last_possible_pos,
        Some(n) => u64::ensure_at_most(u64::ensure_at_least(n,
                                                            from_byte),
                                       last_possible_pos)
    };
    // bytes_so_far only affects seek_to
    let seek_to = u64::ensure_at_most(align(from_byte + bytes_so_far),
                                      last_possible_pos);
    RequiredLenAndSeekTo { required_len : to_byte - from_byte + 1,
                           seek_to                                 }
}

pub fn make_path(path_parts : &[&str]) -> String {
    fn strip_slash_prefix(string : &str) -> &str {
        let str_len = string.len();
        match str_len {
            0 => string,
            1 => match &string[0..1] { "/" => "", _ => string },
            _ => { let char_1st = &string[0..1];
                   if char_1st == "/" || char_1st == "\\" {
                       &string[1..]
                   } else {
                       string
                   }
            }
        }
    }
    fn strip_slash_suffix(string : &str) -> &str {
        let str_len = string.len();
        match str_len {
            0 => string,
            1 => match &string[0..1] { "/" => "", _ => string },
            _ => { let char_last     = &string[str_len - 1..];
                   let char_2nd_last = &string[str_len - 2..];
                   if (char_last == "/" || char_last == "\\")
                   && char_2nd_last != "\\"
                   {
                       &string[0..str_len - 1]
                   } else {
                       string
                   }
            }
        }
    }

    let mut path_buf = PathBuf::new();
    for i in 0..path_parts.len() {
        if i == 0 {
            path_buf.push(path_parts[i]);
        } else {
            let res =
                strip_slash_prefix(
                    strip_slash_suffix(path_parts[i]));
            if res.len() > 0 {
                path_buf.push(res);
            }
        }
    }
    path_buf.to_string_lossy().to_string()
}

pub fn buffer_is_blank(buf : &[u8]) -> bool {
    for p in buf.iter() {
        if *p != 0 { return false; }
    }

    true
}

pub fn to_camelcase(string : &str) -> String {
    let mut res = String::with_capacity(100);

    let split : SmallVec<[&str; 16]> = string.split(' ').collect();

    res.push_str(&split[0].to_lowercase());
    for s in &split[1..] {
        let mut s = s.chars();
        match s.next() {
            None => {},
            Some(c) => {
                let x : String = c.to_uppercase().chain(s).collect();
                res.push_str(&x);
            }
        };
    };

    res
}

pub fn strip_front_end_spaces(string : &str) -> &str {
    let mut start   = None;
    let mut end_inc = 0;
    for (i, c) in string.chars().enumerate() {
        if c != ' ' {
            if let None = start {
                start = Some(i);
            }
            end_inc = i;
        }
    }

    &string[start.unwrap_or(0)..end_inc]
}

pub fn split_key_val_pair(string : &str) -> (&str, &str) {
    let mut spot = 0;
    for (i, c) in string.chars().enumerate() {
        if c == ':' {
            spot = i;
            break;
        }
    }

    (strip_front_end_spaces(&string[0..spot]),
     strip_front_end_spaces(&string[spot+1..]))
}

pub fn escape_quotes(string : &str) -> String {
    let mut res = String::with_capacity(100);
    for c in string.chars() {
        if c == '"' {
            res.push('\\');
        }
        res.push(c);
    }
    res
}
