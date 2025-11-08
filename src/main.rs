mod args;
mod config;
mod domain;
mod service;
mod view;

use crate::config::StdoutConfig;
use anyhow::Context;
use args::Args;
use chrono::Utc;
use clap::Parser;
use config::{Config, OutputType, TablePreset};
use domain::Versions;
use std::{env::VarError, path::Path};

const ENV_VAR_GH_TOKEN: &str = "ENVEE_GH_TOKEN";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.debug {
        print!("DEBUG INFO\n{args}");
        return Ok(());
    }

    match args.command {
        args::EnveeCommand::Run {
            versions_file_path,
            no_commit_logs,
            plain_output,
            only_validate_versions,
        } => {
            let maybe_token = if no_commit_logs || only_validate_versions {
                None
            } else {
                get_env_var(ENV_VAR_GH_TOKEN)?
            };

            if !(no_commit_logs || only_validate_versions) && maybe_token.is_none() {
                anyhow::bail!(
                    "{ENV_VAR_GH_TOKEN} needs to be set to fetch commit logs from GitHub"
                );
            }

            let versions = get_versions_from_file(&versions_file_path)?;

            if only_validate_versions {
                println!("versions file is valid ✅");
                return Ok(());
            }

            let result = service::get_diff_result(versions.envs.clone(), &versions.versions);

            let config = Config {
                output_type: OutputType::Stdout(StdoutConfig {
                    table_preset: TablePreset::Nothing,
                    plain_output,
                }),
            };

            match &config.output_type {
                OutputType::Stdout(table_config) => {
                    println!(
                        "{}",
                        view::render_results_table(result.clone(), table_config)
                    );
                }
            }

            if no_commit_logs {
                return Ok(());
            }

            let token = maybe_token.with_context(|| format!("{ENV_VAR_GH_TOKEN} is not set"))?;

            let commit_logs = service::fetch_commit_logs(&result, &versions, &token).await;

            if !commit_logs.is_empty() {
                println!("\n{}", view::get_commit_logs(commit_logs, Utc::now()));
            }
        }
    }

    Ok(())
}

pub fn get_versions_from_file<P>(path: P) -> anyhow::Result<Versions>
where
    P: AsRef<Path>,
{
    let contents = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "couldn't read file \"{}\"",
            &path.as_ref().to_string_lossy()
        )
    })?;

    let versions: Versions = toml::from_str(&contents).with_context(|| {
        format!(
            r#"couldn't parse file "{}""#,
            &path.as_ref().to_string_lossy()
        )
    })?;

    Ok(versions)
}

fn get_env_var(key: &str) -> anyhow::Result<Option<String>> {
    match std::env::var(key) {
        Ok(v) => Ok(Some(v)),
        Err(e) => match e {
            VarError::NotPresent => Ok(None),
            VarError::NotUnicode(_) => Err(anyhow::anyhow!(
                r#"environment variable "{}"" is not valid unicode"#,
                key
            )),
        },
    }
}
