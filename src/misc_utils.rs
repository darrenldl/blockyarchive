use std::cell::Cell;
use std::sync::mpsc::{channel, sync_channel, Sender, SyncSender, Receiver};

use super::sbx_specs::{SBX_SCAN_BLOCK_SIZE};
use super::integer_utils::IntegerUtils;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidHexString,
    InvalidLen
}

fn is_valid_hex_char(chr : u8) -> bool {
    (0x30 <= chr && chr <= 0x39)
        || (0x41 <= chr && chr <= 0x46)
        || (0x61 <= chr && chr <= 0x66)
}

fn hex_char_to_value(chr : u8) -> u8 {
    if 0x30 <= chr && chr <= 0x39 {
        chr - 0x30
    }
    else if 0x41 <= chr && chr <= 0x46 {
        chr - 0x41 + 0x0A
    }
    else if 0x61 <= chr && chr <= 0x66 {
        chr - 0x61 + 0x0A
    }
    else { panic!() }
}

pub fn hex_string_to_bytes(string : &str) -> Result<Box<[u8]>, Error> {
    if string.len() % 2 == 0 {
        let string = string.as_bytes();
        for c in string {
            if !is_valid_hex_char(*c) {
                return Err(Error::InvalidHexString) }
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
        Err(Error::InvalidLen)
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

pub fn make_channel_for_ctx<T>() -> (Sender<T>, Cell<Option<Receiver<T>>>) {
    let (tx, rx) = channel();

    let rx = Cell::new(Some(rx));

    (tx, rx)
}

pub fn make_sync_channel_for_ctx<T>(size : usize) -> (SyncSender<T>, Cell<Option<Receiver<T>>>) {
    let (tx, rx) = sync_channel(size);

    let rx = Cell::new(Some(rx));

    (tx, rx)
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

pub fn calc_required_len_and_seek_to_from_byte_range
    (from_byte         : Option<u64>,
     to_byte           : Option<u64>,
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
    let to_byte = match to_byte {
        None    => last_possible_pos,
        Some(n) => u64::ensure_at_most(u64::ensure_at_least(n,
                                                            from_byte),
                                       last_possible_pos)
    };
    // bytes_so_far only affects seek_to
    RequiredLenAndSeekTo { required_len : to_byte - from_byte + 1,
                           seek_to      : align(from_byte + bytes_so_far) }
}

pub fn make_path (path_parts : &[String]) -> String {
    fn strip_slash(string : &str) -> &str {
        let str_len = string.len();
        match str_len {
            0 => string,
            1 => { if &string[0..1] == "/" { ""     }
                   else                    { string } },
            _ => { let char_last     = &string[str_len - 1..];
                   let char_2nd_last = &string[str_len - 2..];
                   if char_last == "/" && char_2nd_last != "\\" {
                       &string[0..str_len - 1]
                   } else {
                       string
                   }
            }
        }
    }

    let mut string = String::with_capacity(100);
    for path in path_parts.iter() {
        string.push_str(strip_slash(path));
    }
    string
}
