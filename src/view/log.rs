use super::date::get_humanized_date;
use crate::domain::CommitLog;
use chrono::{DateTime, Utc};

pub fn get_commit_logs(logs: Vec<CommitLog>, reference_time: DateTime<Utc>) -> String {
    let mut output = String::new();

    for (i, log) in logs.iter().enumerate() {
        output.push_str(&format!(
            "{} {}..{} ({}..{})\n\n",
            log.app, log.from_env, log.to_env, log.from_version, log.to_version
        ));

        for commit in &log.commits {
            let short_sha = &commit.sha[..7.min(commit.sha.len())];
            let first_line = commit
                .commit
                .message
                .lines()
                .next()
                .unwrap_or(&commit.commit.message);

            let relative_time = get_humanized_date(&commit.commit.author.date, &reference_time);

            output.push_str(&format!(
                " {} - {} ({}) <{}>\n",
                short_sha, first_line, relative_time, commit.commit.author.name
            ));
        }

        if i < logs.len() - 1 {
            output.push('\n');
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Author, Commit, CommitDetail};
    use chrono::TimeZone;

    #[test]
    fn rendering_commit_logs_works() {
        // GIVEN
        let reference = Utc.with_ymd_and_hms(2025, 1, 16, 12, 0, 0).unwrap();

        let log1 = CommitLog {
            app: "app-one".to_string(),
            from_env: "prod".to_string(),
            to_env: "dev".to_string(),
            from_version: "1.0.0".to_string(),
            to_version: "1.1.0".to_string(),
            commits: vec![Commit {
                sha: "abc1234567890".to_string(),
                commit: CommitDetail {
                    message: "First commit".to_string(),
                    author: Author {
                        name: "User A".to_string(),
                        email: "12345+user-a@users.noreply.github.com".to_string(),
                        date: Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).unwrap(),
                    },
                },
            }],
        };

        let log2 = CommitLog {
            app: "app-two".to_string(),
            from_env: "prod".to_string(),
            to_env: "dev".to_string(),
            from_version: "2.0.0".to_string(),
            to_version: "2.1.0".to_string(),
            commits: vec![
                Commit {
                    sha: "xyz9876543210".to_string(),
                    commit: CommitDetail {
                        message: "Second commit".to_string(),
                        author: Author {
                            name: "User A".to_string(),
                            email: "12345+user-a@users.noreply.github.com".to_string(),
                            date: Utc.with_ymd_and_hms(2025, 1, 16, 11, 0, 0).unwrap(),
                        },
                    },
                },
                Commit {
                    sha: "abc1234567890".to_string(),
                    commit: CommitDetail {
                        message: "First commit".to_string(),
                        author: Author {
                            name: "User B".to_string(),
                            email: "12346+user-b@users.noreply.github.com".to_string(),
                            date: Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).unwrap(),
                        },
                    },
                },
            ],
        };

        // WHEN
        let result = get_commit_logs(vec![log1, log2], reference);

        // THEN
        insta::assert_snapshot!(result, @r"
        app-one prod..dev (1.0.0..1.1.0)

         abc1234 - First commit (1d ago) <User A>

        app-two prod..dev (2.0.0..2.1.0)

         xyz9876 - Second commit (1h ago) <User A>
         abc1234 - First commit (1d ago) <User B>
        ");
    }
}
