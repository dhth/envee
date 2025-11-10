mod args;
mod config;
mod domain;
mod service;
mod versions;
mod view;

use crate::config::{HtmlConfig, OutputFormat, StdoutConfig};
use anyhow::Context;
use args::Args;
use chrono::Utc;
use clap::Parser;
use config::{Config, OutputType};
use regex::Regex;
use std::env::VarError;

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
            output_format,
            table_style,
            plain_output,
            html_output_path,
            html_title,
            html_template_path,
            only_validate_versions,
            app_filter,
        } => {
            // VALIDATIONS
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

            let app_filter = app_filter
                .map(|pattern| Regex::new(&pattern))
                .transpose()
                .context("invalid regex pattern provided")?;

            let html_template = if let Some(ref template_path) = html_template_path {
                std::fs::read_to_string(template_path).with_context(|| {
                    format!("failed to read HTML template from {:?}", template_path)
                })?
            } else {
                view::BUILT_IN_TEMPLATE.to_string()
            };

            // VALIDATIONS END

            let versions = versions::get_from_file(&versions_file_path, app_filter.as_ref())?;

            if only_validate_versions {
                println!("versions file is valid ✅");
                return Ok(());
            }

            let result = service::get_diff_result(versions.envs.clone(), &versions.versions);

            let config = Config {
                output_type: match output_format {
                    OutputFormat::Stdout => OutputType::Stdout(StdoutConfig {
                        table_style,
                        plain_output,
                    }),
                    OutputFormat::Html => OutputType::Html(HtmlConfig {
                        output_path: html_output_path,
                        title: html_title,
                    }),
                },
            };

            match &config.output_type {
                OutputType::Stdout(stdout_config) => {
                    println!("{}", view::render_results_table(&result, stdout_config));

                    if no_commit_logs {
                        return Ok(());
                    }
                }
                OutputType::Html(_) => {}
            }

            let commit_logs = if no_commit_logs {
                vec![]
            } else {
                let token =
                    maybe_token.with_context(|| format!("{ENV_VAR_GH_TOKEN} is not set"))?;
                service::fetch_commit_logs(&result, &versions, &token).await
            };

            let now = Utc::now();

            match &config.output_type {
                OutputType::Stdout(stdout_config) => {
                    if !commit_logs.is_empty() {
                        println!(
                            "\n{}",
                            view::get_commit_logs(commit_logs, now, stdout_config.plain_output)
                        );
                    }
                }
                OutputType::Html(html_config) => {
                    let html = view::render_html(
                        &result,
                        &commit_logs,
                        &html_template,
                        &html_config.title,
                        now,
                    )?;

                    std::fs::write(&html_config.output_path, html).with_context(|| {
                        format!("failed to write HTML to {:?}", html_config.output_path)
                    })?;

                    println!(
                        "HTML report written to: {}",
                        html_config.output_path.display()
                    );
                }
            }
        }
    }

    Ok(())
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
