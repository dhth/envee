#[allow(unused)]
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum TableStyle {
    Ascii,
    Markdown,
    None,
    Utf8,
}

impl std::fmt::Display for TableStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableStyle::Ascii => write!(f, "ascii"),
            TableStyle::Markdown => write!(f, "markdown"),
            TableStyle::None => write!(f, "none"),
            TableStyle::Utf8 => write!(f, "utf8"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StdoutConfig {
    pub table_style: TableStyle,
    pub plain_output: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputType {
    Stdout(StdoutConfig),
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub output_type: OutputType,
}
