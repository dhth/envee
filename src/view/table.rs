use crate::config::{StdoutConfig, TableStyle};
use crate::domain::DiffResult;
use comfy_table::{Cell, Color, Table, presets};

pub fn render_results_table(result: DiffResult, config: &StdoutConfig) -> String {
    let mut table = Table::new();

    match config.table_style {
        TableStyle::Ascii => table.load_preset(presets::ASCII_FULL_CONDENSED),
        TableStyle::Markdown => table.load_preset(presets::ASCII_MARKDOWN),
        TableStyle::None => table.load_preset(presets::NOTHING),
        TableStyle::Utf8 => table.load_preset(presets::UTF8_FULL_CONDENSED),
    };

    let mut header = vec!["app".to_string()];
    header.extend(result.envs.iter().map(|e| e.to_string()));
    header.push("in-sync".to_string());
    table.set_header(header);

    for row in result.app_results {
        let should_highlight = !config.plain_output && !row.in_sync;

        if should_highlight {
            let mut cells = vec![Cell::new(&row.app).fg(Color::Red)];

            for env in &result.envs {
                let version = row.values.get(env).map(|v| v.as_ref()).unwrap_or("");
                cells.push(Cell::new(version).fg(Color::Red));
            }

            let in_sync = if row.in_sync { "YES" } else { "NO" };
            cells.push(Cell::new(in_sync).fg(Color::Red));

            table.add_row(cells);
        } else {
            let mut cells = vec![row.app.to_string()];

            for env in &result.envs {
                let version = row.values.get(env).map(|v| v.as_ref()).unwrap_or("");
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
    fn table_is_rendered_correctly_with_style_ascii() {
        // GIVEN
        let result = create_test_diff_result();
        let config = StdoutConfig {
            table_style: TableStyle::Ascii,
            plain_output: true,
        };

        // WHEN
        let output = render_results_table(result, &config);

        // THEN
        insta::assert_snapshot!(output, @r"
        +-----+-------+---------+-------+---------+
        |app  | qa    | staging | prod  | in-sync |
        +=========================================+
        |app1 | 1.0.0 | 1.0.0   | 1.0.0 | YES     |
        |app2 | 2.0.0 | 2.0.0   | 1.9.0 | NO      |
        |app3 | 0.1.0 | 0.1.0   |       | YES     |
        |app4 | 0.1.0 |         |       | YES     |
        +-----+-------+---------+-------+---------+
        ");
    }

    #[test]
    fn table_is_rendered_correctly_with_style_markdown() {
        // GIVEN
        let result = create_test_diff_result();
        let config = StdoutConfig {
            table_style: TableStyle::Markdown,
            plain_output: true,
        };

        // WHEN
        let output = render_results_table(result, &config);

        // THEN
        insta::assert_snapshot!(output, @r"
        |app  | qa    | staging | prod  | in-sync |
        |-----|-------|---------|-------|---------|
        |app1 | 1.0.0 | 1.0.0   | 1.0.0 | YES     |
        |app2 | 2.0.0 | 2.0.0   | 1.9.0 | NO      |
        |app3 | 0.1.0 | 0.1.0   |       | YES     |
        |app4 | 0.1.0 |         |       | YES     |
        ");
    }

    #[test]
    fn table_is_rendered_correctly_with_style_none() {
        // GIVEN
        let result = create_test_diff_result();
        let config = StdoutConfig {
            table_style: TableStyle::None,
            plain_output: true,
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
    fn table_is_rendered_correctly_with_style_utf8() {
        // GIVEN
        let result = create_test_diff_result();
        let config = StdoutConfig {
            table_style: TableStyle::Utf8,
            plain_output: true,
        };

        // WHEN
        let output = render_results_table(result, &config);

        // THEN
        insta::assert_snapshot!(output, @r"
        ┌─────┬───────┬─────────┬───────┬─────────┐
        │app  ┆ qa    ┆ staging ┆ prod  ┆ in-sync │
        ╞═════╪═══════╪═════════╪═══════╪═════════╡
        │app1 ┆ 1.0.0 ┆ 1.0.0   ┆ 1.0.0 ┆ YES     │
        │app2 ┆ 2.0.0 ┆ 2.0.0   ┆ 1.9.0 ┆ NO      │
        │app3 ┆ 0.1.0 ┆ 0.1.0   ┆       ┆ YES     │
        │app4 ┆ 0.1.0 ┆         ┆       ┆ YES     │
        └─────┴───────┴─────────┴───────┴─────────┘
        ");
    }

    fn create_test_diff_result() -> DiffResult {
        let mut app1_values = HashMap::new();
        app1_values.insert("qa".into(), "1.0.0".into());
        app1_values.insert("staging".into(), "1.0.0".into());
        app1_values.insert("prod".into(), "1.0.0".into());

        let mut app2_values = HashMap::new();
        app2_values.insert("qa".into(), "2.0.0".into());
        app2_values.insert("staging".into(), "2.0.0".into());
        app2_values.insert("prod".into(), "1.9.0".into());

        let mut app3_values = HashMap::new();
        app3_values.insert("qa".into(), "0.1.0".into());
        app3_values.insert("staging".into(), "0.1.0".into());

        let mut app4_values = HashMap::new();
        app4_values.insert("qa".into(), "0.1.0".into());

        DiffResult {
            envs: vec!["qa", "staging", "prod"]
                .into_iter()
                .map(Into::into)
                .collect(),
            app_results: vec![
                AppResult {
                    app: "app1".into(),
                    values: app1_values,
                    in_sync: true,
                },
                AppResult {
                    app: "app2".into(),
                    values: app2_values,
                    in_sync: false,
                },
                AppResult {
                    app: "app3".into(),
                    values: app3_values,
                    in_sync: true,
                },
                AppResult {
                    app: "app4".into(),
                    values: app4_values,
                    in_sync: true,
                },
            ],
        }
    }
}
