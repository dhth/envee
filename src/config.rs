use clap::ValueEnum;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Stdout,
    Html,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Stdout => write!(f, "stdout"),
            OutputFormat::Html => write!(f, "html"),
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct HtmlConfig {
    pub output_path: PathBuf,
    pub title: String,
    pub template: String,
}

#[derive(Debug, Clone)]
pub enum OutputType {
    Stdout(StdoutConfig),
    Html(HtmlConfig),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub output_type: OutputType,
}
