use crate::domain::{
    App, Commit, CommitLog, CommitLogFetchErrors, CommitLogResults, DiffResult, Env,
    GitTagTransform, GithubOrg, SyncStatus, Version, Versions,
};
use anyhow::Context;
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Semaphore;

const MAX_CONCURRENT_FETCHES: usize = 20;
const DEFAULT_API_URL: &str = "https://api.github.com";

#[derive(Clone)]
pub struct GitHubClient {
    http: reqwest::Client,
    api_url: String,
}

impl GitHubClient {
    pub fn new(api_url: Option<String>, token: &str) -> anyhow::Result<Self> {
        let api_url = match api_url {
            Some(value) => {
                let value = value.trim();
                anyhow::ensure!(!value.is_empty(), "GitHub API base URL must not be empty");
                value.trim_end_matches('/').to_string()
            }
            None => DEFAULT_API_URL.to_string(),
        };

        let mut authorization = HeaderValue::from_str(&format!("Bearer {token}"))
            .context("invalid GitHub authorization header")?;
        authorization.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authorization);
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent("envee@v0.1.0")
            .build()
            .context("failed to build GitHub HTTP client")?;

        Ok(Self { http, api_url })
    }

    async fn fetch_commit_log(&self, params: FetchCommitLogParams) -> anyhow::Result<CommitLog> {
        let base_tag = if let Some(ref template) = params.tag_transform {
            build_tag(template, &params.from_version)
        } else {
            params.from_version.to_string()
        };

        let head_tag = if let Some(ref template) = params.tag_transform {
            build_tag(template, &params.to_version)
        } else {
            params.to_version.to_string()
        };

        let url = format!(
            "{}/repos/{}/{}/compare/{}...{}",
            self.api_url, params.github_org, params.app, base_tag, head_tag
        );

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .context("failed to send request to GitHub API")?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "GitHub API request failed with status {}: {}",
                status,
                error_body
            );
        }

        let mut compare_response: CompareResponse = response
            .json()
            .await
            .context("failed to parse GitHub API response")?;

        compare_response.commits.reverse();

        Ok(CommitLog {
            app: params.app,
            from_env: params.from_env,
            to_env: params.to_env,
            from_version: params.from_version,
            to_version: params.to_version,
            commits: compare_response.commits,
            html_url: compare_response.html_url,
        })
    }
}

pub async fn fetch_commit_logs(
    client: &GitHubClient,
    diff_result: &DiffResult,
    versions: &Versions,
) -> CommitLogResults {
    let out_of_sync: Vec<_> = diff_result
        .app_results
        .iter()
        .filter(|row| matches!(row.sync_status, SyncStatus::OutOfSync))
        .collect();

    if out_of_sync.is_empty() {
        return CommitLogResults {
            logs: vec![],
            errors: CommitLogFetchErrors::new(),
        };
    }

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_FETCHES));
    let mut futures = FuturesUnordered::new();

    for row in out_of_sync {
        let semaphore = Arc::clone(&semaphore);
        let github_org = versions.github_org.clone();
        let app = row.app.clone();
        let tag_transform = versions.git_tag_transform.clone();
        let client = client.clone();

        let from_env = diff_result.envs[diff_result.envs.len() - 1].clone();
        let to_env = diff_result.envs[0].clone();

        let Some(from_version) = row.values.get(&from_env).cloned() else {
            continue;
        };

        let Some(to_version) = row.values.get(&to_env).cloned() else {
            continue;
        };

        let app_clone = app.clone();

        futures.push(tokio::task::spawn(async move {
            let permit = semaphore.acquire().await;
            if let Err(e) = permit {
                return (
                    app_clone,
                    Err(anyhow::anyhow!("couldn't acquire semaphore: {e}")),
                );
            }

            let result = client
                .fetch_commit_log(FetchCommitLogParams {
                    github_org,
                    app,
                    from_env,
                    to_env,
                    from_version,
                    to_version,
                    tag_transform,
                })
                .await;

            (app_clone, result)
        }));
    }

    let mut commit_logs = Vec::new();
    let mut errors = CommitLogFetchErrors::new();

    while let Some(task_result) = futures.next().await {
        match task_result {
            Ok((_app, Ok(log))) => commit_logs.push(log),
            Ok((app, Err(e))) => {
                errors.add_app_error(app, e);
            }
            Err(e) => {
                errors.add_system_error(anyhow::anyhow!("task panicked: {e}"));
            }
        }
    }

    commit_logs.sort_by(|a, b| a.app.cmp(&b.app));

    CommitLogResults {
        logs: commit_logs,
        errors,
    }
}

struct FetchCommitLogParams {
    github_org: GithubOrg,
    app: App,
    from_env: Env,
    to_env: Env,
    from_version: Version,
    to_version: Version,
    tag_transform: Option<GitTagTransform>,
}

#[derive(Debug, Deserialize)]
struct CompareResponse {
    commits: Vec<Commit>,
    html_url: String,
}

fn build_tag(template: &str, version: &str) -> String {
    template.replacen("{{version}}", version, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_tag_works() {
        assert_eq!(build_tag("v{{version}}", "0.83.0"), "v0.83.0");
        assert_eq!(build_tag("release-{{version}}", "1.2.3"), "release-1.2.3");
        assert_eq!(build_tag("{{version}}", "2.0.0"), "2.0.0");
        assert_eq!(
            build_tag("v{{version}}-{{version}}", "2.0.0"),
            "v2.0.0-{{version}}"
        );
    }
}
