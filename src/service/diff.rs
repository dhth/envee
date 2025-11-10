use crate::domain::{App, AppResult, AppVersion, DiffResult, Env, SyncStatus, Version};
use std::collections::{HashMap, HashSet};

// Note: TryFrom<RawVersions> for Versions should ensure that every AppVersion.env is present in
// the envs Vec
pub fn get_diff_result(envs: Vec<Env>, versions: &Vec<AppVersion>) -> DiffResult {
    let mut rows = Vec::new();
    let mut app_data: HashMap<App, HashMap<Env, Version>> = HashMap::new();
    for version in versions {
        app_data
            .entry(version.app.clone())
            .or_default()
            .insert(version.env.clone(), version.version.clone());
    }

    for (app, env_to_version) in app_data {
        let sync_status = if env_to_version.len() == 1 {
            SyncStatus::NotApplicable
        } else {
            if envs
                .iter()
                .filter_map(|env| env_to_version.get(env))
                .collect::<HashSet<&Version>>()
                .len()
                == 1
            {
                SyncStatus::InSync
            } else {
                SyncStatus::OutOfSync
            }
        };

        rows.push(AppResult {
            app,
            values: env_to_version,
            sync_status,
        });
    }

    rows.sort_by(|a, b| a.app.cmp(&b.app));

    DiffResult {
        envs,
        app_results: rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AppVersion;

    #[test]
    fn diff_result_is_computed_correctly() {
        // GIVEN
        let envs = vec!["dev".into(), "prod".into()];
        let versions = vec![
            AppVersion {
                app: "app1".into(),
                env: "dev".into(),
                version: "1.0.0".into(),
            },
            AppVersion {
                app: "app1".into(),
                env: "prod".into(),
                version: "1.0.0".into(),
            },
            AppVersion {
                app: "app2".into(),
                env: "dev".into(),
                version: "2.0.0".into(),
            },
            AppVersion {
                app: "app2".into(),
                env: "prod".into(),
                version: "1.9.0".into(),
            },
            AppVersion {
                app: "app3".into(),
                env: "dev".into(),
                version: "0.1.0".into(),
            },
        ];

        // WHEN
        let result = get_diff_result(envs, &versions);

        // THEN
        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings.bind(|| {
            insta::assert_yaml_snapshot!(result, @r"
            envs:
              - dev
              - prod
            app_results:
              - app: app1
                values:
                  dev: 1.0.0
                  prod: 1.0.0
                sync_status: in_sync
              - app: app2
                values:
                  dev: 2.0.0
                  prod: 1.9.0
                sync_status: out_of_sync
              - app: app3
                values:
                  dev: 0.1.0
                sync_status: not_applicable
            ");
        });
    }

    #[test]
    fn envs_not_specified_are_not_considered() {
        // GIVEN
        let envs = vec!["dev".into(), "prod".into()];
        let versions = vec![
            AppVersion {
                app: "app1".into(),
                env: "dev".into(),
                version: "1.0.0".into(),
            },
            AppVersion {
                app: "app1".into(),
                env: "prod".into(),
                version: "1.0.0".into(),
            },
        ];

        // WHEN
        let result = get_diff_result(envs, &versions);

        // THEN
        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings.bind(|| {
            insta::assert_yaml_snapshot!(result, @r"
            envs:
              - dev
              - prod
            app_results:
              - app: app1
                values:
                  dev: 1.0.0
                  prod: 1.0.0
                sync_status: in_sync
            ");
        });
    }

    #[test]
    fn single_env_apps_get_not_applicable_status() {
        let envs = vec!["dev".into(), "prod".into(), "staging".into()];
        let versions = vec![
            AppVersion {
                app: "multi-env-in-sync".into(),
                env: "dev".into(),
                version: "1.0.0".into(),
            },
            AppVersion {
                app: "multi-env-in-sync".into(),
                env: "prod".into(),
                version: "1.0.0".into(),
            },
            AppVersion {
                app: "single-env-app".into(),
                env: "staging".into(),
                version: "2.0.0".into(),
            },
        ];

        let result = get_diff_result(envs, &versions);

        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings.bind(|| {
            insta::assert_yaml_snapshot!(result, @r"
            envs:
              - dev
              - prod
              - staging
            app_results:
              - app: multi-env-in-sync
                values:
                  dev: 1.0.0
                  prod: 1.0.0
                sync_status: in_sync
              - app: single-env-app
                values:
                  staging: 2.0.0
                sync_status: not_applicable
            ");
        });
    }
}
