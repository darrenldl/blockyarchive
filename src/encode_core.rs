use std::fs::File;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};

use super::{Error, ErrorKind};
use super::Reader;
use super::Writer;
use super::sbx_specs;
use super::sbx_specs::Version;
use super::time;

type SharedStats = Arc<Mutex<Stats>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub sbx_version         : Version,
    pub meta_blocks_written : u64,
    pub data_blocks_written : u64,
    pub data_bytes_encoded  : u64,
    pub start_time          : u64,
    pub data_shards         : usize,
    pub parity_shards       : usize
}

impl Stats {
    pub fn new(version : Version) -> Stats {
        Stats {
            sbx_version         : version,
            meta_blocks_written : 0,
            data_blocks_written : 0,
            data_bytes_encoded  : 0,
            start_time          : time::precise_time_ns(),
            data_shards         : 0,
            parity_shards       : 0
        }
    }

    pub fn time_elapsed(&self) -> u64 {
        time::precise_time_ns() - self.start_time
    }
}

fn make_reader(version : Version,
               in_file : &str,
               stats   : &SharedStats)
               -> Result<JoinHandle<()>, Error> {
    let mut reader = match Reader::new(in_file) {
        Ok(r) => r,
        Err(e) => { return Err(Error::new(ErrorKind::FileError(e))) }
    };
    let stats = Arc::clone(stats);
    Ok(thread::spawn(move || {
        let mut raw_buf : [u8; 4096] = [0; 4096];
    }))
}

fn packer(version : Version)
          -> Result<(), Error> {
    use self::Version::*;
    match version {
        V1  | V2  | V3 => {
        },
        V11 | V12 | V13 => {
        }
    }
    Ok(())
}

fn hasher() {
}

fn writer() -> Result<(), Error> {
    Ok(())
}

pub fn encode_file(in_file  : &str,
                   out_file : &str,
                   version  : Version)
                   -> Result<Stats, Error> {
    let stats : SharedStats =
        Arc::new(Mutex::new(Stats::new(version)));

    let reader = make_reader(version, in_file, &stats);

    Ok(Stats::new(version))
}
