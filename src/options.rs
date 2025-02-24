pub struct CsvOptions {
    pub(crate) input_file: Option<String>,
    pub(crate) output_file: Option<String>,
    pub(crate) delimiter: Option<char>,
    pub(crate) output_headers: Option<bool>,
    pub(crate) input_columns: Option<Vec<String>>,
    pub(crate) quote_char: Option<char>,
    pub(crate) escape_char: Option<char>,
    pub(crate) trim_fields: Option<bool>,
    pub(crate) flexible: Option<bool>,
    pub(crate) comment_char: Option<char>,
}

impl <'a> Default for CsvOptions {
    fn default() -> Self {
        CsvOptions {
            input_file: None,
            output_file: None,
            input_columns: None,
            comment_char: None,
            delimiter: None,
            escape_char: None,
            flexible: None,
            output_headers: None,
            quote_char: None,
            trim_fields: None,
        }
    }
}