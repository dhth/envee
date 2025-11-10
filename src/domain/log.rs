use super::{App, Env, Version};
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug)]
pub struct CommitLog {
    pub app: App,
    pub from_env: Env,
    pub to_env: Env,
    pub from_version: Version,
    pub to_version: Version,
    pub commits: Vec<Commit>,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub commit: CommitDetail,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct CommitDetail {
    pub message: String,
    pub author: Author,
}

#[derive(Debug, Deserialize)]
pub struct Author {
    pub name: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CommitLogResults {
    pub logs: Vec<CommitLog>,
    pub errors: CommitLogFetchErrors,
}

#[derive(Debug)]
pub enum CommitLogFetchError {
    App { app: App, error: anyhow::Error },
    System { error: anyhow::Error },
}

#[derive(Debug)]
pub struct CommitLogFetchErrors {
    errors: Vec<CommitLogFetchError>,
}

impl CommitLogFetchErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_app_error(&mut self, app: App, error: anyhow::Error) {
        self.errors.push(CommitLogFetchError::App { app, error });
    }

    pub fn add_system_error(&mut self, error: anyhow::Error) {
        self.errors.push(CommitLogFetchError::System { error });
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl std::fmt::Display for CommitLogFetchErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "couldn't fetch commit logs for some apps:")?;
        for error in &self.errors {
            match error {
                CommitLogFetchError::App { app, error } => {
                    writeln!(f, " - {}: {}", app, error)?;
                }
                CommitLogFetchError::System { error } => {
                    writeln!(f, " - system error: {}", error)?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for CommitLogFetchErrors {}
