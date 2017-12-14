use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
//use std::cell::RefCell;
use std::collections::VecDeque;

struct Stats<T> {
    // read_pos    : usize,
    // write_pos   : usize,
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

//pub type Sender<T> = Arc<Queue<T>>;
pub struct Sender<T> {
    queue : Arc<Queue<T>>,
}

//pub type Receiver<T> = Arc<Queue<T>>;
pub struct Receiver<T> {
    queue          : Arc<Queue<T>>,
}

impl<T> Stats<T> {
    fn new(requested_size : usize) -> Stats<T> {
        let actual_size = requested_size + 1;

        Stats {
            // read_pos   : 0,
            // write_pos  : 0,
            data       : VecDeque::with_capacity(actual_size),
            capacity   : actual_size,
        }
    }

    fn max_member(&self) -> usize {
        self.capacity - 1
    }

    fn member_count(&self) -> usize {
        //let capacity   = self.capacity;
        /* Since |write_pos - read_pos| < size, we denote this as (1)
                                          * if write_pos - read_pos >= 0 then
         *     (write_pos - read_pos + size) mod size
         *   = (write_pos - read_pos)        mod size
         * else write_pos - read_pos < 0
                                       *   from (1), we have write_pos - read_pos > - size
         *   then, we have write_pos - read_pos + size > 0,
         *   which is well suited for mod(as it's not modulo arithmetic,
                                          *   and thus falls apart with negative value in this use case)
         *     (write_pos - read_pos + size) mod size
         *   = (write_pos - read_pos)        mod size
         */

        //(self.write_pos - self.read_pos + capacity) % capacity
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

macro_rules! finish_run {
    ($res:ident <= $ret:expr) => {{
        $res = Some ($ret);
    }}
}

impl<T> Sender<T> {
    pub fn send(&self, item : T) -> Result<(), T> {
        loop {
            
        }
        while match res { None => true, _ => false } {
            let receiver_count = &self.queue.receiver_count;

            if receiver_count.load(Ordering::Relaxed) == 0 {
                finish_run!(res <= Err (item));
            }
            else {
                let mut stats = self.queue.stats.lock().unwrap();
                match stats.is_full() {
                    true  => run_again!(send =>
                                        stats, self.queue),
                    false => { stats.data.push_front(item);
                               return Ok(()) },
                }
            }
        };
        res.unwrap()
    }
}

impl <T> Receiver<T> {
    pub fn recv(&self) -> Result<T, ()> {
        let mut res = None;
        while match res { None => true, _ => false } {
            let sender_count = &self.queue.sender_count;

            if sender_count.load(Ordering::Relaxed) == 0 {
                finish_run!(res <= Err(()))
            }
            else {
                let mut stats = self.queue.stats.lock().unwrap();
                match stats.is_empty() {
                    true  => run_again!(recv =>
                                        stats, self.queue),
                    false => {
                        match stats.data.pop_back() {
                            Some (x) => finish_run!(res <= Ok (x)),
                            None     => run_again!(recv =>
                                                   stats, self.queue),
                        }
                    }
                }
            }
        };
        res.unwrap()
    }
}

pub fn channel<T>(size : usize) -> (Sender<T>, Receiver<T>){
    let queue = Queue::new(size);
    queue.receiver_count.fetch_add(1, Ordering::Relaxed);
    queue.sender_count.fetch_add(1, Ordering::Relaxed);
    let queue_arc1 = Arc::new(queue);
    let queue_arc2 = Arc::clone(&queue_arc1);

    (Sender {queue : queue_arc1}, Receiver {queue : queue_arc2})
}
