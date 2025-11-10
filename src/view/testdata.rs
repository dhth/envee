use crate::domain::{AppResult, Author, Commit, CommitDetail, CommitLog, DiffResult};
use chrono::{TimeZone, Utc};
use std::collections::HashMap;

pub(super) fn get_result_and_commit_logs() -> (DiffResult, Vec<CommitLog>) {
    let mut app1_values = HashMap::new();
    app1_values.insert("dev".into(), "1.1.0".into());
    app1_values.insert("prod".into(), "1.0.0".into());

    let mut app2_values = HashMap::new();
    app2_values.insert("dev".into(), "2.1.0".into());
    app2_values.insert("prod".into(), "2.0.0".into());

    let mut app3_values = HashMap::new();
    app3_values.insert("dev".into(), "1.5.0".into());
    app3_values.insert("prod".into(), "1.5.0".into());

    let diff_result = DiffResult {
        envs: vec!["dev", "prod"].into_iter().map(Into::into).collect(),
        app_results: vec![
            AppResult {
                app: "app-one".into(),
                values: app1_values,
                in_sync: false,
            },
            AppResult {
                app: "app-two".into(),
                values: app2_values,
                in_sync: false,
            },
            AppResult {
                app: "app-three".into(),
                values: app3_values,
                in_sync: true,
            },
        ],
    };

    let log1 = CommitLog {
        app: "app-one".into(),
        from_env: "prod".into(),
        to_env: "dev".into(),
        from_version: "1.0.0".into(),
        to_version: "1.1.0".into(),
        commits: vec![Commit {
            sha: "ae7de14".to_string(),
            commit: CommitDetail {
                message: "First commit".to_string(),
                author: Author {
                    name: "User A".to_string(),
                    date: Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).unwrap(),
                },
            },
            html_url: "https://github.com/org/app-one/commit/abc1234567890".to_string(),
        }],
        html_url: "https://github.com/org/app-one/compare/1.0.0...1.1.0".to_string(),
    };

    let log2 = CommitLog {
        app: "app-two".into(),
        from_env: "prod".into(),
        to_env: "dev".into(),
        from_version: "2.0.0".into(),
        to_version: "2.1.0".into(),
        commits: vec![
            Commit {
                sha: "1443d43".to_string(),
                commit: CommitDetail {
                    message: "add cli test for when no versions match app filter".to_string(),
                    author: Author {
                        name: "User A".to_string(),
                        date: Utc.with_ymd_and_hms(2025, 1, 16, 11, 30, 0).unwrap(),
                    },
                },
                html_url: "https://github.com/org/app-two/commit/1443d43".to_string(),
            },
            Commit {
                sha: "c536d77".to_string(),
                commit: CommitDetail {
                    message: "allow filtering apps to run for (#3) commit".to_string(),
                    author: Author {
                        name: "User B".to_string(),
                        date: Utc.with_ymd_and_hms(2025, 1, 16, 11, 0, 0).unwrap(),
                    },
                },
                html_url: "https://github.com/org/app-two/commit/c536d77".to_string(),
            },
            Commit {
                sha: "2ff3e97".to_string(),
                commit: CommitDetail {
                    message: "allow configuring table style (#2) commit".to_string(),
                    author: Author {
                        name: "User A".to_string(),
                        date: Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).unwrap(),
                    },
                },
                html_url: "https://github.com/org/app-two/commit/2ff3e97".to_string(),
            },
        ],
        html_url: "https://github.com/org/app-two/compare/2.0.0...2.1.0".to_string(),
    };

    (diff_result, vec![log1, log2])
}
