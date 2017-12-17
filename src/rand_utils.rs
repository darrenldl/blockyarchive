extern crate rand;

pub fn fill_random_bytes(bytes : &mut [u8]) {
    for i in 0..bytes.len() {
        bytes[i] = rand::random::<u8>();
    }
}

pub fn make_random_bytes(size : usize) -> Box<[u8]> {
    let mut bytes : Box<[u8]> = vec![0u8; size].into_boxed_slice();
    fill_random_bytes(&mut bytes);
    bytes
}
