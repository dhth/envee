use crate::domain::{RawVersions, Versions};
use anyhow::Context;
use regex::Regex;
use std::path::Path;

pub fn get_from_file<P>(path: P, app_filter: Option<&Regex>) -> anyhow::Result<Versions>
where
    P: AsRef<Path>,
{
    let contents = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "couldn't read file \"{}\"",
            &path.as_ref().to_string_lossy()
        )
    })?;

    let versions = get_versions(&contents, app_filter).with_context(|| {
        format!(
            "couldn't get versions from file \"{}\"",
            &path.as_ref().to_string_lossy()
        )
    })?;

    Ok(versions)
}

pub fn get_versions<S>(contents: S, app_filter: Option<&Regex>) -> anyhow::Result<Versions>
where
    S: AsRef<str>,
{
    let mut raw: RawVersions = toml::from_str(contents.as_ref())?;

    if let Some(regex) = app_filter {
        raw.versions.retain(|v| regex.is_match(&v.app));

        if raw.versions.is_empty() {
            anyhow::bail!("no versions match the provided filter");
        }
    }

    let versions: Versions = raw.try_into()?;

    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_TOML: &str = r#"
envs = ["dev", "prod"]
github_org = "dhth"
git_tag_transform = "v{{version}}"

[[versions]]
app = "repo-a"
env = "prod"
version = "0.1.0"

[[versions]]
app = "repo-b"
env = "prod"
version = "1.0.0"

[[versions]]
app = "repo-c"
env = "prod"
version = "1.9.0"

[[versions]]
app = "repo-a"
env = "dev"
version = "0.1.0"

[[versions]]
app = "repo-b"
env = "dev"
version = "1.2.0"

[[versions]]
app = "repo-c"
env = "dev"
version = "2.0.0"
"#;

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn parsing_valid_versions_config_works() {
        // GIVEN

        // WHEN
        let versions = get_versions(VALID_TOML, None).expect("result should've been Ok");

        // THEN
        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings.bind(|| {
            insta::assert_yaml_snapshot!(versions, @r#"
            envs:
              - dev
              - prod
            github_org: dhth
            versions:
              - app: repo-a
                env: prod
                version: 0.1.0
              - app: repo-b
                env: prod
                version: 1.0.0
              - app: repo-c
                env: prod
                version: 1.9.0
              - app: repo-a
                env: dev
                version: 0.1.0
              - app: repo-b
                env: dev
                version: 1.2.0
              - app: repo-c
                env: dev
                version: 2.0.0
            git_tag_transform: "v{{version}}"
            "#);
        });
    }

    #[test]
    fn parsing_with_filter_matching_some_apps_works() {
        // GIVEN
        let filter = Regex::new("repo-[ab]").unwrap();

        // WHEN
        let versions = get_versions(VALID_TOML, Some(&filter)).expect("result should've been Ok");

        // THEN
        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings.bind(|| {
            insta::assert_yaml_snapshot!(versions, @r#"
            envs:
              - dev
              - prod
            github_org: dhth
            versions:
              - app: repo-a
                env: prod
                version: 0.1.0
              - app: repo-b
                env: prod
                version: 1.0.0
              - app: repo-a
                env: dev
                version: 0.1.0
              - app: repo-b
                env: dev
                version: 1.2.0
            git_tag_transform: "v{{version}}"
            "#);
        });
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn parsing_invalid_versions_toml_config_fails() {
        // GIVEN
        let contents = "invalid toml";

        // WHEN
        let error = get_versions(contents, None).expect_err("result should've been an error");

        // THEN
        insta::assert_snapshot!(error.to_string(), @r"
        TOML parse error at line 1, column 9
          |
        1 | invalid toml
          |         ^
        key with no value, expected `=`
        ");
    }

    #[test]
    fn parsing_invalid_versions_config_fails() {
        // GIVEN
        let contents = r#"
envs = "not an array"
"#;

        // WHEN
        let error = get_versions(contents, None).expect_err("result should've been an error");

        // THEN

        insta::assert_snapshot!(error.to_string(), @r#"
        TOML parse error at line 2, column 8
          |
        2 | envs = "not an array"
          |        ^^^^^^^^^^^^^^
        invalid type: string "not an array", expected a sequence
        "#);
    }

    #[test]
    fn parsing_config_with_no_versions_fails() {
        // GIVEN
        let contents = r#"
envs = ["dev", "prod"]
github_org = "dhth"
"#;

        // WHEN
        let error = get_versions(contents, None).expect_err("result should've been an error");

        // THEN
        insta::assert_snapshot!(error.to_string(), @r"
        TOML parse error at line 1, column 1
          |
        1 | 
          | ^
        missing field `versions`
        ");
    }

    #[test]
    fn parsing_with_filter_matching_no_apps_fails() {
        // GIVEN
        let filter = Regex::new("^nonexistent").unwrap();

        // WHEN
        let error =
            get_versions(VALID_TOML, Some(&filter)).expect_err("result should've been an error");

        // THEN
        insta::assert_snapshot!(error.to_string(), @"no versions match the provided filter");
    }

    #[test]
    fn parsing_versions_with_invalid_data_fails() {
        // GIVEN
        let contents = r#"
envs = ["unknown"]
github_org = ""
git_tag_transform = "v{version}}"

[[versions]]
app = ""
env = ""
version = ""

[[versions]]
app = "valid-app"
env = "dev"
version = "1.0.0"

[[versions]]
app = ""
env = "prod"
version = ""
"#;

        // WHEN
        let error = get_versions(contents, None).expect_err("result should've been an error");

        // THEN
        insta::assert_snapshot!(error.to_string(), @r#"
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
}
