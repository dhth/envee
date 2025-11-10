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
    pub errors: CommitLogErrors,
}

#[derive(Debug)]
pub struct CommitLogErrors {
    errors: Vec<CommitLogError>,
}

impl CommitLogErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, error: CommitLogError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl std::fmt::Display for CommitLogErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "failed to fetch commit logs for some apps:")?;
        for error in &self.errors {
            writeln!(f, " - {}: {}", error.app, error.error)?;
        }
        Ok(())
    }
}

impl std::error::Error for CommitLogErrors {}

#[derive(Debug)]
pub struct CommitLogError {
    pub app: App,
    pub error: anyhow::Error,
}
