use std::fs::File;
use std::io::{stdin, BufRead, BufReader, Error};

#[derive(Default, Clone)]
pub struct CsvOptions {
    pub(crate) input_file: Option<String>,
    pub(crate) output_file: Option<String>,
    pub(crate) delimiter: Option<char>,
    pub(crate) input_has_headers: Option<bool>,
    pub(crate) output_headers: Option<bool>,
    pub(crate) quote_char: Option<char>,
    pub(crate) escape_char: Option<char>,
    pub(crate) trim_fields: Option<bool>,
    pub(crate) flexible: Option<bool>,
    pub(crate) comment_char: Option<char>,
}

impl CsvOptions {
    pub(crate) fn get_input(&self) -> Result<Box<dyn BufRead>, Error> {
        if let Some(file) = &self.input_file {
            Ok(Box::new(BufReader::new(File::open(file)?)))
        } else {
            Ok(Box::new(BufReader::new(stdin())))
        }
    }
}

impl CsvOptions {
    pub fn new() -> Self {
        Default::default()
    }
}