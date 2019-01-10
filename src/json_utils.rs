use crate::misc_utils::strip_front_end_chars;

pub fn split_key_val_pair(string: &str) -> (&str, &str) {
    let mut spot = 0;
    for (i, c) in string.chars().enumerate() {
        if c == ':' {
            spot = i;
            break;
        }
    }

    (
        strip_front_end_chars(&string[0..spot], " "),
        strip_front_end_chars(&string[spot + 1..], " "),
    )
}
