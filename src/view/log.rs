use super::date::get_humanized_date;
use crate::domain::CommitLog;
use chrono::{DateTime, Utc};
use colored::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const AUTHOR_COLOR_POOL: [Color; 6] = [
    Color::Blue,
    Color::BrightBlue,
    Color::BrightCyan,
    Color::BrightMagenta,
    Color::Cyan,
    Color::Magenta,
];

pub fn get_commit_logs(
    logs: Vec<CommitLog>,
    reference_time: DateTime<Utc>,
    plain_output: bool,
) -> String {
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

            let formatted_line = if plain_output {
                format!(
                    " {} - {} ({}) <{}>\n",
                    short_sha, first_line, relative_time, commit.commit.author.name
                )
            } else {
                let author_color = get_author_color(&commit.commit.author.name);
                format!(
                    " {} - {} ({}) <{}>\n",
                    short_sha.dimmed(),
                    first_line,
                    relative_time.yellow(),
                    commit.commit.author.name.color(author_color)
                )
            };

            output.push_str(&formatted_line);
        }

        if i < logs.len() - 1 {
            output.push('\n');
        }
    }

    output
}

fn get_author_color(author_name: &str) -> Color {
    let mut hasher = DefaultHasher::new();
    author_name.hash(&mut hasher);
    let hash = hasher.finish();

    let index = (hash % AUTHOR_COLOR_POOL.len() as u64) as usize;
    AUTHOR_COLOR_POOL[index]
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::domain::{Author, Commit, CommitDetail};
    use chrono::TimeZone;

    #[test]
    fn rendering_commit_logs_works() {
        // GIVEN
        let reference = Utc.with_ymd_and_hms(2025, 1, 16, 12, 0, 0).unwrap();

        let log1 = CommitLog {
            app: "app-one".into(),
            from_env: "prod".into(),
            to_env: "dev".into(),
            from_version: "1.0.0".into(),
            to_version: "1.1.0".into(),
            commits: vec![Commit {
                sha: "abc1234567890".to_string(),
                commit: CommitDetail {
                    message: "First commit".to_string(),
                    author: Author {
                        name: "User A".to_string(),
                        date: Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).unwrap(),
                    },
                },
            }],
        };

        let log2 = CommitLog {
            app: "app-two".into(),
            from_env: "prod".into(),
            to_env: "dev".into(),
            from_version: "2.0.0".into(),
            to_version: "2.1.0".into(),
            commits: vec![
                Commit {
                    sha: "xyz9876543210".to_string(),
                    commit: CommitDetail {
                        message: "Second commit".to_string(),
                        author: Author {
                            name: "User A".to_string(),
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
                            date: Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).unwrap(),
                        },
                    },
                },
            ],
        };

        // WHEN
        let result = get_commit_logs(vec![log1, log2], reference, true);

        // THEN
        insta::assert_snapshot!(result, @r"
        app-one prod..dev (1.0.0..1.1.0)

         abc1234 - First commit (1d ago) <User A>

        app-two prod..dev (2.0.0..2.1.0)

         xyz9876 - Second commit (1h ago) <User A>
         abc1234 - First commit (1d ago) <User B>
        ");
    }

    #[test]
    fn get_author_color_returns_consistent_color_for_same_author() {
        // GIVEN
        let author = "Alan Turing";

        // WHEN
        let mut set = HashSet::new();
        for _ in 1..=100 {
            set.insert(get_author_color(author).to_bg_str());
        }

        // THEN
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn get_author_color_returns_valid_color_from_pool() {
        // GIVEN
        let test_authors = [
            "Alan Turing",
            "Grace Hopper",
            "Donald Knuth",
            "Ada Lovelace",
            "Dennis Ritchie",
            "Ken Thompson",
            "Linus Torvalds",
            "",
        ];

        // WHEN
        // THEN
        for author in &test_authors {
            let color = get_author_color(author);
            assert!(
                AUTHOR_COLOR_POOL.contains(&color),
                "Color for author '{}' should be in the valid color pool",
                author
            );
        }
    }
}
