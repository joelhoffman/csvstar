use std::fs::File;
use std::io::{stdin, BufRead, BufReader, BufWriter, Error, Read, Write};
use std::{error, io};

#[derive(Default, Clone)]
pub struct CsvOptions {
    pub(crate) input_file: Option<String>,
    pub output_file: Option<String>,
    pub(crate) delimiter: Option<char>,
    pub input_has_headers: Option<bool>,
    pub output_headers: Option<bool>,
    pub(crate) quote_char: Option<char>,
    pub(crate) escape_char: Option<char>,
    pub(crate) trim_fields: Option<bool>,
    pub(crate) flexible: Option<bool>,
    pub(crate) comment_char: Option<char>,
}

impl CsvOptions {
    pub fn get_input_file(&self) -> Result<Box<dyn BufRead>, Error> {
        if let Some(file) = &self.input_file {
            Ok(Box::new(BufReader::new(File::open(file)?)))
        } else {
            Ok(Box::new(BufReader::new(stdin())))
        }
    }

    pub fn get_output_file(&self) -> Result<Box<BufWriter<dyn Write>>, Box<dyn error::Error>> {
        let csv_file_handle: Box<BufWriter<dyn Write>>;
        if let Some(file) = &self.output_file {
            csv_file_handle = Box::new(BufWriter::new(File::create(file)?));
        } else {
            csv_file_handle = Box::new(BufWriter::new(io::stdout()));
        }
        Ok(csv_file_handle)
    }
}

impl CsvOptions {
    pub fn new() -> Self {
        Default::default()
    }
}

