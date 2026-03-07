# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Build & Test Commands

```sh
cargo build                          # Debug build
cargo build --release                # Release build (optimised for size)
cargo test                           # Run all tests
cargo test --test parser_tests       # Run a specific test file
cargo test test_round_trip           # Run tests matching a pattern
cargo test -- --nocapture            # Run with stdout visible
cargo clippy -- -D warnings          # Lint (CI enforces zero warnings)
cargo fmt --check                    # Check formatting
cargo fmt                            # Auto-format
```

## Architecture

HostsButler is a cross-platform TUI for managing `/etc/hosts` (or equivalent). It uses an Elm-style architecture: events → state mutation → pure render.

### Event Loop (main.rs)

```
crossterm events → AppEvent enum → App::handle_key() → mutate App state → ui::render::render(frame, &app)
```

Single-threaded, polling every 250ms. All state lives in the `App` struct. The `AppMode` enum drives a mode-based state machine (Normal, Search, AddEntry, EditEntry, ConfirmDelete, ConfirmSave, BackupManager, Help). UI components are pure render functions that read from `&App` without mutating it.

### Document Model (model/)

`HostsFile` stores `Vec<Line>`, not a flat entry list. The `Line` enum has four variants: `Blank`, `Comment`, `GroupHeader`, and `Entry(HostEntry)`. This preserves comments, blank lines, and group headers in their original positions — critical for round-trip fidelity.

### Round-Trip Fidelity

Loading a file and saving it without changes must produce byte-identical output. This works because:
- Each `HostEntry` stores its original `raw: String` and `separator: String`
- Unmodified entries serialize using their raw text verbatim
- Only modified entries (where `raw` is cleared) are re-serialized via `to_line_string()`
- The parser normalizes CRLF → LF; Windows platform code converts back on write
- `trailing_newline: bool` tracks whether the file ended with a newline

### Undo/Redo

Before every mutation, the entire `lines` vector is cloned onto `undo_stack`. Simple and correct for typical hosts files. Stacks are cleared on save/reload.

### Entry IDs

Sequential IDs assigned during parsing, not persisted to disk. Used for all operations (toggle, edit, delete) during a session. `HostsFile::next_id()` increments the counter.

### Platform Abstraction (platform/)

The `Platform` trait abstracts OS differences (hosts path, config dir, privilege escalation, DNS flush, line endings). Platform modules are conditionally compiled via `#[cfg(target_os)]` in `platform/mod.rs`. `detect_platform()` returns `Box<dyn Platform>` — called once at startup, no `#[cfg]` blocks elsewhere.

macOS/Linux use `unsafe extern "C" { pub safe fn geteuid() -> u32; }` (Rust 2024 edition pattern) for the root check — the `pub safe fn` means call sites don't need `unsafe` blocks.

### Commands (commands/)

`entry_cmds`, `file_cmds`, `backup_cmds` — thin wrappers that validate inputs then delegate to `HostsFile` methods. All return `anyhow::Result<T>`.

### Validation (validation/)

`validate_ip()` delegates to `std::net::IpAddr::parse()`. `validate_hostname()` enforces RFC 1123 (max 253 chars, labels max 63, alphanumeric + hyphens, no leading/trailing hyphens). Called in entry commands before mutation.

### Search

`HostEntry::matches_search()` supports prefix filters: `ip:`, `host:`, `group:`, or searches all fields. The `App` maintains `filtered_entry_ids: Vec<usize>` rebuilt on each keystroke.

### Disabled Entry Heuristic

The parser distinguishes disabled entries from comments by stripping the leading `#` and attempting to parse as an IP. If valid: disabled entry (preserving original comment prefix `"# "` vs `"#"`). If not: plain comment.

## Testing

- **Inline unit tests** in `src/parser/reader.rs`, `src/parser/writer.rs`, `src/validation/mod.rs`
- **Fixture-based tests** in `tests/parser_tests.rs` using files from `tests/fixtures/`
- **Integration tests** in `tests/integration_tests.rs` — full CRUD, toggle, undo/redo, search, import/export
- **Backup tests** in `tests/backup_tests.rs` — create, restore, delete, rotation

Fixture files must use LF line endings (enforced by `.gitattributes`). Round-trip tests also normalize CRLF as a safeguard.

## Error Handling

- `thiserror` for structured error enums (`PlatformError`, `ValidationError`)
- `anyhow::Result` for command functions and main
- `color_eyre` for enhanced panic reports (installed with `.ok()` to avoid conflict with anyhow)

## CI

Single workflow (`.github/workflows/ci.yml`) with three chained jobs:
1. **Check** — fmt, clippy, build, test on ubuntu/macos/windows (runs on push to main + PRs)
2. **Build** — release binaries for 5 targets (runs only on `v*` tags, after check passes)
3. **Release** — collects artifacts, creates GitHub Release (runs only on `v*` tags, after build passes)

## Rust Edition

Uses Rust 2024 edition. Key implication: `extern "C"` blocks require `unsafe` keyword, but individual functions can be marked `pub safe fn` to make them callable without `unsafe` at call sites.
