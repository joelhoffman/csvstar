#[derive(Default, Clone)]
pub struct CsvOptions {
    pub(crate) input_file: Option<String>,
    pub(crate) output_file: Option<String>,
    pub(crate) delimiter: Option<char>,
    pub(crate) input_has_headers: Option<bool>,
    pub(crate) output_headers: Option<bool>,
    pub(crate) input_columns: Option<Vec<String>>,
    pub(crate) quote_char: Option<char>,
    pub(crate) escape_char: Option<char>,
    pub(crate) trim_fields: Option<bool>,
    pub(crate) flexible: Option<bool>,
    pub(crate) comment_char: Option<char>,
}

impl CsvOptions {
    pub fn new() -> Self {
        Default::default()
    }
}