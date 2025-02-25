use crate::options::CsvOptions;
use clap::{Arg, ArgMatches, Command};

pub fn global_args() -> Command {
    Command::new("CsvStar")
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
        .arg(Arg::new("trim_fields").short('m').long("trimfields").help("Trim fields and headers").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("delimiter").short('d').long("delimiter").help("Delimiter character"))
        .arg(Arg::new("quote_char").short('q').long("quotechar").help("Quote character"))
        .arg(Arg::new("escape_char").short('p').long("escapechar").help("Escape character"))
        .arg(Arg::new("comment_char").short('n').long("commentchar").help("Comment character"))
}

pub fn build_options(mut arg_matches: ArgMatches) -> CsvOptions {
    let mut options = CsvOptions::new();
    options.input_file = arg_matches.remove_one("input").filter(|f| f != "-");
    options.output_file = arg_matches.remove_one("output").filter(|f| f != "-");

    options.output_headers = arg_matches.remove_one::<bool>("no_output_headers").map(|v| !v);
    options.input_has_headers = arg_matches.remove_one::<bool>("input_has_no_headers").map(|v| !v);
    options.flexible = arg_matches.remove_one("flexible");

    options.delimiter = arg_matches.remove_one::<String>("delimiter")
        .map(|s| s.chars().next().unwrap());
    options.quote_char = arg_matches.remove_one::<String>("quote_char")
        .map(|s| s.chars().next().unwrap());
    options.escape_char = arg_matches.remove_one::<String>("escape_char")
        .map(|s| s.chars().next().unwrap());
    options.comment_char = arg_matches.remove_one::<String>("comment_char")
        .map(|s| s.chars().next().unwrap());
    options.trim_fields = arg_matches.remove_one("trim_fields");

    options
}


#[cfg(test)]
mod tests {
    use crate::args::{build_options, global_args};

    #[test]
    fn test_build_args() {
        let args = vec![
            "CsvStar",
            "test.csv",
            "--output", "output.csv",
            "--delimiter", ";",
            "--quotechar", "'",
            "--escapechar", "@",
            "--commentchar", "$",
            "--trimfields",
            "--no-header-row",
            "--no-output-headers",
        ].iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let matches = global_args().get_matches_from(args);
        let options = build_options(matches);

        assert_eq!(options.input_file.unwrap(), "test.csv");
        assert_eq!(options.output_file.unwrap(), "output.csv");

        assert_eq!(options.output_headers.unwrap(), false);
        assert_eq!(options.input_has_headers.unwrap(), false);
        assert_eq!(options.delimiter.unwrap(), ';');
        assert_eq!(options.quote_char.unwrap(), '\'');
        assert_eq!(options.escape_char.unwrap(), '@');
        assert_eq!(options.comment_char.unwrap(), '$');
        assert_eq!(options.trim_fields.unwrap(), true);
    }
}