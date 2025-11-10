use crate::domain::{AppResult, Author, Commit, CommitDetail, CommitLog, DiffResult, SyncStatus};
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
                sync_status: SyncStatus::OutOfSync,
            },
            AppResult {
                app: "app-two".into(),
                values: app2_values,
                sync_status: SyncStatus::OutOfSync,
            },
            AppResult {
                app: "app-three".into(),
                values: app3_values,
                sync_status: SyncStatus::InSync,
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

pub(super) const TEST_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
  <title>{{ title }}</title>
</head>
<body>
  <h1>{{ title }}</h1>
  <p>Generated: {{ timestamp }}</p>

  <table>
    <thead>
      <tr>
        {%- for column in columns %}
        <th>{{ column }}</th>
        {%- endfor %}
      </tr>
    </thead>
    <tbody>
      {%- for row in rows %}
      <tr>
        {%- for cell in row.data %}
        <td>{{ cell }}</td>
        {%- endfor %}
      </tr>
      {%- endfor %}
    </tbody>
  </table>

  {%- if commit_logs %}
  <h2>Commit Logs</h2>
  {%- for log in commit_logs %}
  <div>
    <h3>{{ log.app }}</h3>
    <p>{{ log.from_env }}..{{ log.to_env }} ({{ log.from_version }}...{{ log.to_version }})</p>
    <p>Compare: <a href="{{ log.compare_url }}">{{ log.compare_url }}</a></p>
    <ul>
      {%- for commit in log.commits %}
      <li>
        <a href="{{ commit.html_url }}">{{ commit.short_sha }}</a>
        - {{ commit.message }}
        - {{ commit.author }}
        - {{ commit.date }}
      </li>
      {%- endfor %}
    </ul>
  </div>
  {%- endfor %}
  {%- endif %}
</body>
</html>
"#;
