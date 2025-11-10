use crate::domain::{CommitLog, DiffResult};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tera::Tera;

pub const BUILT_IN_TEMPLATE: &str = include_str!("assets/template.html");

#[derive(Serialize)]
struct HtmlData {
    title: String,
    timestamp: String,
    columns: Vec<String>,
    rows: Vec<HtmlRow>,
    commit_logs: Vec<HtmlCommitLog>,
}

#[derive(Serialize)]
struct HtmlRow {
    data: Vec<String>,
    in_sync: bool,
}

#[derive(Serialize)]
struct HtmlCommitLog {
    app: String,
    from_env: String,
    to_env: String,
    from_version: String,
    to_version: String,
    compare_url: Option<String>,
    commits: Vec<HtmlCommit>,
}

#[derive(Serialize)]
struct HtmlCommit {
    short_sha: String,
    html_url: String,
    message: String,
    author: String,
    date: String,
}

pub fn render_html(
    diff_result: &DiffResult,
    commit_logs: &[CommitLog],
    template: &str,
    title: &str,
    now: DateTime<Utc>,
) -> Result<String> {
    let mut tera = Tera::default();

    tera.add_raw_template("html", template)
        .context("failed to parse HTML template")?;

    let html_data = build_html_data(diff_result, commit_logs, title, now);

    let mut context = tera::Context::new();
    context.insert("title", &html_data.title);
    context.insert("timestamp", &html_data.timestamp);
    context.insert("columns", &html_data.columns);
    context.insert("rows", &html_data.rows);
    context.insert("commit_logs", &html_data.commit_logs);

    tera.render("html", &context)
        .context("failed to render HTML template")
}

fn build_html_data(
    diff_result: &DiffResult,
    commit_logs: &[CommitLog],
    title: &str,
    now: DateTime<Utc>,
) -> HtmlData {
    let mut columns = vec!["app".to_string()];
    columns.extend(diff_result.envs.iter().map(|e| e.to_string()));
    columns.push("in sync".to_string());

    let rows: Vec<HtmlRow> = diff_result
        .app_results
        .iter()
        .map(|app_result| {
            let mut row_data = vec![app_result.app.to_string()];

            for env in &diff_result.envs {
                let version_str = app_result
                    .values
                    .get(env)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string());
                row_data.push(version_str);
            }

            let in_sync_str = if app_result.in_sync { "✓" } else { "✗" };
            row_data.push(in_sync_str.to_string());

            HtmlRow {
                data: row_data,
                in_sync: app_result.in_sync,
            }
        })
        .collect();

    let html_commit_logs: Vec<HtmlCommitLog> = commit_logs
        .iter()
        .map(|log| {
            let commits: Vec<HtmlCommit> = log
                .commits
                .iter()
                .map(|commit| {
                    let short_sha = commit.sha.chars().take(7).collect::<String>();
                    let html_url = commit.html_url.clone();
                    let message = commit
                        .commit
                        .message
                        .lines()
                        .next()
                        .unwrap_or(&commit.commit.message)
                        .to_string();
                    let author = commit.commit.author.name.clone();
                    let date = commit.commit.author.date.format("%b %e, %Y").to_string();

                    HtmlCommit {
                        short_sha,
                        html_url,
                        message,
                        author,
                        date,
                    }
                })
                .collect();

            let compare_url = if !commits.is_empty() {
                Some(log.html_url.clone())
            } else {
                None
            };

            HtmlCommitLog {
                app: log.app.to_string(),
                from_env: log.from_env.to_string(),
                to_env: log.to_env.to_string(),
                from_version: log.from_version.to_string(),
                to_version: log.to_version.to_string(),
                compare_url,
                commits,
            }
        })
        .collect();

    HtmlData {
        title: title.to_string(),
        timestamp: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        columns,
        rows,
        commit_logs: html_commit_logs,
    }
}

