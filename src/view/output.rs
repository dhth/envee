use crate::config::{Config, OutputType};
use crate::domain::{CommitLogResults, DiffResult};
use chrono::{DateTime, Utc};

pub fn render_output(
    diff_result: &DiffResult,
    commit_log_results: Option<&CommitLogResults>,
    config: &Config,
    now: DateTime<Utc>,
) -> anyhow::Result<String> {
    let output = match &config.output_type {
        OutputType::Stdout(stdout_config) => {
            let mut output = super::render_results_table(diff_result, stdout_config);

            if let Some(results) = commit_log_results
                && !results.logs.is_empty()
            {
                output.push_str("\n\n");
                output.push_str(&super::render_commit_logs(
                    &results.logs,
                    now,
                    stdout_config.plain_output,
                ));
            }

            output
        }
        OutputType::Html(html_config) => {
            let commit_logs = commit_log_results.map(|r| &r.logs[..]).unwrap_or(&[]);

            super::render_html(
                diff_result,
                commit_logs,
                &html_config.template,
                &html_config.title,
                now,
            )?
        }
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::super::testdata::{TEST_HTML_TEMPLATE, get_result_and_commit_logs};
    use super::*;
    use crate::config::{HtmlConfig, StdoutConfig, TableStyle};
    use crate::domain::{CommitLogFetchErrors, CommitLogResults};
    use chrono::TimeZone;
    use std::path::PathBuf;

    #[test]
    fn getting_stdout_output_with_plain_output_and_commit_logs_works() {
        // GIVEN
        let (diff_result, logs) = get_result_and_commit_logs();
        let commit_log_results = Some(CommitLogResults {
            logs,
            errors: CommitLogFetchErrors::new(),
        });
        let config = Config {
            output_type: OutputType::Stdout(StdoutConfig {
                table_style: TableStyle::Ascii,
                plain_output: true,
            }),
        };
        let now = Utc.with_ymd_and_hms(2025, 1, 16, 12, 0, 0).unwrap();

        // WHEN
        let result =
            render_output(&diff_result, commit_log_results.as_ref(), &config, now).unwrap();

        // THEN
        insta::assert_snapshot!(result, @r"
        +----------+-------+-------+---------+
        |app       | dev   | prod  | in-sync |
        +====================================+
        |app-one   | 1.1.0 | 1.0.0 | NO      |
        |app-two   | 2.1.0 | 2.0.0 | NO      |
        |app-three | 1.5.0 | 1.5.0 | YES     |
        +----------+-------+-------+---------+

        app-one prod..dev (1.0.0..1.1.0)

         ae7de14  First commit  User A  1d ago 

        app-two prod..dev (2.0.0..2.1.0)

         1443d43  add cli test for when no versions match app filter  User A  30m ago 
         c536d77  allow filtering apps to run for (#3) commit         User B  1h ago  
         2ff3e97  allow configuring table style (#2) commit           User A  1d ago
        ");
    }

    #[test]
    fn getting_stdout_output_with_plain_output_without_commit_logs_works() {
        // GIVEN
        let (diff_result, _) = get_result_and_commit_logs();
        let commit_log_results: Option<CommitLogResults> = None;
        let config = Config {
            output_type: OutputType::Stdout(StdoutConfig {
                table_style: TableStyle::Ascii,
                plain_output: true,
            }),
        };
        let now = Utc.with_ymd_and_hms(2025, 1, 16, 12, 0, 0).unwrap();

        // WHEN
        let result =
            render_output(&diff_result, commit_log_results.as_ref(), &config, now).unwrap();

        // THEN
        insta::assert_snapshot!(result, @r"
        +----------+-------+-------+---------+
        |app       | dev   | prod  | in-sync |
        +====================================+
        |app-one   | 1.1.0 | 1.0.0 | NO      |
        |app-two   | 2.1.0 | 2.0.0 | NO      |
        |app-three | 1.5.0 | 1.5.0 | YES     |
        +----------+-------+-------+---------+
        ");
    }

    #[test]
    fn getting_stdout_output_with_plain_output_and_empty_commit_logs_works() {
        // GIVEN
        let (diff_result, _) = get_result_and_commit_logs();
        let commit_log_results = Some(CommitLogResults {
            logs: vec![],
            errors: CommitLogFetchErrors::new(),
        });
        let config = Config {
            output_type: OutputType::Stdout(StdoutConfig {
                table_style: TableStyle::Ascii,
                plain_output: true,
            }),
        };
        let now = Utc.with_ymd_and_hms(2025, 1, 16, 12, 0, 0).unwrap();

        // WHEN
        let result =
            render_output(&diff_result, commit_log_results.as_ref(), &config, now).unwrap();

        // THEN
        insta::assert_snapshot!(result, @r"
        +----------+-------+-------+---------+
        |app       | dev   | prod  | in-sync |
        +====================================+
        |app-one   | 1.1.0 | 1.0.0 | NO      |
        |app-two   | 2.1.0 | 2.0.0 | NO      |
        |app-three | 1.5.0 | 1.5.0 | YES     |
        +----------+-------+-------+---------+
        ");
    }

    #[test]
    fn getting_html_output_with_commit_logs_works() {
        // GIVEN
        let (diff_result, logs) = get_result_and_commit_logs();
        let commit_log_results = Some(CommitLogResults {
            logs,
            errors: CommitLogFetchErrors::new(),
        });
        let config = Config {
            output_type: OutputType::Html(HtmlConfig {
                output_path: PathBuf::from("/tmp/output.html"),
                title: "versions".to_string(),
                template: TEST_HTML_TEMPLATE.to_string(),
            }),
        };
        let now = Utc.with_ymd_and_hms(2025, 1, 16, 12, 0, 0).unwrap();

        // WHEN
        let result =
            render_output(&diff_result, commit_log_results.as_ref(), &config, now).unwrap();

        // THEN
        insta::assert_snapshot!(result, @r#"
        <!DOCTYPE html>
        <html>
        <head>
          <title>versions</title>
        </head>
        <body>
          <h1>versions</h1>
          <p>Generated: 2025-01-16T12:00:00Z</p>

          <table>
            <thead>
              <tr>
                <th>app</th>
                <th>dev</th>
                <th>prod</th>
                <th>in sync</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>app-one</td>
                <td>1.1.0</td>
                <td>1.0.0</td>
                <td>✗</td>
              </tr>
              <tr>
                <td>app-two</td>
                <td>2.1.0</td>
                <td>2.0.0</td>
                <td>✗</td>
              </tr>
              <tr>
                <td>app-three</td>
                <td>1.5.0</td>
                <td>1.5.0</td>
                <td>✓</td>
              </tr>
            </tbody>
          </table>
          <h2>Commit Logs</h2>
          <div>
            <h3>app-one</h3>
            <p>prod..dev (1.0.0...1.1.0)</p>
            <p>Compare: <a href="https://github.com/org/app-one/compare/1.0.0...1.1.0">https://github.com/org/app-one/compare/1.0.0...1.1.0</a></p>
            <ul>
              <li>
                <a href="https://github.com/org/app-one/commit/abc1234567890">ae7de14</a>
                - First commit
                - User A
                - Jan 15, 2025
              </li>
            </ul>
          </div>
          <div>
            <h3>app-two</h3>
            <p>prod..dev (2.0.0...2.1.0)</p>
            <p>Compare: <a href="https://github.com/org/app-two/compare/2.0.0...2.1.0">https://github.com/org/app-two/compare/2.0.0...2.1.0</a></p>
            <ul>
              <li>
                <a href="https://github.com/org/app-two/commit/1443d43">1443d43</a>
                - add cli test for when no versions match app filter
                - User A
                - Jan 16, 2025
              </li>
              <li>
                <a href="https://github.com/org/app-two/commit/c536d77">c536d77</a>
                - allow filtering apps to run for (#3) commit
                - User B
                - Jan 16, 2025
              </li>
              <li>
                <a href="https://github.com/org/app-two/commit/2ff3e97">2ff3e97</a>
                - allow configuring table style (#2) commit
                - User A
                - Jan 15, 2025
              </li>
            </ul>
          </div>
        </body>
        </html>
        "#);
    }

    #[test]
    fn getting_html_output_without_commit_logs_works() {
        // GIVEN
        let (diff_result, _) = get_result_and_commit_logs();
        let commit_log_results: Option<CommitLogResults> = None;
        let config = Config {
            output_type: OutputType::Html(HtmlConfig {
                output_path: PathBuf::from("/tmp/output.html"),
                title: "versions".to_string(),
                template: TEST_HTML_TEMPLATE.to_string(),
            }),
        };
        let now = Utc.with_ymd_and_hms(2025, 1, 16, 12, 0, 0).unwrap();

        // WHEN
        let result =
            render_output(&diff_result, commit_log_results.as_ref(), &config, now).unwrap();

        // THEN
        insta::assert_snapshot!(result, @r"
        <!DOCTYPE html>
        <html>
        <head>
          <title>versions</title>
        </head>
        <body>
          <h1>versions</h1>
          <p>Generated: 2025-01-16T12:00:00Z</p>

          <table>
            <thead>
              <tr>
                <th>app</th>
                <th>dev</th>
                <th>prod</th>
                <th>in sync</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>app-one</td>
                <td>1.1.0</td>
                <td>1.0.0</td>
                <td>✗</td>
              </tr>
              <tr>
                <td>app-two</td>
                <td>2.1.0</td>
                <td>2.0.0</td>
                <td>✗</td>
              </tr>
              <tr>
                <td>app-three</td>
                <td>1.5.0</td>
                <td>1.5.0</td>
                <td>✓</td>
              </tr>
            </tbody>
          </table>
        </body>
        </html>
        ");
    }
}
