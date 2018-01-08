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

            println!("{}, {}", l_chr_val, r_chr_val);
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
