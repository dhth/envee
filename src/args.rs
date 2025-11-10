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
        /// Only validate versions file
        #[arg(long = "validate-only")]
        only_validate_versions: bool,
        /// Show commits between tags corresponding to different environments (requires ENVEE_GH_TOKEN to be set)
        #[arg(long = "no-commit-logs", short = 'C')]
        no_commit_logs: bool,
        /// Output format
        #[arg(long = "output-format", short = 'o', default_value_t = OutputFormat::Stdout, value_name = "FORMAT")]
        output_format: OutputFormat,
        /// Regex to use for filtering apps
        #[arg(long = "filter", short = 'f', value_name = "REGEX")]
        app_filter: Option<String>,
        /// Table style for stdout output
        #[arg(long = "stdout-table-style", default_value_t = TableStyle::Utf8, value_name="STRING")]
        stdout_table_style: TableStyle,
        /// Whether to use output text to stdout without color
        #[arg(long = "stdout-plain")]
        stdout_plain_output: bool,
        /// Path for the HTML output file
        #[arg(
            long = "html-output",
            value_name = "PATH",
            default_value = "envee-report.html"
        )]
        html_output_path: PathBuf,
        /// Title for HTML report
        #[arg(long = "html-title", value_name = "STRING", default_value = "envee")]
        html_title: String,
        /// Path to custom HTML template file
        #[arg(long = "html-template", value_name = "PATH")]
        html_template_path: Option<PathBuf>,
    },
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match &self.command {
            EnveeCommand::Run {
                versions_file_path,
                only_validate_versions,
                no_commit_logs,
                output_format,
                app_filter,
                stdout_table_style,
                stdout_plain_output,
                html_output_path,
                html_title,
                html_template_path,
            } => {
                let flags_based_on_output = match output_format {
                    OutputFormat::Stdout => format!(
                        r#"
table style:                          {}
plain output:                         {}
"#,
                        stdout_table_style, stdout_plain_output
                    ),
                    OutputFormat::Html => {
                        format!(
                            r#"
output path:                          {}
title:                                {}
template path:                        {}
"#,
                            html_output_path.to_string_lossy(),
                            html_title,
                            html_template_path
                                .as_ref()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or(NOT_PROVIDED.to_string())
                        )
                    }
                };

                format!(
                    r#"
command:                              Run
versions file:                        {}
only validate versions file:          {}
don't show commit logs:               {}
output format:                        {}
app filter:                           {}{}
"#,
                    versions_file_path.to_string_lossy(),
                    only_validate_versions,
                    no_commit_logs,
                    match output_format {
                        OutputFormat::Stdout => "stdout",
                        OutputFormat::Html => "html",
                    },
                    app_filter.as_deref().unwrap_or(NOT_PROVIDED),
                    flags_based_on_output
                )
            }
        };

        f.write_str(&output)
    }
}
