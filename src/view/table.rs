use crate::config::{StdoutConfig, TablePreset};
use crate::domain::DiffResult;
use comfy_table::{Cell, Color, Table, presets};

pub fn render_results_table(result: DiffResult, config: &StdoutConfig) -> String {
    let mut table = Table::new();

    match config.table_preset {
        TablePreset::Nothing => table.load_preset(presets::NOTHING),
        TablePreset::AsciiFull => table.load_preset(presets::ASCII_FULL),
    };

    let mut header = vec!["app".to_string()];
    header.extend(result.envs.iter().cloned());
    header.push("in-sync".to_string());
    table.set_header(header);

    for row in result.app_results {
        let should_highlight = config.highlight_out_of_sync && !row.in_sync;

        if should_highlight {
            let mut cells = vec![Cell::new(&row.app).fg(Color::Red)];

            for env in &result.envs {
                let version = row.values.get(env).map(|v| v.as_str()).unwrap_or("");
                cells.push(Cell::new(version).fg(Color::Red));
            }

            let in_sync = if row.in_sync { "YES" } else { "NO" };
            cells.push(Cell::new(in_sync).fg(Color::Red));

            table.add_row(cells);
        } else {
            let mut cells = vec![row.app.clone()];

            for env in &result.envs {
                let version = row.values.get(env).map(|v| v.as_str()).unwrap_or("");
                cells.push(version.to_string());
            }

            let in_sync = if row.in_sync { "YES" } else { "NO" };
            cells.push(in_sync.to_string());

            table.add_row(cells);
        }
    }

    if let Some(column) = table.column_mut(0) {
        column.set_padding((0, 1));
    }

    table.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StdoutConfig;
    use crate::domain::{AppResult, DiffResult};
    use std::collections::HashMap;

    #[test]
    fn table_is_rendered_correctly_with_preset_nothing() {
        // GIVEN
        let result = create_test_diff_result();
        let config = StdoutConfig {
            table_preset: TablePreset::Nothing,
            highlight_out_of_sync: false,
        };

        // WHEN
        let output = render_results_table(result, &config);

        // THEN
        insta::assert_snapshot!(output, @r"
        app   qa     staging  prod   in-sync 
        app1  1.0.0  1.0.0    1.0.0  YES     
        app2  2.0.0  2.0.0    1.9.0  NO      
        app3  0.1.0  0.1.0           YES     
        app4  0.1.0                  YES
        ");
    }

    #[test]
    fn table_is_rendered_correctly_with_preset_asciifull() {
        // GIVEN
        let result = create_test_diff_result();
        let config = StdoutConfig {
            table_preset: TablePreset::AsciiFull,
            highlight_out_of_sync: false,
        };

        // WHEN
        let output = render_results_table(result, &config);

        // THEN
        insta::assert_snapshot!(output, @r"
        +-----+-------+---------+-------+---------+
        |app  | qa    | staging | prod  | in-sync |
        +=========================================+
        |app1 | 1.0.0 | 1.0.0   | 1.0.0 | YES     |
        |-----+-------+---------+-------+---------|
        |app2 | 2.0.0 | 2.0.0   | 1.9.0 | NO      |
        |-----+-------+---------+-------+---------|
        |app3 | 0.1.0 | 0.1.0   |       | YES     |
        |-----+-------+---------+-------+---------|
        |app4 | 0.1.0 |         |       | YES     |
        +-----+-------+---------+-------+---------+
        ");
    }

    fn create_test_diff_result() -> DiffResult {
        let mut app1_values = HashMap::new();
        app1_values.insert("qa".to_string(), "1.0.0".to_string());
        app1_values.insert("staging".to_string(), "1.0.0".to_string());
        app1_values.insert("prod".to_string(), "1.0.0".to_string());

        let mut app2_values = HashMap::new();
        app2_values.insert("qa".to_string(), "2.0.0".to_string());
        app2_values.insert("staging".to_string(), "2.0.0".to_string());
        app2_values.insert("prod".to_string(), "1.9.0".to_string());

        let mut app3_values = HashMap::new();
        app3_values.insert("qa".to_string(), "0.1.0".to_string());
        app3_values.insert("staging".to_string(), "0.1.0".to_string());

        let mut app4_values = HashMap::new();
        app4_values.insert("qa".to_string(), "0.1.0".to_string());

        DiffResult {
            envs: vec!["qa", "staging", "prod"]
                .into_iter()
                .map(String::from)
                .collect(),
            app_results: vec![
                AppResult {
                    app: "app1".to_string(),
                    values: app1_values,
                    in_sync: true,
                },
                AppResult {
                    app: "app2".to_string(),
                    values: app2_values,
                    in_sync: false,
                },
                AppResult {
                    app: "app3".to_string(),
                    values: app3_values,
                    in_sync: true,
                },
                AppResult {
                    app: "app4".to_string(),
                    values: app4_values,
                    in_sync: true,
                },
            ],
        }
    }
}
