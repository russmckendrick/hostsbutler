# Agent Workflow

Use this guide for task-specific expectations that do not belong in the root `AGENTS.md`.

## Change Boundaries

- Prefer extending existing modules over introducing new patterns for the same job.
- Keep platform logic in `src/platform/` and cross-platform behavior in shared modules.
- Reuse the existing document model and command wrappers before adding new state or mutation paths.

## Critical Touchpoints

### Parser and model changes

- Preserve `HostEntry.raw`, `HostEntry.separator`, and `trailing_newline` semantics.
- Treat comments, blank lines, group headers, and entries as first-class `Line` variants.
- Add or update round-trip coverage when parsing or serialization behavior changes.

### App, modes, and shortcuts

- Keep mode transitions in `src/app.rs` coherent with `AppMode`.
- If a shortcut changes, update `src/ui/components/status_bar.rs`, `src/ui/components/help_overlay.rs`, and user-facing docs.
- Rebuild any filtered or selected entry state after mutations that affect search or grouping.

### Platform behavior

- Route file access, privilege escalation, DNS flushes, config paths, and line ending differences through the `Platform` trait.
- Avoid adding direct OS checks outside `src/platform/mod.rs` unless there is no viable abstraction point.

### Backups and file operations

- Keep backup and import/export behavior in the command and backup modules rather than UI code.
- Cover restore, rotation, and serialization edge cases in `tests/backup_tests.rs` or integration tests when behavior changes.

## Verification

- Run targeted tests while iterating on parser, backup, or validation changes.
- Before handoff for code changes, prefer this sequence:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
- For documentation-only changes, tests are optional unless the task also changes behavior.

## Reference Map

- Architecture overview: [`docs/architecture.md`](architecture.md)
- Development commands and test inventory: [`docs/development.md`](development.md)
- Hosts file parsing and round-trip rules: [`docs/file-format.md`](file-format.md)
- Platform-specific behavior: [`docs/platform-support.md`](platform-support.md)
