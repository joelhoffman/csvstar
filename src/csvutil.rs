use std::io::BufRead;
use csv::{Reader, ReaderBuilder, StringRecord, Trim};
use std::ops::RangeInclusive;
use std::error::Error;
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

    if options.trim_fields.unwrap_or(false) {
        reader_builder.trim(Trim::All);
    }

    if let Some(c) = options.quote_char {
        reader_builder.delimiter(c as u8);
    }

    reader_builder.from_reader(input)
}

pub fn parse_range(s: &str) -> Result<RangeInclusive<usize>, ()> {
    let (min, max) = s.split_once('-').ok_or(())?;
    Ok(RangeInclusive::new(
            min.parse::<usize>().map_err(|_| ())?,
            max.parse::<usize>().map_err(|_| ())?))
}

pub fn validate_range(range: RangeInclusive<usize>, first_row: &StringRecord) -> Result<Vec<usize>, Box<dyn Error>> {
    if range.start() >= range.end() {
        return Err(Box::from(format!("Invalid range. Must be increasing: {}-{}", range.start(), range.end())));
    }
    if *range.end() > first_row.len() {
        return Err(Box::from(format!("Invalid range. There are only {} columns: {}-{}", first_row.len(), range.start(), range.end())));
    }
    Ok(range.clone().map(|i| i - 1).collect::<Vec<_>>())
}

pub fn add_numeric_col(first_row: &StringRecord, n_headers: i32, numeric: i32) -> Result<usize, Box<dyn Error>> {
    if numeric == 0 {
        Err(Box::from("Column 0 is invalid. Columns are 1-based."))
    } else if (numeric < -n_headers) || (numeric > n_headers) {
        Err(Box::from(format!("Column {} is invalid. There are {} columns.", numeric, first_row.len())))
    } else if numeric > 0 {
        Ok((numeric - 1) as usize)
    } else {
        Ok((n_headers + numeric) as usize)
    }
}

pub fn select_column_indices(first_row: &StringRecord, columns: &Option<Vec<String>>) -> Result<Vec<usize>, Box<dyn Error>> {
    Ok(match columns {
        Some(cols) => {
            let mut idx_vec = vec![];
            let n_headers = first_row.len() as i32;
            for col in cols {
                if let Ok(numeric) = col.parse::<i32>() {
                    idx_vec.push(add_numeric_col(&first_row, n_headers, numeric)?);
                } else if let Ok(range) = parse_range(col) {
                    idx_vec.extend(validate_range(range, &first_row)?);
                } else {
                    idx_vec.push(first_row
                        .iter()
                        .position(|h| h == col)
                        .ok_or_else(|| format!("Column '{}' not found in input file", col))?)
                }
            }
            idx_vec
        },
        None => (0..first_row.len()).collect(), // If no columns are specified, include all columns
    })
}

pub fn enumerate_output_headers(input_has_headers: bool, first_row: StringRecord, selected_indices: &Vec<usize>) -> Vec<String> {
    let mut out_headers = vec![];
    if input_has_headers {
        out_headers.extend(selected_indices.iter().map(|&i| first_row[i].to_string()));
    } else {
        let alphabet = ('a'..='z').map(String::from).collect::<Vec<_>>();
        out_headers.extend(selected_indices.iter()
            .map(|&i| alphabet[i % 26].repeat(1 + i / 26)));
    }
    out_headers
}