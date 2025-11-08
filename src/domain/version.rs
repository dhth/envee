use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type App = String;
pub type Env = String;
pub type Version = String;

#[derive(Debug, Clone, Deserialize)]
pub struct Versions {
    pub envs: Vec<Env>,
    pub github_org: String,
    pub versions: Vec<AppVersion>,
    pub git_tag_transform: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppVersion {
    pub app: App,
    pub env: Env,
    pub version: Version,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub envs: Vec<Env>,
    pub app_results: Vec<AppResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppResult {
    pub app: App,
    pub values: HashMap<Env, Version>,
    pub in_sync: bool,
}
