use std::io::BufRead;
use csv::{Reader, ReaderBuilder, Trim};
use crate::options::CsvOptions;

pub fn csv_reader(options: &CsvOptions, input: Box<dyn BufRead>) -> Reader<Box<dyn BufRead>> {
    let mut reader_builder = ReaderBuilder::new();

    reader_builder.has_headers(options.input_has_headers.unwrap_or(true))
        .comment(options.comment_char.map(|c| c as u8))
        .escape(options.escape_char.map(|c| c as u8))
        .flexible(options.flexible.unwrap_or(true));

    if let Some(c) = options.delimiter {
        reader_builder.delimiter(c as u8);
    }

    if options.trim_fields.is_some() {
        reader_builder.trim(Trim::All);
    }

    if let Some(c) = options.quote_char {
        reader_builder.delimiter(c as u8);
    }

    reader_builder.from_reader(input)
}