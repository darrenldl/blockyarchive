/* This file is translated from implementation of libcrc (https://github.com/lammertb/libcrc)
 *
 * The translation is done by Darren Ldl as part of the blockyarchive project
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

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const CRC_POLY_CCITT: u16 = 0x1021;

fn make_crcccitt_tab() -> [u16; 256] {
    let mut crc: u16;
    let mut c: u16;

    let mut table: [u16; 256] = [0; 256];

    for i in 0u16..256u16 {
        crc = 0;
        c = i << 8;

        for _ in 0..8 {
            if ((crc ^ c) & 0x8000u16) != 0 {
                crc = (crc << 1) ^ CRC_POLY_CCITT;
            } else {
                crc = crc << 1;
            }

            c = c << 1;
        }

        table[i as usize] = crc;
    }

    table
}

fn main() {
    let table = make_crcccitt_tab();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("table.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut table_str = String::from("static CRCCCITT_TABLE : [u16; 256] = [");

    for v in table.iter() {
        let str = format!("{}, ", v);
        table_str.push_str(&str);
    }

    table_str.push_str("];");

    f.write_all(table_str.as_bytes()).unwrap();
}
