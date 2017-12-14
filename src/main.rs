extern crate mpmc_rb;

use mpmc_rb::channel;
use std::{thread, time};
use std::time::SystemTime;

fn main() {
    let (tx, rx) = channel::<String>(1000);

    let mut v = vec![];

    for i in 1..1000 {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            loop {
                let msg = format!("{:?}", SystemTime::now());
                //println!("Sender {}, sending {}", i, msg);
                tx.send(msg).unwrap();
                thread::sleep(time::Duration::from_millis(10));
            }
        });
        v.push(handle);
    }

    for i in 1..1000 {
        let rx = rx.clone();
        let handle = thread::spawn(move || {
            loop {
                println!("Receiver {}, received {:?}", i, rx.recv());
            }
        });
        v.push(handle);
    }

    for i in v.into_iter() {
        i.join().unwrap();
    }
}
