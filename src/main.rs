mod config;
mod domain;
mod service;
mod view;

use crate::config::StdoutConfig;
use anyhow::Context;
use chrono::Utc;
use config::{Config, OutputType, TablePreset};
use domain::Versions;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = std::env::var("ENVEE_GH_TOKEN").context("ENVEE_GH_TOKEN is not set")?;

    let versions = get_versions_from_file(PathBuf::from("versions.toml"))?;
    let result = service::get_diff_result(versions.envs.clone(), &versions.versions);

    let config = Config {
        output_type: OutputType::Stdout(StdoutConfig {
            table_preset: TablePreset::Nothing,
            highlight_out_of_sync: true,
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

    let commit_logs = service::fetch_commit_logs(&result, &versions, &token).await;

    if !commit_logs.is_empty() {
        println!("\n{}", view::get_commit_logs(commit_logs, Utc::now()));
    }

    Ok(())
}

pub fn get_versions_from_file(path: PathBuf) -> anyhow::Result<Versions> {
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("couldn't read file \"{}\"", &path.to_string_lossy()))?;

    let versions: Versions = toml::from_str(&contents)
        .with_context(|| format!(r#"couldn't parse file "{}""#, &path.to_string_lossy()))?;

    Ok(versions)
}
