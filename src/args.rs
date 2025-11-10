use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::{OutputFormat, TableStyle};

const NOT_PROVIDED: &str = "<NOT PROVIDED>";

/// envee compares application versions across environments and shows the commits between them
#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: EnveeCommand,
    /// Output debug information without doing anything
    #[arg(long = "debug", global = true)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum EnveeCommand {
    /// Show results based on a versions file
    #[command(name = "run")]
    Run {
        /// Path to the versions file
        #[arg(
            long = "versions",
            short = 'V',
            value_name = "PATH",
            default_value = "versions.toml"
        )]
        versions_file_path: PathBuf,
        /// Show commits between tags corresponding to different environments (requires ENVEE_GH_TOKEN to be set)
        #[arg(long = "no-commit-logs", short = 'C')]
        no_commit_logs: bool,
        /// Output format
        #[arg(long = "output-format", short = 'o', default_value_t = OutputFormat::Stdout, value_name = "FORMAT")]
        output_format: OutputFormat,
        /// Table style for stdout output
        #[arg(long = "stdout-table-style", short='s', default_value_t = TableStyle::Utf8, value_name="STRING")]
        stdout_table_style: TableStyle,
        /// Whether to use output text to stdout without color
        #[arg(long = "stdout-plain", short = 'p')]
        stdout_plain_output: bool,
        /// Path for the HTML output file
        #[arg(
            long = "html-output",
            value_name = "PATH",
            default_value = "envee-report.html"
        )]
        html_output_path: PathBuf,
        /// Title for HTML report (for html output)
        #[arg(long = "html-title", value_name = "STRING", default_value = "envee")]
        html_title: String,
        /// Path to custom HTML template file
        #[arg(long = "html-template", value_name = "PATH")]
        html_template_path: Option<PathBuf>,
        /// Only validate versions file
        #[arg(long = "validate-only")]
        only_validate_versions: bool,
        /// Regex to use for filtering apps
        #[arg(long = "filter", short = 'f', value_name = "REGEX")]
        app_filter: Option<String>,
    },
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match &self.command {
            EnveeCommand::Run {
                versions_file_path,
                no_commit_logs,
                output_format,
                stdout_table_style,
                stdout_plain_output,
                html_output_path,
                html_title,
                html_template_path,
                only_validate_versions,
                app_filter,
            } => {
                let output_format_str = match output_format {
                    OutputFormat::Stdout => "stdout",
                    OutputFormat::Html => "html",
                };
                format!(
                    r#"
command:                              Run
versions file:                        {}
don't show commit logs:               {}
output format:                        {}
stdout table style:                   {}
stdout plain output:                  {}
html output path:                     {}
html title:                           {}
html template path:                   {}
only validate versions file:          {}
app filter:                           {}
"#,
                    versions_file_path.to_string_lossy(),
                    no_commit_logs,
                    output_format_str,
                    stdout_table_style,
                    stdout_plain_output,
                    html_output_path.to_string_lossy(),
                    html_title,
                    html_template_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or(NOT_PROVIDED.to_string()),
                    only_validate_versions,
                    app_filter.as_deref().unwrap_or(NOT_PROVIDED),
                )
            }
        };

        f.write_str(&output)
    }
}
