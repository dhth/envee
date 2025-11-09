use derive_more::{Deref, Display};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct RawVersions {
    pub envs: Vec<String>,
    pub github_org: String,
    pub versions: Vec<RawAppVersion>,
    pub git_tag_transform: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(crate) struct RawAppVersion {
    pub app: String,
    pub env: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct App(String);

impl TryFrom<String> for App {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            Err("app is empty")
        } else {
            Ok(Self(trimmed))
        }
    }
}

#[cfg(test)]
impl From<&str> for App {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Env(String);

impl TryFrom<String> for Env {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            Err("env is empty")
        } else {
            Ok(Self(trimmed))
        }
    }
}

#[cfg(test)]
impl From<&str> for Env {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Version(String);

impl TryFrom<String> for Version {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            Err("version is empty")
        } else {
            Ok(Self(trimmed))
        }
    }
}

#[cfg(test)]
impl From<&str> for Version {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, Deref, Display)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct GithubOrg(String);

impl TryFrom<String> for GithubOrg {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            Err("github_org is empty")
        } else {
            Ok(Self(trimmed))
        }
    }
}

#[derive(Debug, Clone, Deref, Display)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct GitTagTransform(String);

impl TryFrom<String> for GitTagTransform {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim().to_string();
        if !trimmed.contains("{{version}}") {
            Err("git_tag_transform doesn't include the placeholder \"{{version}}\"")
        } else {
            Ok(Self(trimmed))
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Versions {
    pub envs: Vec<Env>,
    pub github_org: GithubOrg,
    pub versions: Vec<AppVersion>,
    pub git_tag_transform: Option<GitTagTransform>,
}

#[derive(Debug)]
pub struct VersionsValidationErrors {
    top_level_errors: Vec<String>,
    version_errors: Vec<(usize, AppVersionValidationError)>,
}

impl VersionsValidationErrors {
    pub fn new() -> Self {
        Self {
            top_level_errors: Vec::new(),
            version_errors: Vec::new(),
        }
    }

    fn add_top_level_error(&mut self, message: impl Into<String>) {
        self.top_level_errors.push(message.into());
    }

    fn add_version_error(&mut self, version_index: usize, error: AppVersionValidationError) {
        self.version_errors.push((version_index, error));
    }

    fn is_empty(&self) -> bool {
        self.top_level_errors.is_empty() && self.version_errors.is_empty()
    }
}

impl std::fmt::Display for VersionsValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "versions config has errors:")?;

        for error in &self.top_level_errors {
            writeln!(f, " - {}", error)?;
        }

        for (index, error) in &self.version_errors {
            writeln!(f, " - version #{} has errors:", index)?;
            write!(f, "{}", error)?;
        }

        Ok(())
    }
}

impl std::error::Error for VersionsValidationErrors {}

impl TryFrom<RawVersions> for Versions {
    type Error = VersionsValidationErrors;

