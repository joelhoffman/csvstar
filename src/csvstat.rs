use std::fmt::write;
use crate::args::global_args;
use crate::options::CsvOptions;
use clap::Arg;
use clap::ArgAction::SetTrue;
use multiset::HashMultiSet;
use priority_queue::DoublePriorityQueue;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};

pub mod csvutil;
pub mod args;
pub mod options;

struct CsvStatOptions { input_columns: Option<Vec<String>>, csv: bool }

struct CsvColumnStat {
    idx: usize,
    name: String,
    n: u64,
    n_numeric: u64,
    sum: f64,
    mean: f64,
    v_k: f64,
    variance: f64,
    n_zeros: u64,
    min: f64,
    max: f64,
    min_str: String,
    max_str: String,
    n_missing: u64,
    n_empty: u64,
    distinct: HashMultiSet<String>,
    max_len: usize
}

impl CsvColumnStat {
    pub(crate) fn freq(&self) -> Vec<String> {
        let mut v: Vec<String> = vec![];
        let mut p: DoublePriorityQueue<&String, usize> = DoublePriorityQueue::new();
        self.distinct.iter().for_each(|d| {
            p.push(d, self.distinct.count_of(d));
            while p.len() > 100 {
                p.pop_min();
            }
        });

        while !p.is_empty() {
            let (d, c) = p.pop_min().unwrap();
            v.push(d.clone() + " (" + &c.to_string() + "X)");
        }

        v
    }
}

impl CsvColumnStat {
    pub(crate) fn stdev(&self) -> f64 {
        if self.n_numeric < 2 {
            return 0.0;
        }
        return (self.v_k / (self.n_numeric as f64 - 1.0)).sqrt();
    }

    pub(crate) fn median(&self) -> f64 {
        if self.n_numeric < 2 {
            return 0.0;
        }
        return 0.0;
    }

    pub(crate) fn mean(&self) -> f64 {
        if self.sum == 0.0 {
            return 0.0;
        }

        self.n_numeric as f64 / self.sum
    }

    pub(crate) fn unique(&self) -> usize {
        self.distinct.len()
    }

    pub(crate) fn nulls(&self) -> bool {
        self.n_missing > 0
    }
    pub(crate) fn name(&self) -> &String {
        &self.name
    }

    pub fn infer_type(&self) -> String {
        if self.is_numeric() {
            "numeric".to_string()
        } else {
            "string".to_string()
        }
    }

    fn is_numeric(&self) -> bool {
        (self.n_numeric == self.n - self.n_missing) && (self.n_numeric > 0)
    }

    pub fn max(&self) -> String {
        if self.is_numeric() {
            String::from(self.max.to_string())
        } else {
            self.max_str.clone()
        }
    }

