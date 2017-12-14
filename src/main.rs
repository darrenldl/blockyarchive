extern crate mpmc_rb;

use mpmc_rb::channel;
use std::thread;

fn main() {
    let (tx, rx) = channel::<String>(1);

    let mut v = vec![];

    for i in 1..2 {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            loop {
                let msg = String::from("abcd");
                println!("Sender {}, sending {}", i, msg);
                tx.send(msg).unwrap();
            }
        });
        v.push(handle);
    }

    for i in 1..5 {
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
