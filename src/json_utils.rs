#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JSONContext {
    pub json_enabled : bool,
    pub first_item   : bool,
}

impl JSONContext {
    pub fn new(json_enabled : bool) -> JSONContext {
        JSONContext {
            json_enabled,
            first_item : true,
        }
    }
}

pub fn split_key_val_pair(string : &str) -> (&str, &str) {
    let mut spot = 0;
    for (i, c) in string.chars().enumerate() {
        if c == ':' {
            spot = i;
            break;
        }
    }

    (strip_front_end_chars(&string[0..spot],  " "),
     strip_front_end_chars(&string[spot+1..], " "))
}