    fn try_from(raw: RawVersions) -> Result<Self, Self::Error> {
        let mut errors = VersionsValidationErrors::new();

        let mut version_envs = HashSet::new();
        let mut versions = Vec::new();
        for (i, raw_version) in raw.versions.into_iter().enumerate() {
            match AppVersion::try_from(raw_version) {
                Ok(app_version) => {
                    version_envs.insert(app_version.env.clone());
                    versions.push(app_version);
                }
                Err(e) => errors.add_version_error(i, e),
            }
        }

        if raw.envs.len() < 2 {
            errors.add_top_level_error(format!(
                "envs array has only {} element{}, need at least 2",
                raw.envs.len(),
                if raw.envs.len() == 1 { "" } else { "s" }
            ));
        }

        let mut envs = Vec::new();
        for (i, env_str) in raw.envs.into_iter().enumerate() {
            match Env::try_from(env_str) {
                Ok(env) => envs.push(env),
                Err(e) => {
                    errors.add_top_level_error(format!("envs[{}]: {}", i, e));
                }
            }
        }

        for env in &envs {
            if !version_envs.contains(env) {
                errors.add_top_level_error(format!(
                    r#"env "{env}" is not present in any of the versions configured"#
                ));
            }
        }

        let maybe_github_org = match GithubOrg::try_from(raw.github_org) {
            Ok(org) => Some(org),
            Err(e) => {
                errors.add_top_level_error(e);
                None
            }
        };

        let git_tag_transform = match raw.git_tag_transform {
            Some(transform) => match GitTagTransform::try_from(transform) {
                Ok(t) => Some(t),
                Err(e) => {
                    errors.add_top_level_error(e);
                    None
                }
            },
            None => None,
        };

        match maybe_github_org {
            Some(github_org) if errors.is_empty() => Ok(Self {
                envs,
                github_org,
                versions,
                git_tag_transform,
            }),
            _ => Err(errors),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct AppVersion {
    pub app: App,
    pub env: Env,
    pub version: Version,
}

#[derive(Debug)]
pub struct AppVersionValidationError {
    errors: Vec<String>,
}

impl AppVersionValidationError {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }

    fn add_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }
}

impl std::fmt::Display for AppVersionValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.errors {
            writeln!(f, "   - {}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for AppVersionValidationError {}

impl TryFrom<RawAppVersion> for AppVersion {
    type Error = AppVersionValidationError;

    fn try_from(raw: RawAppVersion) -> Result<Self, Self::Error> {
        let mut errors = AppVersionValidationError::new();

        let app = match App::try_from(raw.app) {
            Ok(a) => Some(a),
            Err(e) => {
                errors.add_error(e);
                None
            }
        };

        let env = match Env::try_from(raw.env) {
            Ok(e) => Some(e),
            Err(e) => {
                errors.add_error(e);
                None
            }
        };

        let version = match Version::try_from(raw.version) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.add_error(e);
                None
            }
        };

        match (app, env, version) {
            (Some(app), Some(env), Some(version)) => Ok(Self { app, env, version }),
            _ => Err(errors),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct DiffResult {
    pub envs: Vec<Env>,
    pub app_results: Vec<AppResult>,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct AppResult {
    pub app: App,
    pub values: HashMap<Env, Version>,
    pub in_sync: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_snapshot, assert_yaml_snapshot};

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn parsing_versions_works() {
        // GIVEN
        let raw = RawVersions {
            envs: vec!["dev".to_string(), "prod".to_string()],
            github_org: "my-org".to_string(),
            git_tag_transform: Some("v{{version}}".to_string()),
            versions: vec![
                RawAppVersion {
                    app: "app-a".to_string(),
                    env: "dev".to_string(),
                    version: "1.0.0".to_string(),
                },
                RawAppVersion {
                    app: "app-a".to_string(),
                    env: "prod".to_string(),
                    version: "1.0.0".to_string(),
                },
            ],
        };

        // WHEN
        let versions = Versions::try_from(raw).expect("result should've been Ok");

        // THEN
        assert_yaml_snapshot!(versions, @r#"
        envs:
          - dev
          - prod
        github_org: my-org
        versions:
          - app: app-a
            env: dev
            version: 1.0.0
          - app: app-a
            env: prod
            version: 1.0.0
        git_tag_transform: "v{{version}}"
        "#);
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn parsing_invalid_versions_fails() {
        // GIVEN
        let raw = RawVersions {
            envs: vec!["unknown".to_string()],
            github_org: "".to_string(),
            git_tag_transform: Some("no-placeholder".to_string()),
            versions: vec![
                RawAppVersion {
                    app: "".to_string(),
                    env: "".to_string(),
                    version: "".to_string(),
                },
                RawAppVersion {
                    app: "valid-app".to_string(),
                    env: "dev".to_string(),
                    version: "1.0.0".to_string(),
                },
                RawAppVersion {
                    app: "".to_string(),
                    env: "prod".to_string(),
                    version: "".to_string(),
                },
            ],
        };

        // WHEN
        let error = Versions::try_from(raw).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(error.to_string(), @r#"
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
    fn parsing_values_with_whitespace_only_fails() {
        // GIVEN
        let empty = "  ".to_string();
        let raw = RawVersions {
            envs: vec![empty.clone(), empty.clone()],
            github_org: empty.clone(),
            git_tag_transform: Some(empty.clone()),
            versions: vec![
                RawAppVersion {
                    app: empty.clone(),
                    env: empty.clone(),
                    version: empty.clone(),
                },
                RawAppVersion {
                    app: "valid-app".to_string(),
                    env: "dev".to_string(),
                    version: "1.0.0".to_string(),
                },
            ],
        };

        // WHEN
        let error = Versions::try_from(raw).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(error.to_string(), @r#"
        versions config has errors:
         - envs[0]: env is empty
         - envs[1]: env is empty
         - github_org is empty
         - git_tag_transform doesn't include the placeholder "{{version}}"
         - version #0 has errors:
           - app is empty
           - env is empty
           - version is empty
        "#);
    }
}
