mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn shows_help() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["run", "--help"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Show results based on a versions file

    Usage: envee run [OPTIONS]

    Options:
      -V, --versions <PATH>       Path to the versions file [default: versions.toml]
      -C, --no-commit-logs        Show commits between tags corresponding to different environments (requires ENVEE_GH_TOKEN to be set)
          --debug                 Output debug information without doing anything
      -s, --table-style <STRING>  Output table style [default: utf8] [possible values: ascii, markdown, none, utf8]
      -p, --plain                 Whether to use output text to stdout without color
          --validate-only         Only validate versions file
      -f, --filter <REGEX>        Regex to use for filtering apps
      -h, --help                  Print help

    ----- stderr -----
    ");
}

#[test]
fn debug_flag_works_for_defaults() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["run", "--debug"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                              Run
    versions file:                        versions.toml
    don't show commit logs:               false
    table style:                          utf8
    plain output:                         false
    only validate versions file:          false
    app filter:                           <NOT PROVIDED>

    ----- stderr -----
    ");
}

#[test]
fn debug_flag_works_with_overridden_flags() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--debug",
        "--filter",
        "repo",
        "--plain",
        "--table-style",
        "ascii",
        "--versions",
        "tests/assets/valid-versions.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                              Run
    versions file:                        tests/assets/valid-versions.toml
    don't show commit logs:               false
    table style:                          ascii
    plain output:                         true
    only validate versions file:          false
    app filter:                           repo

    ----- stderr -----
    ");
}

#[test]
fn works_for_valid_versions_file() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--no-commit-logs",
        "--plain",
        "--versions",
        "tests/assets/valid-versions.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    ┌───────┬───────┬───────┬─────────┐
    │app    ┆ dev   ┆ prod  ┆ in-sync │
    ╞═══════╪═══════╪═══════╪═════════╡
    │repo-a ┆ 0.1.0 ┆ 0.1.0 ┆ YES     │
    │repo-b ┆ 1.2.0 ┆ 1.0.0 ┆ NO      │
    │repo-c ┆ 2.0.0 ┆ 1.9.0 ┆ NO      │
    └───────┴───────┴───────┴─────────┘

    ----- stderr -----
    ");
}

#[test]
fn validating_versions_file_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--validate-only",
        "--versions",
        "tests/assets/valid-versions.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    versions file is valid ✅

    ----- stderr -----
    ");
}

//-------------//
//  FAILURES   //
//-------------//

#[test]
fn fails_if_provided_with_absent_versions_file() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--no-commit-logs",
        "--plain",
        "--versions",
        "tests/assets/absent.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't read file "tests/assets/absent.toml"

    Caused by:
        No such file or directory (os error 2)
    "#);
}

#[test]
fn fails_if_provided_with_invalid_versions_schema() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--no-commit-logs",
        "--plain",
        "--versions",
        "tests/assets/invalid-schema.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't get versions from file "tests/assets/invalid-schema.toml"

    Caused by:
        TOML parse error at line 1, column 8
          |
        1 | envs = "dev"
          |        ^^^^^
        invalid type: string "dev", expected a sequence
    "#);
}

#[test]
fn fails_if_provided_with_invalid_versions_data() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--no-commit-logs",
        "--plain",
        "--versions",
        "tests/assets/invalid-data.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't get versions from file "tests/assets/invalid-data.toml"

    Caused by:
        versions config has errors:
         - envs array has only 1 element, need at least 2
         - env "unknown" is not present in any of the versions configured
         - github_org is empty
         - git_tag_transform doesn't include the placeholder "{{version}}"
         - version #0 has errors:
           - app is empty
           - env is empty
           - version is empty
         - version #2 has errors:
           - app is empty
           - version is empty
    "#);
}

#[test]
fn validating_invalid_versions_schema_fails() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--no-commit-logs",
        "--validate-only",
        "--versions",
        "tests/assets/invalid-schema.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't get versions from file "tests/assets/invalid-schema.toml"

    Caused by:
        TOML parse error at line 1, column 8
          |
        1 | envs = "dev"
          |        ^^^^^
        invalid type: string "dev", expected a sequence
    "#);
}

#[test]
fn fails_if_provided_invalid_regex() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--filter",
        "(invalid|",
        "--no-commit-logs",
        "--versions",
        "tests/assets/valid-versions.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: invalid regex pattern provided

    Caused by:
        regex parse error:
            (invalid|
            ^
        error: unclosed group
    ");
}

#[test]
fn fails_if_no_versions_defined() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--no-commit-logs",
        "--versions",
        "tests/assets/no-versions.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't get versions from file "tests/assets/no-versions.toml"

    Caused by:
        TOML parse error at line 1, column 1
          |
        1 | envs = ["dev", "prod"]
          | ^
        missing field `versions`
    "#);
}

#[test]
fn fails_if_no_versions_match_filter() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--no-commit-logs",
        "--filter",
        "absent",
        "--versions",
        "tests/assets/valid-versions.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't get versions from file "tests/assets/valid-versions.toml"

    Caused by:
        no versions match the provided filter
    "#);
}

#[test]
fn fails_if_no_gh_token_is_provided() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["run", "--versions", "tests/assets/valid-versions.toml"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: ENVEE_GH_TOKEN needs to be set to fetch commit logs from GitHub
    ");
}