#[cfg(test)]
mod tests {
    use super::super::testdata::{TEST_HTML_TEMPLATE, get_result_and_commit_logs};
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn built_in_html_template_is_rendered_correctly() {
        // GIVEN
        let (diff_result, commit_logs) = get_result_and_commit_logs();
        let now = Utc.with_ymd_and_hms(2025, 1, 16, 12, 0, 0).unwrap();

        // WHEN
        let html = render_html(
            &diff_result,
            &commit_logs,
            BUILT_IN_TEMPLATE,
            "versions",
            now,
        )
        .expect("result should've been Ok");

        // THEN
        insta::assert_snapshot!(html, @r#"
        <!DOCTYPE html>
        <html lang="en">
          <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>
            <link rel="icon" type="image/png" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAACN0lEQVR4AUxSO2tUURD+5kpWd0EtgkSFJYogu3/AQlubRdNaqbGIyhYahGAgBOIzoAY0gkYlCxHEQhAkwQcWChYShETEIojGTeGzSJAUPlI4+b65d7M5nDnzzeubOffcpFQq+WopZ3ZDl8qMSzK/csuZLZ0YshXA4JnpCEdqOZVlNrW7RdTpTyiMNraDcViYHucqRtqMqIpIUTNDgsaih2E42QkhFuNh0NIZXhpN7O5NApKtBIUre3fhxrVTuFfrx9m+TmzZ3AqYSCSESFc6AUnllrApjhzch8sXqtjUuhGfZr9gf2U37lzvwZokTVcOB4V0eJrEjvzaHKpdHZhfWMShoxcxMDiGJ88n0d7ehp07iogVnYRMV2B7bmRr+7atyK/LYcP6Ap49GsLkyxF0VPaoGXK5lsgynSSx9BsQyaEURnR/53z1ue/o7b+Jkz3D6DoxhM5jl/B+ZlZZfJg0UZXN/0Ak9NTnfuDf0hKKxTZ8/TaPN1MfMDU9g4Vfi/D/YDETeWd1J0L8B6wj9mD//ecvancf8xoteHj/HMZu9WL8wSCunD8OPRt7MzclEs4mEJSfmvt2bRx9Z0bx9t1HFAp5fK7/xMjoRDRwpUmYJxwTkDqC4jXe32lNPH2NavdVHDg8gO7Tw3jxajodn8XKCYMkfEaeLHAGPNNUiI9p8jAgo4FpelQTwPSMADXFQqVTMMURSyo+WExGl1G4Q7mLgCkEukYUM+KkopIJNXcWhEHNQYAIItYyAAAA//8sM7n7AAAABklEQVQDAO2/0Ju6kMR1AAAAAElFTkSuQmCC">
            <title>versions</title>
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=Fira+Mono:wght@400;500;700&family=Open+Sans:ital,wght@0,300..800;1,300..800&display=swap" rel="stylesheet">
            <style>
              body {
                font-family: "Open Sans", sans-serif;
              }
              .changes-section {
                font-family: "Fira Mono", monospace;
              }
            </style>
          </head>
          <body class="bg-[#282828]">
            <div class="w-2/3 max-sm:w-full max-sm:px-4 mx-auto min-h-screen pt-8">
              <h1 class="text-[#fbf1c7] text-3xl mb-4 font-semibold">
                versions
              </h1>
              <p class="text-[#928374] italic mt-4">Generated at 2025-01-16T12:00:00Z</p>
              <div class="mt-2 overflow-x-auto">
                <table class="table-auto w-full text-right max-sm:text-xs font-semibold whitespace-nowrap">
                  <thead>
                    <tr class="text-[#fbf1c7] bg-[#3c3836]">
                      <th class="px-10 py-2">app</th>
                      <th class="px-10 py-2">dev</th>
                      <th class="px-10 py-2">prod</th>
                      <th class="px-10 py-2">in sync</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr class="text-[#fb4934]">
                      <td class="px-10 py-2">app-one</td>
                      <td class="px-10 py-2">1.1.0</td>
                      <td class="px-10 py-2">1.0.0</td>
                      <td class="px-10 py-2">✗</td>
                    </tr>
                    <tr class="text-[#fb4934]">
                      <td class="px-10 py-2">app-two</td>
                      <td class="px-10 py-2">2.1.0</td>
                      <td class="px-10 py-2">2.0.0</td>
                      <td class="px-10 py-2">✗</td>
                    </tr>
                    <tr class="text-[#b8bb26]">
                      <td class="px-10 py-2">app-three</td>
                      <td class="px-10 py-2">1.5.0</td>
                      <td class="px-10 py-2">1.5.0</td>
                      <td class="px-10 py-2">✓</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <div class="overflow-x-auto">
                <div class="flex gap-4 items-center mt-8">
                  <p class="text-[#fabd2f] text-xl font-semibold">Changes</p>
                  <button class="bg-[#83a598] text-[#282828] font-semibold text-xs p-2 hover:bg-[#fabd2f]" onclick="toggleAllDetails()">
                  Toggle All
                  </button>
                </div>
                <div class="my-4 overflow-x-auto">
                  <details>
                    <summary class="text-[#83a598] cursor-pointer max-sm:text-sm">app-one</summary>
                    <div class="mt-2 max-sm:p-2 p-4 bg-[#2e2c2c] changes-section max-sm:text-xs text-sm">
                      <div class="flex flex-col items-left gap-4 overflow-x-auto">
                        <a class="text-[#928374]" href="https://github.com/org/app-one/compare/1.0.0...1.1.0" target="_blank">prod..dev (1.0.0...1.1.0)</a>
                        <table class="w-full text-left max-sm:text-xs text-sm whitespace-nowrap">
                          <tbody>
                            <tr class="">
                              <td class="px-4 py-1 text-[#fabd2f]"><a target="_blank" href="https://github.com/org/app-one/commit/abc1234567890">ae7de14</a></td>
                              <td class="px-4 py-1 text-[#83a598]"><a target="_blank" href="https://github.com/org/app-one/commit/abc1234567890">First commit</a></td>
                              <td class="px-4 py-1 text-[#d3869b]"><a target="_blank" href="https://github.com/org/app-one/commit/abc1234567890">User A</a></td>
                              <td class="px-4 py-1 text-[#bdae93]"><a target="_blank" href="https://github.com/org/app-one/commit/abc1234567890">Jan 15, 2025</a></td>
                            </tr>
                          </tbody>
                        </table>
                      </div>
                    </div>
                  </details>
                </div>
                <div class="my-4 overflow-x-auto">
                  <details>
                    <summary class="text-[#83a598] cursor-pointer max-sm:text-sm">app-two</summary>
                    <div class="mt-2 max-sm:p-2 p-4 bg-[#2e2c2c] changes-section max-sm:text-xs text-sm">
                      <div class="flex flex-col items-left gap-4 overflow-x-auto">
                        <a class="text-[#928374]" href="https://github.com/org/app-two/compare/2.0.0...2.1.0" target="_blank">prod..dev (2.0.0...2.1.0)</a>
                        <table class="w-full text-left max-sm:text-xs text-sm whitespace-nowrap">
                          <tbody>
                            <tr class="">
                              <td class="px-4 py-1 text-[#fabd2f]"><a target="_blank" href="https://github.com/org/app-two/commit/1443d43">1443d43</a></td>
                              <td class="px-4 py-1 text-[#83a598]"><a target="_blank" href="https://github.com/org/app-two/commit/1443d43">add cli test for when no versions match app filter</a></td>
                              <td class="px-4 py-1 text-[#d3869b]"><a target="_blank" href="https://github.com/org/app-two/commit/1443d43">User A</a></td>
                              <td class="px-4 py-1 text-[#bdae93]"><a target="_blank" href="https://github.com/org/app-two/commit/1443d43">Jan 16, 2025</a></td>
                            </tr>
                            <tr class="">
                              <td class="px-4 py-1 text-[#fabd2f]"><a target="_blank" href="https://github.com/org/app-two/commit/c536d77">c536d77</a></td>
                              <td class="px-4 py-1 text-[#83a598]"><a target="_blank" href="https://github.com/org/app-two/commit/c536d77">allow filtering apps to run for (#3) commit</a></td>
                              <td class="px-4 py-1 text-[#d3869b]"><a target="_blank" href="https://github.com/org/app-two/commit/c536d77">User B</a></td>
                              <td class="px-4 py-1 text-[#bdae93]"><a target="_blank" href="https://github.com/org/app-two/commit/c536d77">Jan 16, 2025</a></td>
                            </tr>
                            <tr class="">
                              <td class="px-4 py-1 text-[#fabd2f]"><a target="_blank" href="https://github.com/org/app-two/commit/2ff3e97">2ff3e97</a></td>
                              <td class="px-4 py-1 text-[#83a598]"><a target="_blank" href="https://github.com/org/app-two/commit/2ff3e97">allow configuring table style (#2) commit</a></td>
                              <td class="px-4 py-1 text-[#d3869b]"><a target="_blank" href="https://github.com/org/app-two/commit/2ff3e97">User A</a></td>
                              <td class="px-4 py-1 text-[#bdae93]"><a target="_blank" href="https://github.com/org/app-two/commit/2ff3e97">Jan 15, 2025</a></td>
                            </tr>
                          </tbody>
                        </table>
                      </div>
                    </div>
                  </details>
                </div>
              </div>
              <p class="text-[#928374] italic my-10 pt-2 border-t-2 border-[#92837433]">Built using <a class="font-bold" href="https://github.com/dhth/envee" target="_blank">envee</a></p>
            </div>
            <button id="scrollToTop" onclick="window.scrollTo({top: 0, behavior: 'instant'});"
              class="hidden fixed bottom-4 left-4 z-50 bg-[#928374] text-[#282828] px-4 py-2 rounded-full shadow-lg hover:bg-[#d3869b] font-bold transition"
              aria-label="Go to top">
            ↑
            </button>
          </body>
          <script>
        const scrollToTopButton = document.getElementById("scrollToTop");
        let allDetailsOpen = false;

        function toggleAllDetails() {
          allDetailsOpen = !allDetailsOpen;
          document.querySelectorAll("details").forEach((detail) => {
              detail.open = allDetailsOpen;
          });
        }

        window.addEventListener("scroll", function () {
          if (window.scrollY > 100) {
              scrollToTopButton.classList.remove("hidden");
          } else {
              scrollToTopButton.classList.add("hidden");
          }
        });
          </script>
        </html>
        "#);
    }

    #[test]
    fn custom_html_template_is_rendered_correctly() {
        // GIVEN
        let (diff_result, commit_logs) = get_result_and_commit_logs();
        let now = Utc.with_ymd_and_hms(2025, 1, 16, 12, 0, 0).unwrap();

        // WHEN
        let html = render_html(
            &diff_result,
            &commit_logs,
            TEST_HTML_TEMPLATE,
            "versions",
            now,
        )
        .expect("result should've been Ok");

        // THEN
        insta::assert_snapshot!(html, @r#"
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
}
