#![cfg(test)]
use super::*;

#[test]
#[should_panic]
fn cancel_slot_panics_when_empty1() {
    let mut buffer = RescueBuffer::new(2);

    buffer.cancel_slot();
}

#[test]
#[should_panic]
fn cancel_slot_panics_when_empty2() {
    let mut buffer = RescueBuffer::new(2);

    let _ = buffer.get_slot();

    buffer.cancel_slot();
    buffer.cancel_slot();
}

quickcheck! {
    #[should_panic]
    fn qc_cancel_slot_panics_when_empty(size: usize, cancels: usize) -> bool {
        let size = 1 + size % 1000;
        let cancels = 1 + cancels % 1000;

        let cancels = std::cmp::min(size, cancels);

        let mut buffer = RescueBuffer::new(2);

        for _ in 0..cancels {
            let _ = buffer.get_slot();
        }

        for _ in 0..cancels+1 {
            buffer.cancel_slot();
        }

        true
    }

    fn qc_cancel_slot_when_not_empty(size: usize,
                                     cancels: usize) -> bool {
        let size = 1 + size % 1000;
        let cancels = 1 + cancels % 1000;

        let cancels = std::cmp::min(size, cancels);

        let mut buffer = RescueBuffer::new(size);

        for _ in 0..cancels {
            let _ = buffer.get_slot();
        }

        for _ in 0..cancels {
            buffer.cancel_slot();
        }

        true
    }

    fn qc_get_slot_result(size: usize,
                          tries: usize) -> bool {
        let size = 1 + size % 1000;
        let tries = 2 + tries % 100;

        let mut buffer = RescueBuffer::new(size);

        for _ in 0..tries {
            for _ in 0..size {
                match buffer.get_slot() {
                    Some(_) => {}
                    None => panic!()
                }
            }

            match buffer.get_slot() {
                Some(_) => panic!(),
                None => {}
            }

            for _ in 0..size {
                buffer.cancel_slot()
            }
        }

        true
    }
}
