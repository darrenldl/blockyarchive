/* This file is translated from implementation of libcrc (https://github.com/lammertb/libcrc)
 *
 * The translation is done by Darren Ldl as part of the rust-SeqBox project
 *
 * The translated source code is under the same license as stated and used by the
 * original file (MIT License)
 *
 * Below is the original info text from crcccitt.c
 *
 * Library: libcrc
 * File:    src/crcccitt.c
 * Author:  Lammert Bies
 *
 * This file is licensed under the MIT License as stated below
 *
 * Copyright (c) 1999-2016 Lammert Bies
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 * 
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * Description
 * -----------
 * The module src/crcccitt.c contains routines which are used to calculate the
 * CCITT CRC values of a string of bytes.
 */

include!(concat!(env!("OUT_DIR"), "/table.rs"));

pub fn crc_ccitt_generic (input : &[u8], start_val : u16) -> u16 {
    let mut crc : u16 = start_val;

    for c in input {
        let c_u16 :u16 = *c as u16;

        crc =
            (crc << 8)
            ^
            CRCCCITT_TABLE[ (((crc >> 8) ^ c_u16) & 0x00FFu16) as usize ];
    }

    crc
}

#[cfg(test)]
mod tests {
    use super::crc_ccitt_generic;

    #[test]
    fn basic_value_tests_0xffff() {
        assert_eq!(crc_ccitt_generic(b"a", 0xFFFF), 0x9D77);
        assert_eq!(crc_ccitt_generic(b"abcd", 0xFFFF), 0x2CF6);
        assert_eq!(crc_ccitt_generic(b"0", 0xFFFF), 0xD7A3);
        assert_eq!(crc_ccitt_generic(b"0123", 0xFFFF), 0x3F7B);
    }

    #[test]
    fn basic_value_tests_0x1d0f() {
        assert_eq!(crc_ccitt_generic(b"a", 0x1D0f), 0xB01B);
        assert_eq!(crc_ccitt_generic(b"abcd", 0x1D0f), 0xA626);
        assert_eq!(crc_ccitt_generic(b"0", 0x1D0F), 0xFACF);
        assert_eq!(crc_ccitt_generic(b"0123", 0x1D0F), 0xB5AB);
    }
}
