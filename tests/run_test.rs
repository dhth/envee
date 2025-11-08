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
      -V, --versions <PATH>  Path to the versions file [default: versions.toml]
      -C, --no-commit-logs   Show commits between tags corresponding to different environments (requires ENVEE_GH_TOKEN to be set)
          --debug            Output debug information without doing anything
      -p, --plain            Whether to use output text to stdout without color
          --validate-only    Only validate versions file
      -h, --help             Print help

    ----- stderr -----
    ");
}

#[test]
fn debug_flag_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--debug",
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
    plain output:                         false
    only validate versions file:          false

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
    app     dev    prod   in-sync 
    repo-a  0.1.0  0.1.0  YES     
    repo-b  1.2.0  1.0.0  NO      
    repo-c  2.0.0  1.9.0  NO      

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
fn fails_if_provided_with_invalid_versions_file() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--no-commit-logs",
        "--plain",
        "--versions",
        "tests/assets/invalid-versions.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't parse file "tests/assets/invalid-versions.toml"

    Caused by:
        TOML parse error at line 1, column 8
          |
        1 | envs = "dev"
          |        ^^^^^
        invalid type: string "dev", expected a sequence
    "#);
}

#[test]
fn validating_invalid_versions_file_fails() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "run",
        "--validate-only",
        "--versions",
        "tests/assets/invalid-versions.toml",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't parse file "tests/assets/invalid-versions.toml"

    Caused by:
        TOML parse error at line 1, column 8
          |
        1 | envs = "dev"
          |        ^^^^^
        invalid type: string "dev", expected a sequence
    "#);
}
