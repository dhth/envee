# envee

`envee` compares application versions across environments from a TOML file,
optionally fetches commit logs from GitHub, and renders stdout or HTML reports.

This project uses `mise` for tool management and tasks. Always use `mise` to
execute project commands; see `mise.toml` for the available tasks. `.env` is
loaded when present; never expose or commit its secrets.

```text
versions TOML → validated domain types → version diff → optional GitHub logs → report
```

- `args` defines the CLI; `main` orchestrates input, services, and output.
- `config` holds output settings; `versions` reads, parses, and filters TOML
    input.
- `domain` owns validated types, input validation, and diff/commit-log models.
- `service/diff` computes version differences; `service/github` handles GitHub
  requests and tag transformations.
- `view` renders reports. The built-in Tera template lives at
  `src/view/assets/template.html`; custom templates use the same context.

Keep diff computation and rendering independent of filesystem, network, and
environment access. Pass time explicitly to rendering functions. Preserve the
validated domain types rather than passing unchecked strings into services. Use
idiomatic Rust and contextual errors; Clippy denies `unwrap` and `expect`
outside tests.

Preserve configured environment order and alphabetical application ordering.
Commit comparisons run from the last configured environment to the first;
versions are strings, not necessarily semantic versions. `--no-commit-logs` and
`--validate-only` must work without a GitHub token or network access. Commit-log
fetch failures should still allow a partial report before returning an error.

When adding tests, follow the existing inline `insta` snapshots and `insta-cmd`
CLI tests, using `tests/common` and `tests/assets` for fixtures. Keep tests
independent of live GitHub access. When snapshots need updating, always use
`mise run update-snapshots`, then inspect the diff to confirm the changes are
intentional. `mise run review-snapshots` is interactive and for humans only;
agents must not run it.
