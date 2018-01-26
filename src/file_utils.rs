use std::fs;
use super::Error;

use super::file_reader::FileReader;
use super::file_error::adapt_to_err;

pub fn get_file_metadata(file : &str) -> Result<fs::Metadata, Error> {
    let reader = adapt_to_err(FileReader::new(&param.in_file))?;
    adapt_to_err(reader.metadata())?
}
