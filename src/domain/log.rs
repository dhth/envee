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
