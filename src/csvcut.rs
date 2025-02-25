mod options;

use clap::{Arg, Command};
use csv::{ReaderBuilder, Trim, WriterBuilder};
use options::CsvOptions;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::ops::Range;

fn main() -> Result<(), String> {
    let options = parse_args(std::env::args().collect::<Vec<_>>());

    match process_csv(&options) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn parse_args(args: Vec<String>) -> CsvOptions {
    let executable_name = args[0].clone();
    let mut matches = Command::new("CsvStar")
        .display_name(executable_name)
        .version("1.0")
        .about("A description of your program")
        .arg(Arg::new("input")
                .help("Input file to process")
                .required(false))
        .arg(Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file")
                .required(false))
        .arg(Arg::new("no_output_headers")
                .long("no-output-headers")
                .help("Exclude output headers (will default to input file)")
                .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("input_has_no_headers")
                .short('H')
                .long("no-header-row")
                .help("Input file has no headers")
                .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("flexible")
                .short('f')
                .long("flexible")
                .help("Allow variable number of fields per record")
                .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("input_columns")
                .short('c')
                .long("columns")
                .allow_negative_numbers(true)
                .help("List of column names, offsets or ranges to include, e.g. \"1,id,-2,3-5. Negative offsets are interpreted as relative to the end (-1 is the last column).")
                .action(clap::ArgAction::Append))
        .arg(Arg::new("trim_fields").short('m').long("trimfields").help("Trim fields and headers").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("delimiter").short('d').long("delimiter").help("Delimiter character"))
        .arg(Arg::new("quote_char").short('q').long("quotechar").help("Quote character"))
        .arg(Arg::new("escape_char").short('p').long("escapechar").help("Escape character"))
        .arg(Arg::new("comment_char").short('n').long("commentchar").help("Comment character"))
        .get_matches_from(args);

    let mut options = CsvOptions::new();

    options.input_file = matches.remove_one("input").filter(|f| f != "-");
    options.output_file = matches.remove_one("output").filter(|f| f != "-");

    options.output_headers = matches.remove_one::<bool>("no_output_headers").map(|v| !v);
    options.input_has_headers = matches.remove_one::<bool>("input_has_no_headers").map(|v| !v);
    options.flexible = matches.remove_one("flexible");
    options.input_columns = matches.remove_many::<String>("input_columns")
        .map(|v| v.flat_map(|s| s.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
            .collect::<Vec<_>>());

    options.delimiter = matches.remove_one::<String>("delimiter")
        .map(|s| s.chars().next().unwrap());
    options.quote_char = matches.remove_one::<String>("quote_char")
        .map(|s| s.chars().next().unwrap());
    options.escape_char = matches.remove_one::<String>("escape_char")
        .map(|s| s.chars().next().unwrap());
    options.comment_char = matches.remove_one::<String>("comment_char")
        .map(|s| s.chars().next().unwrap());
    options.trim_fields = matches.remove_one("trim_fields");
    options
}

fn parse_range(s: &str) -> Result<Range<usize>, ()> {
    let (min, max) = s.split_once('-').ok_or(())?;
    Ok(
        Range {
            start: min.parse::<usize>().map_err(|_| ())?,
            end: max.parse::<usize>().map_err(|_| ())?,
        }
    )
}

fn process_csv(options: &CsvOptions) -> Result<(), Box<dyn Error>> {
    let input:Box<dyn BufRead>;
    if let Some(file) = &options.input_file {
        input = Box::new(BufReader::new(File::open(file)?));
    } else {
        input = Box::new(BufReader::new(stdin()));
    }

    // Open the CSV reader
    let input_has_headers = options.input_has_headers.unwrap_or(true);

    let mut reader_builder = ReaderBuilder::new();

    reader_builder.has_headers(input_has_headers)
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

    let mut reader = reader_builder.from_reader(input);

    // Get the column headers
    let headers = reader.headers()?.clone();

    // Determine which columns to include
    let selected_indices: Vec<usize> = match &options.input_columns {
        Some(cols) => {
            let mut idx_vec = vec![];
            let n_headers = headers.len() as i32;
            for col in cols {
                if let Ok(numeric) = col.parse::<i32>() {
                    if numeric == 0 {
                        return Err(Box::from("Column 0 is invalid. Columns are 1-based."));
                    } else if (numeric < -n_headers) || (numeric > n_headers) {
                        return Err(Box::from(format!("Column {} is invalid. There are {} columns.", numeric, headers.len())));
                    } else if numeric > 0 {
                        idx_vec.push((numeric -1) as usize);
                    } else {
                        idx_vec.push((n_headers + numeric) as usize);
                    }
                } else if let Ok(range) = parse_range(col) {
                    if range.start >= range.end {
                        return Err(Box::from(format!("Invalid range. Must be increasing: {}-{}", range.start, range.end)));
                    }
                    if range.end > headers.len() {
                        return Err(Box::from(format!("Invalid range. There are only {} columns: {}-{}", headers.len(), range.start, range.end)));
                    }
                    idx_vec.extend(range.clone().map(|i| i - 1));
                } else {
                    idx_vec.push(headers
                        .iter()
                        .position(|h| h == col)
                        .ok_or_else(|| format!("Column '{}' not found in input file", col))?)
                }
            }
            idx_vec
        },
        None => (0..headers.len()).collect(), // If no columns are specified, include all columns
    };

    let csv_file_handle:Box<dyn io::Write>;
    if let Some(file) = &options.output_file {
        csv_file_handle = Box::new(BufWriter::new(File::create(file)?));
    } else {
        csv_file_handle = Box::new(io::stdout());
    }

    let output_has_headers = options.output_headers.unwrap_or(input_has_headers);
    let mut csv_writer = WriterBuilder::new().has_headers(output_has_headers)
        .from_writer(csv_file_handle);

    if output_has_headers {
        csv_writer.write_record(selected_indices.iter().map(|&i| &headers[i]))?;
    }

    for result in reader.records() {
        let record = result?;
        let selected_values: Vec<&str> = selected_indices.iter().map(|&i| &record[i]).collect();
        csv_writer.write_record(selected_values)?;
    }

    csv_writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_process_csv_with_valid_input() {
        let input_file = "test/test_input.csv";
        let output_file = "test_output.csv";

        let options = CsvOptions {
            input_file: Some(input_file.to_string()),
            output_file: Some(output_file.to_string()),
            input_columns: Some(vec!["col1".to_string(), "col3".to_string()]),
            ..Default::default()
        };

        process_csv(&options).expect("process_csv failed");

        let expected_output = "col1,col3\n1,3\n4,6\n7,9\n";
        let actual_output = fs::read_to_string(output_file).expect("Unable to read output file");
        assert_eq!(actual_output, expected_output);

        fs::remove_file(output_file).expect("Unable to delete test output file");
    }

    #[test]
    fn test_process_csv_with_invalid_column() {
        let input_file = "test/test_input.csv";
        let output_file = "test_output.csv";

        let mut options = CsvOptions {
            input_file: Some(input_file.to_string()),
            output_file: Some(output_file.to_string()),
            input_columns: Some(vec!["1-4".to_string()]),
            ..Default::default()
        };

        assert_eq!(process_csv(&options).expect_err("").to_string(),
                   "Invalid range. There are only 3 columns: 1-4");

        options.input_columns = Some(vec!["4-1".to_string()]);
        assert_eq!(process_csv(&options).expect_err("").to_string(),
                   "Invalid range. Must be increasing: 4-1");

        options.input_columns = Some(vec!["1-1".to_string()]);
        assert_eq!(process_csv(&options).expect_err("").to_string(),
                   "Invalid range. Must be increasing: 1-1");
    }

    #[test]
    fn test_process_csv_without_columns_specified() {
        let input_file = "test/test_input.csv";
        let output_file = "test_output_no_columns.csv";
        let input_data = fs::read_to_string(input_file).expect("Unable to read test input file");

        let options = CsvOptions {
            input_file: Some(input_file.to_string()),
            output_file: Some(output_file.to_string()),
            input_columns: None,
            output_headers: Some(true),
            ..Default::default()
        };

        process_csv(&options).expect("process_csv failed");

        let expected_output = input_data; // Since no columns are filtered, all columns are written
        let actual_output = fs::read_to_string(output_file).expect("Unable to read output file");
        assert_eq!(actual_output, expected_output);

        fs::remove_file(output_file).expect("Unable to delete test output file");
    }

    #[test]
    fn test_build_args() {
        let args = vec![
            "CsvStar",
            "test.csv",
            "--output", "output.csv",
            "--columns", "col1,col2",
            "--delimiter", ";",
            "--quotechar", "'",
            "--escapechar", "@",
            "--commentchar", "$",
            "--trimfields",
            "--no-header-row",
            "--no-output-headers",
        ].iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let options = parse_args(args);

        assert_eq!(options.input_file.unwrap(), "test.csv");
        assert_eq!(options.output_file.unwrap(), "output.csv");
        let columns: Vec<String> = options.input_columns.unwrap();
        assert_eq!(columns, vec!["col1", "col2"]);

        assert_eq!(options.output_headers.unwrap(), false);
        assert_eq!(options.input_has_headers.unwrap(), false);
        assert_eq!(options.delimiter.unwrap(), ';');
        assert_eq!(options.quote_char.unwrap(), '\'');
        assert_eq!(options.escape_char.unwrap(), '@');
        assert_eq!(options.comment_char.unwrap(), '$');
        assert_eq!(options.trim_fields.unwrap(), true);
    }
}

