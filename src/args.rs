use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::TableStyle;

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
        /// Output table style
        #[arg(long = "table-style", short='s', default_value_t = TableStyle::Utf8, value_name="STRING")]
        table_style: TableStyle,
        /// Whether to use output text to stdout without color
        #[arg(long = "plain", short = 'p')]
        plain_output: bool,
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
                table_style,
                plain_output,
                only_validate_versions,
                app_filter,
            } => format!(
                r#"
command:                              Run
versions file:                        {}
don't show commit logs:               {}
table style:                          {}
plain output:                         {}
only validate versions file:          {}
app filter:                           {}
"#,
                versions_file_path.to_string_lossy(),
                no_commit_logs,
                table_style,
                plain_output,
                only_validate_versions,
                app_filter.as_deref().unwrap_or(NOT_PROVIDED),
            ),
        };

        f.write_str(&output)
    }
}
