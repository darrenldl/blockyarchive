use std::fs;
use super::Error;

use super::file_reader::FileReader;

pub fn get_file_metadata(file : &str) -> Result<fs::Metadata, Error> {
    let reader = FileReader::new(file)?;
    reader.metadata()
}
