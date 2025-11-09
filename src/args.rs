use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::TableStyle;

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
        #[arg(long = "table-style", short='s', default_value_t = TableStyle::Utf8)]
        table_style: TableStyle,
        /// Whether to use output text to stdout without color
        #[arg(long = "plain", short = 'p')]
        plain_output: bool,
        /// Only validate versions file
        #[arg(long = "validate-only")]
        only_validate_versions: bool,
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
                only_validate_versions: validate_only,
            } => format!(
                r#"
command:                              Run
versions file:                        {}
don't show commit logs:               {}
table style:                          {}
plain output:                         {}
only validate versions file:          {}
"#,
                versions_file_path.to_string_lossy(),
                no_commit_logs,
                table_style,
                plain_output,
                validate_only,
            ),
        };

        f.write_str(&output)
    }
}
