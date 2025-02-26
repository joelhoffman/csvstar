pub mod options;
pub mod args;
mod csvutil;

use clap::Arg;
use csv::{StringRecord, WriterBuilder};
use options::CsvOptions;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead};
use std::iter::{Iterator};
use std::string::ToString;
use serial_test::serial;
use crate::args::global_args;

struct CsvCutOptions { input_columns: Option<Vec<String>> }

fn main() -> Result<(), String> {
    let (options, action) = parse_args(std::env::args().collect::<Vec<_>>());

    match process_csv(&options, &action) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn parse_args(args: Vec<String>) -> (CsvOptions, CsvCutOptions) {
    let executable_name = args[0].clone();

    let command = global_args()
        .display_name(executable_name)
        .about("Selects columns from CSV files.")
        .arg(Arg::new("input_columns")
            .short('c')
            .long("columns")
            .allow_negative_numbers(true)
            .help("List of column names, offsets or ranges to include, e.g. \"1,id,-2,3-5. Negative offsets are interpreted as relative to the end (-1 is the last column). Ranges are inclusive.")
            .action(clap::ArgAction::Append));

    let mut matches = command.get_matches_from(args);

    let action = CsvCutOptions {
        input_columns: matches.remove_many::<String>("input_columns")
            .map(|v| v.flat_map(|s| s.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
                .collect::<Vec<_>>())
    };

    (args::build_options(matches), action)
}

fn process_csv(options: &CsvOptions, cut_options: &CsvCutOptions) -> Result<(), Box<dyn Error>> {
    let input:Box<dyn BufRead> = options.get_input_file()?;

    let mut reader = csvutil::csv_reader(options, input);

    // Get the column headers
    let first_row = reader.headers()?.clone();

    // Determine which columns to include
    let selected_indices: Vec<usize> = csvutil::select_column_indices(&first_row, &cut_options.input_columns)?;

    let csv_file_handle = options.get_output_file()?;

    let output_has_headers = options.output_headers
        .or(options.input_has_headers)
        .unwrap_or(true);

    let mut csv_writer = WriterBuilder::new().has_headers(output_has_headers)
        .from_writer(csv_file_handle);

    if output_has_headers {
        let out_headers = csvutil::enumerate_output_headers(options.input_has_headers.unwrap_or(true), first_row, &selected_indices);
        csv_writer.write_record(out_headers)?;
    }

    let mut record = StringRecord::new();
    while !reader.is_done() {
        reader.read_record(&mut record)?;
        if !reader.is_done() {
            let selected_values = selected_indices.iter().flat_map(|&i| record.get(i));
            csv_writer.write_record(selected_values)?;
        }
    }

    csv_writer.flush()?;

    Ok(())
}

#[cfg(test)]
#[serial] // tests must be serial because they write files with the same name
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_process_csv_with_valid_input() {
        let input_file = "test/test_input.csv";
        let output_file = "test_output.csv";

        let action = CsvCutOptions {
            input_columns: Some(vec!["col1".to_string(), "col3".to_string()]),
        };

        let options = CsvOptions {
            input_file: Some(input_file.to_string()),
            output_file: Some(output_file.to_string()),
            ..Default::default()
        };

        process_csv(&options, &action).expect("process_csv failed");

        let expected_output = "col1,col3\n1,3\n4,6\n7,9\n";
        let actual_output = fs::read_to_string(output_file).expect("Unable to read output file");
        assert_eq!(actual_output, expected_output);

        fs::remove_file(output_file).expect("Unable to delete test output file");
    }

    #[test]
    fn test_process_csv_with_valid_input_no_headers() {
        let input_file = "test/test_input_no_headers.csv";
        let output_file = "test_output.csv";

        let action = CsvCutOptions {
            input_columns: Some(vec!["1".to_string(), "-1".to_string()]),
        };

        let options = CsvOptions {
            input_file: Some(input_file.to_string()),
            output_file: Some(output_file.to_string()),
            input_has_headers: Some(false),
            ..Default::default()
        };

        process_csv(&options, &action).expect("process_csv failed");

        let expected_output = "1,3\n4,6\n7,9\n";
        let actual_output = fs::read_to_string(output_file).expect("Unable to read output file");
        assert_eq!(actual_output, expected_output);

        fs::remove_file(output_file).expect("Unable to delete test output file");
    }

    #[test]
    fn test_process_csv_with_valid_input_no_headers_range() {
        let input_file = "test/test_input_no_headers.csv";
        let output_file = "test_output.csv";

        let action = CsvCutOptions {
            input_columns: Some(vec!["1-2".to_string()]),
        };

        let options = CsvOptions {
            input_file: Some(input_file.to_string()),
            output_file: Some(output_file.to_string()),
            input_has_headers: Some(false),
            ..Default::default()
        };

        process_csv(&options, &action).expect("process_csv failed");

        let expected_output = "1,2\n4,5\n7,8\n";
        let actual_output = fs::read_to_string(output_file).expect("Unable to read output file");
        assert_eq!(actual_output, expected_output);

        fs::remove_file(output_file).expect("Unable to delete test output file");
    }

    #[test]
    fn test_process_csv_with_invalid_column() {
        let input_file = "test/test_input.csv";
        let output_file = "test_output.csv";

        let action = CsvCutOptions {
            input_columns: Some(vec!["1-4".to_string()]),
        };

        let options = CsvOptions {
            input_file: Some(input_file.to_string()),
            output_file: Some(output_file.to_string()),
            ..Default::default()
        };

        assert_eq!(process_csv(&options, &action).expect_err("").to_string(),
                   "Invalid range. There are only 3 columns: 1-4");

        let action = CsvCutOptions {
            input_columns: Some(vec!["4-1".to_string()]),
        };
        assert_eq!(process_csv(&options, &action).expect_err("").to_string(),
                   "Invalid range. Must be increasing: 4-1");

        let action = CsvCutOptions {
            input_columns: Some(vec!["1-1".to_string()]),
        };
        assert_eq!(process_csv(&options, &action).expect_err("").to_string(),
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
            output_headers: Some(true),
            ..Default::default()
        };

        let action = CsvCutOptions {
            input_columns: None,
        };

        process_csv(&options, &action).expect("process_csv failed");

        let expected_output = input_data; // Since no columns are filtered, all columns are written
        let actual_output = fs::read_to_string(output_file).expect("Unable to read output file");
        assert_eq!(actual_output, expected_output);

        fs::remove_file(output_file).expect("Unable to delete test output file");
    }

    #[test]
    fn test_process_csv_inventing_column_headers() {
        let input_file = "test/100_empty_columns.csv";
        let output_file = "test_output.csv";

        let options = CsvOptions {
            input_file: Some(input_file.to_string()),
            output_file: Some(output_file.to_string()),
            input_has_headers: Some(false),
            output_headers: Some(true),
            ..Default::default()
        };

        let action = CsvCutOptions {
            input_columns: None,
        };

        process_csv(&options, &action).expect("process_csv failed");

        let expected_output = "\
a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,aa,bb,cc,dd,ee,ff,gg,hh,ii,jj,kk,ll,mm,nn,oo,pp,qq,rr,ss,tt,uu,vv,ww,xx,yy,zz,aaa,bbb,ccc,ddd,eee,fff,ggg,hhh,iii,jjj,kkk,lll,mmm,nnn,ooo,ppp,qqq,rrr,sss,ttt,uuu,vvv,www,xxx,yyy,zzz,aaaa,bbbb,cccc,dddd,eeee,ffff,gggg,hhhh,iiii,jjjj,kkkk,llll,mmmm,nnnn,oooo,pppp,qqqq,rrrr,ssss,tttt,uuuu,vvvv,wwww
,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
";
        let actual_output = fs::read_to_string(output_file).expect("Unable to read output file");
        assert_eq!(actual_output, expected_output);

        fs::remove_file(output_file).expect("Unable to delete test output file");
    }

    #[test]
    fn test_build_args() {
        let args = vec!["CsvStar", "--columns", "col1,col2"]
            .iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let (_, action) = parse_args(args);

        let columns: Vec<String> = action.input_columns.unwrap();
        assert_eq!(columns, vec!["col1", "col2"]);
    }
}