    pub fn min(&self) -> String {
        if self.is_numeric() {
            String::from(self.min.to_string())
        } else {
            self.min_str.clone()
        }
    }
}
fn main() -> Result<(), String> {
    let (options, stat_options) = parse_args(std::env::args().collect::<Vec<_>>());

    match process_csv(&options, &stat_options) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn process_csv(options: &CsvOptions, stat_options: &CsvStatOptions) -> Result<(), Box<dyn std::error::Error>> {
    let input:Box<dyn BufRead> = options.get_input_file()?;

    let mut reader = csvutil::csv_reader(options, input);

    // Get the column headers
    let first_row = reader.headers()?.clone();

    // Determine which columns to include
    let selected_indices: Vec<usize> = csvutil::select_column_indices(&first_row, &stat_options.input_columns)?;

    let mut csv_file_handle:Box<dyn io::Write>;
    if let Some(file) = &options.output_file {
        csv_file_handle = Box::new(File::create(file)?);
    } else {
        csv_file_handle = Box::new(io::stdout());
    }

    let output_has_headers = options.output_headers
        .unwrap_or(true);

    let out_headers = csvutil::enumerate_output_headers(options.input_has_headers.unwrap_or(true), first_row, &selected_indices);

    let mut statistics: Vec<CsvColumnStat> = vec![];
    selected_indices.iter().for_each(|&i| statistics.push(CsvColumnStat {
        idx: i,
        name: out_headers[i].clone(),
        n: 0,
        n_numeric: 0,
        sum: 0.0,
        mean: 0.0,
        v_k: 0.0,
        variance: 0.0,
        min: 0.0,
        max: 0.0,
        min_str: "".to_string(),
        max_str: "".to_string(),
        max_len: 0,
        n_zeros: 0,
        n_missing: 0,
        n_empty: 0,
        distinct: HashMultiSet::new()
    }));

    for result in reader.records() {
        let record = result?;
        selected_indices.iter()
            .for_each(|&i| add_statistic(record.get(i), &mut statistics[i]));
    }

    if stat_options.csv {
        let out_headers = vec!["column_id","column_name","type","nulls","unique","min","max","sum","mean","median","stdev","len","freq"];
        if output_has_headers {
            csv_file_handle.write(format_args!("{}\n", out_headers.join(",")).to_string().as_bytes())?;
        }
        for statistic in statistics {
            if statistic.is_numeric() {
                csv_file_handle.write(format_args!("{},{},Number,{},{},{},{},{},{},{},{},,{}\n",
                       statistic.idx,
                       statistic.name,
                       statistic.nulls(),
                       statistic.unique(),
                       statistic.min,
                       statistic.max,
                       statistic.sum,
                       statistic.mean(),
                       statistic.median(),
                       statistic.stdev(),
                       statistic.freq().join(",")).to_string().as_bytes())?;

            } else {
                csv_file_handle.write(format_args!("{},{},Text,{},{},{},{},,,,,{},\"{}\"\n",
                                                   statistic.idx,
                                                   statistic.name,
                                                   statistic.nulls(),
                                                   statistic.unique(),
                                                   statistic.min_str,
                                                   statistic.max_str,
                                                   statistic.max_len,
                                                   statistic.freq().join(",")).to_string().as_bytes())?;
            }
        }
    } else {

    }

    Ok(())
}

fn add_statistic(value: Option<&str>, p1: &mut CsvColumnStat) -> () {
    p1.n += 1;

    if value.is_none() {
        p1.n_missing += 1;
        return;
    }
    let string = value.unwrap().to_string();

    if p1.n - (p1.n_missing+p1.n_empty) == 1 || string > p1.max_str {
        p1.max_str = string.clone();
    }
    if p1.n - (p1.n_missing+p1.n_empty) == 1 || string < p1.min_str {
        p1.min_str = string.clone();
    }
    if string.is_empty() {
        p1.n_empty += 1;
    }
    p1.distinct.insert(string.clone());
    if let Ok(float) = string.parse::<f64>() {
        p1.n_numeric += 1;
        p1.sum += float;
        let prev_mean = p1.mean;
        // This method for computing the stream mean and variance is apparently from Knuth
        // and I found it at https://math.stackexchange.com/questions/20593/calculate-variance-from-a-stream-of-sample-values
        // n.b. if this is the first numeric value, then m_1 will be x_1 here as long as n_numeric has been previously incremented.
        p1.mean = p1.mean + (float - p1.mean) / p1.n_numeric as f64;
        p1.variance = p1.variance + (float - p1.mean) * (float - prev_mean);
        if p1.n_numeric == 1 || float > p1.max {
            p1.max = float;
        }
        if p1.n_numeric == 1 || float < p1.min {
            p1.min = float;
        }
    }
}

fn parse_args(args: Vec<String>) -> (CsvOptions, CsvStatOptions) {
    let executable_name = args[0].clone();

    let command = global_args()
        .display_name(executable_name)
        .about("Computes statistics from CSV files.")
        .arg(Arg::new("csv")
            .long("csv")
            .action(SetTrue)
            .help("Output statistics in CSV format"))
        .arg(Arg::new("input_columns")
            .short('c')
            .long("columns")
            .allow_negative_numbers(true)
            .help("List of column names, offsets or ranges to include, e.g. \"1,id,-2,3-5. Negative offsets are interpreted as relative to the end (-1 is the last column). Ranges are inclusive.")
            .action(clap::ArgAction::Append));

    let mut matches = command.get_matches_from(args);

    let action = CsvStatOptions {
        input_columns: matches.remove_many::<String>("input_columns")
            .map(|v| v.flat_map(|s| s.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
                .collect::<Vec<_>>()),
        csv: matches.remove_one("csv").unwrap_or(false),
    };

    (args::build_options(matches), action)
}
