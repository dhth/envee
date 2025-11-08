use crate::domain::{App, AppResult, AppVersion, DiffResult, Env, Version};
use std::collections::{HashMap, HashSet};

pub fn get_diff_result(envs: Vec<Env>, versions: &Vec<AppVersion>) -> DiffResult {
    let mut rows = Vec::new();
    let mut app_to_versions: HashMap<App, HashMap<Env, Version>> = HashMap::new();
    for version in versions {
        if !envs.contains(&version.env) {
            continue;
        }

        app_to_versions
            .entry(version.app.clone())
            .or_default()
            .insert(version.env.clone(), version.version.clone());
    }

    for (app, env_to_version) in app_to_versions {
        let in_sync = envs
            .iter()
            .filter_map(|env| env_to_version.get(env))
            .collect::<HashSet<_>>()
            .len()
            == 1;

        rows.push(AppResult {
            app,
            values: env_to_version,
            in_sync,
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
        let envs = vec!["dev".to_string(), "prod".to_string()];
        let versions = vec![
            AppVersion {
                app: "app1".to_string(),
                env: "dev".to_string(),
                version: "1.0.0".to_string(),
            },
            AppVersion {
                app: "app1".to_string(),
                env: "prod".to_string(),
                version: "1.0.0".to_string(),
            },
            AppVersion {
                app: "app2".to_string(),
                env: "dev".to_string(),
                version: "2.0.0".to_string(),
            },
            AppVersion {
                app: "app2".to_string(),
                env: "prod".to_string(),
                version: "1.9.0".to_string(),
            },
            AppVersion {
                app: "app3".to_string(),
                env: "dev".to_string(),
                version: "0.1.0".to_string(),
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
                in_sync: true
              - app: app2
                values:
                  dev: 2.0.0
                  prod: 1.9.0
                in_sync: false
              - app: app3
                values:
                  dev: 0.1.0
                in_sync: true
            ");
        });
    }

    #[test]
    fn envs_not_specified_are_not_considered() {
        // GIVEN
        let envs = vec!["dev".to_string(), "prod".to_string()];
        let versions = vec![
            AppVersion {
                app: "app1".to_string(),
                env: "dev".to_string(),
                version: "1.0.0".to_string(),
            },
            AppVersion {
                app: "app1".to_string(),
                env: "prod".to_string(),
                version: "1.0.0".to_string(),
            },
            AppVersion {
                app: "app1".to_string(),
                env: "other".to_string(),
                version: "2.0.0".to_string(),
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
                in_sync: true
            ");
        });
    }
}
