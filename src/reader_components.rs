use super::Error;

const READ_RETRIES : usize = 5;

pub struct ReadReq<'a> {
    pos  : Option(usize),
    data : &'a [u8]
}

pub fn work(file : FILE) -> Result<(), Error> {
    Ok(())
}
