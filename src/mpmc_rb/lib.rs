use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::collections::VecDeque;
use std::cmp::max;

struct Stats<T> {
    data        : VecDeque<T>,
    capacity    : usize,
}

struct Queue<T> {
    send_cond      : Condvar,
    send_cond_lock : Mutex<bool>,
    recv_cond      : Condvar,
    recv_cond_lock : Mutex<bool>,
    sender_count   : AtomicUsize,
    receiver_count : AtomicUsize,
    stats          : Mutex<Stats<T>>,
}

pub type SendError<T> = mpsc::SendError<T>;

pub struct Sender<T> {
    queue : Arc<Queue<T>>,
}

pub struct Receiver<T> {
    queue          : Arc<Queue<T>>,
}

impl<T> Stats<T> {
    fn new(requested_size : usize) -> Stats<T> {
        let requested_size = max(1, requested_size);
        let actual_size    = requested_size + 1;

        Stats {
            data       : VecDeque::with_capacity(actual_size),
            capacity   : actual_size,
        }
    }

    fn max_member(&self) -> usize {
        self.capacity - 1
    }

    fn member_count(&self) -> usize {
        self.data.len()
    }

    fn is_full(&self) -> bool {
        self.member_count() == self.max_member()
    }

    fn is_empty(&self) -> bool {
        self.member_count() == 0
    }
}

fn wait_for(cond : &Condvar, lock : &Mutex<bool>) {
    let mut okay = lock.lock().unwrap();
    while !*okay {
        okay = cond.wait(okay).unwrap();
    }
    *okay = false;
}

impl<T> Queue<T> {
    fn new(requested_size : usize) -> Queue<T> {
        Queue {
            send_cond      : Condvar::new(),
            send_cond_lock : Mutex::new(false),
            recv_cond      : Condvar::new(),
            recv_cond_lock : Mutex::new(false),
            sender_count   : AtomicUsize::new(0),
            receiver_count : AtomicUsize::new(0),
            stats          : Mutex::new(Stats::new(requested_size)),
        }
    }

    fn notify_single_sender(&self) {
        let mut okay = self.send_cond_lock.lock().unwrap();
        *okay = true;
        self.send_cond.notify_one();
    }

    fn notify_single_receiver(&self) {
        let mut okay = self.recv_cond_lock.lock().unwrap();
        *okay = true;
        self.recv_cond.notify_one();
    }

    fn wait_for_send(&self) {
        wait_for(&self.send_cond, &self.send_cond_lock);
    }

    fn wait_for_recv(&self) {
        wait_for(&self.recv_cond, &self.recv_cond_lock);
    }
}

macro_rules! run_again {
    (recv => $lock:expr, $queue:expr) => {{
        drop($lock);
        $queue.wait_for_recv();
    }};
    (send => $lock:expr, $queue:expr) => {{
        drop($lock);
        $queue.wait_for_send();
    }}
}

impl<T> Sender<T> {
    pub fn send(&self, item : T) -> Result<(), T> {
        loop {
            let receiver_count = &self.queue.receiver_count;

            if receiver_count.load(Ordering::SeqCst) == 0 {
                break Err (item);
            }
            else {
                let mut stats = self.queue.stats.lock().unwrap();
                match stats.is_full() {
                    true  => run_again!(send =>
                                        stats, self.queue),
                    false => { stats.data.push_front(item);
                               self.queue.notify_single_receiver();
                               break Ok(()) },
                }
            }
        }
    }
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, ()> {
        loop {
            let sender_count = &self.queue.sender_count;

            if sender_count.load(Ordering::SeqCst) == 0 {
                break Err(());
            }
            else {
                let mut stats = self.queue.stats.lock().unwrap();
                match stats.is_empty() {
                    true  => run_again!(recv =>
                                        stats, self.queue),
                    false => {
                        match stats.data.pop_back() {
                            Some (x) => { self.queue.notify_single_sender();
                                          break Ok (x) },
                            None     => run_again!(recv =>
                                                   stats, self.queue),
                        }
                    }
                }
            }
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone (&self) -> Sender<T> {
        let new_arc = Arc::clone(&self.queue);

        self.queue.sender_count.fetch_add(1, Ordering::SeqCst);

        Sender { queue : new_arc }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone (&self) -> Receiver<T> {
        let new_arc = Arc::clone(&self.queue);

        self.queue.receiver_count.fetch_add(1, Ordering::SeqCst);

        Receiver { queue : new_arc }
    }
}

impl<T> Drop for Sender<T> {
    fn drop (&mut self) {
        self.queue.sender_count.fetch_sub(1, Ordering::SeqCst);
    }
}

impl<T> Drop for Receiver<T> {
    fn drop (&mut self) {
        self.queue.receiver_count.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn channel<T>(size : usize) -> (Sender<T>, Receiver<T>){
    let queue = Queue::new(size);
    queue.receiver_count.fetch_add(1, Ordering::SeqCst);
    queue.sender_count.fetch_add(1, Ordering::SeqCst);
    let queue_arc1 = Arc::new(queue);
    let queue_arc2 = Arc::clone(&queue_arc1);

    (Sender {queue : queue_arc1}, Receiver {queue : queue_arc2})
}
