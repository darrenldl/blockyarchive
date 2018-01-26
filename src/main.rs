#![allow(dead_code)]

#[macro_use]
extern crate nom;

extern crate time;

extern crate pond;

extern crate smallvec;
use smallvec::SmallVec;

#[macro_use]
extern crate reed_solomon_erasure;
use reed_solomon_erasure::ReedSolomon;

macro_rules! worker_stop {
    (
        graceful => $tx_error:path, $shutdown_flag:path
    ) => {{
        use std::sync::atomic::Ordering;
        $tx_error.send(None).unwrap();
        $shutdown_flag.store(true, Ordering::Relaxed);
        break;
    }};
    (
        graceful_ret => $tx_error:path, $shutdown_flag:path
    ) => {{
        use std::sync::atomic::Ordering;
        $tx_error.send(None).unwrap();
        $shutdown_flag.store(true, Ordering::Relaxed);
        return;
    }};
    (
        graceful_if ($cond:expr) =>$tx_error:path, $shutdown_flag:path
    ) => {{
        if $cond {
            worker_stop!(graceful => $tx_error, $shutdown_flag)
        }
    }};
    (
        graceful_if_ret ($cond:expr) =>$tx_error:path, $shutdown_flag:path
    ) => {{
        if $cond {
            worker_stop!(graceful_ret => $tx_error, $shutdown_flag)
        }
    }};
    (
        graceful_if_shutdown => $tx_error:path, $shutdown_flag:path
    ) => {{
        use std::sync::atomic::Ordering;
        worker_stop!(graceful_if ($shutdown_flag.load(Ordering::Relaxed)) =>
                     $tx_error, $shutdown_flag)
    }};
    (
        graceful_if_shutdown_ret => $tx_error:path, $shutdown_flag:path
    ) => {{
        use std::sync::atomic::Ordering;
        worker_stop!(graceful_if_ret ($shutdown_flag.load(Ordering::Relaxed)) =>
                     $tx_error, $shutdown_flag)
    }};
    (
        with_error $error:expr => $tx_error:path, $shutdown_flag:path
    ) => {{
        use std::sync::atomic::Ordering;
        $tx_error.send(Some($error)).unwrap();
        $shutdown_flag.store(true, Ordering::Relaxed);
        break;
    }};
    (
        with_error_if_fail ($expr:expr) => $tx_error:path, $shutdown_flag:path
    ) => {{
        match $expr {
            Ok(res) => res,
            Err(e)  => worker_stop!(with_error e => $tx_error, $shutdown_flag)
        }
    }};
    (
        with_error_ret $error:expr => $tx_error:path, $shutdown_flag:path
    ) => {{
        use std::sync::atomic::Ordering;
        $tx_error.send(Some($error)).unwrap();
        $shutdown_flag.store(true, Ordering::Relaxed);
        return;
    }}
}

macro_rules! send {
    (
        no_back_off $item:expr => $sender:ident, $tx_error:path, $shutdown_flag:path
    ) => {{
        match $sender.send($item) {
            Ok(()) => {},
            Err(_) => worker_stop!(graceful => $tx_error, $shutdown_flag)
        }
    }};
    (
        no_back_off_ret $item:expr => $sender:ident, $tx_error:path, $shutdown_flag:path
    ) => {{
        match $sender.send($item) {
            Ok(()) => {},
            Err(_) => worker_stop!(graceful_ret => $tx_error, $shutdown_flag)
        }
    }};
    (
        back_off_millis $time:expr, $item:expr => $sender:ident, $tx_error:path, $shutdown_flag:path
    ) => {{
        use std::time::Duration;
        use std::sync::mpsc::TrySendError;
        match $sender.try_send($item) {
            Ok(()) => None,
            Err(TrySendError::Full(b)) => {
                thread::sleep(Duration::from_millis($time));
                worker_stop!(graceful_if_shutdown => $tx_error, $shutdown_flag);
                Some(b)
            },
            Err(TrySendError::Disconnected(_)) =>
                worker_stop!(graceful => $tx_error, $shutdown_flag)
        }
    }};
    (
        back_off $item:expr => $sender:ident, $tx_error:path, $shutdown_flag:path
    ) => {{
        send!(back_off_millis 100, $item => $sender, $tx_error, $shutdown_flag)
    }}
}

macro_rules! recv {
    (
        no_timeout => $receiver:ident, $tx_error:path, $shutdown_flag:path
    ) => {{
        match $receiver.recv() {
            Ok(item) => item,
            Err(_)   => worker_stop!(graceful => $tx_error, $shutdown_flag)
        }
    }};
    (
        timeout_millis $timeout:expr => $receiver:ident, $tx_error:path, $shutdown_flag:path
    ) => {{
        use std::time::Duration;
        use std::sync::mpsc::RecvTimeoutError;
        match $receiver.recv_timeout(Duration::from_millis($timeout)) {
            Ok(item)                            => item,
            Err(RecvTimeoutError::Timeout)      => {
                worker_stop!(graceful_if_shutdown => $tx_error, $shutdown_flag);
                continue;
            },
            Err(RecvTimeoutError::Disconnected) => {
                worker_stop!(graceful =>
                             $tx_error, $shutdown_flag );
            }
        }
    }};
    (
        timeout => $receiver:ident, $tx_error:path, $shutdown_flag:path
    ) => {{
        recv!(timeout_millis 1000 => $receiver, $tx_error, $shutdown_flag)
    }}
}

mod file_error;
use file_error::FileError;

mod general_error;
use general_error::Error;
use general_error::ErrorKind;

mod multihash;
mod multihash_test;
mod misc_utils;
mod misc_utils_test;
mod rand_utils;
mod sbx_block;
mod sbx_specs;

mod encode_core;
mod decode_core;
mod rescue_core;
mod repair_core;
mod show_core;
mod sort_core;

mod file_reader;
use file_reader::FileReader;
mod file_writer;
use file_writer::FileWriter;

mod worker;

fn main () {
    use encode_core::Param;
    let param = Param {
        version : sbx_specs::Version::V1,
        file_uid : [0, 1, 2, 3, 4, 5],
        rs_enabled : true,
        rs_data    : 10,
        rs_parity  : 2,
        hash_enabled : true,
        hash_type  : multihash::HashType::SHA256,
        in_file    : String::from("test"),
        out_file   : String::from("test.sbx")
    };
    match encode_core::encode_file(&param) {
        Ok(_)  => {},
        Err(e) => println!("Error : {}", e)
    }
}
