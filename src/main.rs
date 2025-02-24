mod options;

use clap::{Arg, ArgMatches, Command};
use options::CsvOptions;

fn main() {
    let mut matches = build_args();
    let mut options = CsvOptions {
        ..Default::default()
    };
    // Handle the arguments
    options.output_file = matches.remove_one::<String>("output");
    // options.output_headers = matches.get_one::<bool>("output_headers");
    let option   = matches.remove_many::<String>("input_columns")
        .map(|v| v.flat_map(|s| s.split(',').map(|s| s.trim().to_string())
            .collect::<Vec<_>>())
            .collect::<Vec<_>>());
    options.input_columns = option;
    options.input_file = matches.remove_one::<String>("input");
    println!("{:?}", options.input_columns);
}

fn build_args<'a>() -> ArgMatches {
    let args = std::env::args().collect::<Vec<_>>();
    let string = args[0].clone();
    Command::new("CsvStar")
        .display_name(string)
        .version("1.0")
        .author("Your Name")
        .about("A description of your program")
        .arg(Arg::new("input")
                .short('i')
                .long("input")
                .help("Input file to process")
                .required(false))
        .arg(Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file")
                .required(false))
        .arg(Arg::new("output_headers")
                .long("output_headers")
                .help("Include output headers")
                .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("input_columns")
                .short('c')
                .long("input_columns")
                .help("List of column names or offsets to include")
                .action(clap::ArgAction::Append))
        .arg(Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Print verbose output")
                .action(clap::ArgAction::SetTrue))
        .get_matches()
}
