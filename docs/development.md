# Development Guide

Guide for contributors and developers working on HostsButler.

## Prerequisites

- Rust 1.85+ (2024 edition)
- Git

## Building

```sh
# Debug build
cargo build

# Release build (optimised for size)
cargo build --release
```

The release profile is configured for small binary size:

```toml
[profile.release]
opt-level = "z"    # Optimise for size
lto = true         # Link-time optimisation
codegen-units = 1  # Single codegen unit for better optimisation
strip = true       # Strip debug symbols
panic = "abort"    # Abort on panic (smaller binary)
```

## Running Tests

```sh
# Run all tests
cargo test

# Run a specific test file
cargo test --test parser_tests
cargo test --test validation_tests
cargo test --test integration_tests
cargo test --test backup_tests

# Run tests matching a pattern
cargo test test_round_trip

# Run with output
cargo test -- --nocapture
```

### Test Organisation

| File | Coverage |
|------|----------|
| `src/parser/reader.rs` (inline) | Parser unit tests: single entries, disabled entries, comments, groups, IPv6, inline comments, blank lines, round-trip |
| `src/parser/writer.rs` (inline) | Serialisation round-trip, CRLF conversion, trailing newline preservation |
| `src/validation/mod.rs` (inline) | IP and hostname validation edge cases |
| `tests/parser_tests.rs` | Fixture-based parser tests: simple, complex, malformed, round-trip fidelity |
| `tests/validation_tests.rs` | Comprehensive IP/hostname validation: IPv4, IPv6, private ranges, RFC 1123 rules |
| `tests/integration_tests.rs` | Full CRUD cycles, toggle, undo/redo, duplicate detection, search, CLI import/export, validation rejection |
| `tests/backup_tests.rs` | Backup create, list, restore, delete, ordering, rotation to the latest 20 |

### Test Fixtures

Located in `tests/fixtures/`:

| Fixture | Purpose |
|---------|---------|
| `simple.hosts` | Basic entries: localhost, IPv6, broadcast |
| `complex.hosts` | Groups, comments, disabled entries, multiple hostnames, inline comments, mixed IPv4/v6 |
| `windows.hosts` | Windows-style hosts file with different formatting |
| `malformed.hosts` | Edge cases: invalid lines, missing fields, unusual spacing |

## Linting

```sh
# Run clippy with warnings as errors
cargo clippy -- -D warnings

# Check formatting
cargo fmt --check

# Auto-format
cargo fmt
```

## Project Structure

```
src/
  main.rs                  CLI entry point and event loop
  lib.rs                   Library module re-exports
  app.rs                   Application state machine (AppMode, App struct)
  event.rs                 Crossterm event handling (AppEvent, EventHandler)
  tui.rs                   Terminal init/restore (raw mode, alternate screen)

  model/
    mod.rs                 Module exports
    entry.rs               HostEntry, EntryStatus
    group.rs               HostGroup (name + count)
    hosts_file.rs          HostsFile document model (CRUD, undo/redo, groups)
    line.rs                Line enum (Blank, Comment, GroupHeader, Entry)

  parser/
    mod.rs                 Module exports
    reader.rs              Line-by-line parser with group context tracking
    writer.rs              Serialiser (round-trip safe, CRLF support)

  platform/
    mod.rs                 Platform trait + detect_platform()
    macos.rs               macOS: paths, sudo, DNS flush
    linux.rs               Linux: paths, pkexec/sudo, systemd-resolved
    windows.rs             Windows: paths, is_elevated, ipconfig

  backup/
    mod.rs                 BackupManager (create, list, restore, delete, rotate)
    store.rs               BackupMetadata (serde struct)

  dns/
    mod.rs                 DNS resolution testing (test_resolution, test_entry_resolution)

  validation/
    mod.rs                 IP validation, hostname RFC 1123 validation

  commands/
    mod.rs                 Module exports
    entry_cmds.rs          Add, toggle, update, delete entries (with validation)
    file_cmds.rs           Import/export, shared load/save helpers, DNS flush warnings
    backup_cmds.rs         Create, list, restore, delete backups

  ui/
    mod.rs                 Module exports
    render.rs              Top-level render dispatch
    layout.rs              AppLayout (title bar, groups, table, status bar)
    theme.rs               Colour palette and style constants
    components/
      mod.rs               Module exports
      table_view.rs        Main entries table
      group_panel.rs       Left sidebar group list
      search_bar.rs        Search input display
      status_bar.rs        Bottom bar (shortcuts + mode indicator)
      entry_dialog.rs      Add/edit entry modal form
      confirm_dialog.rs    Delete/save confirmation dialog
      backup_view.rs       Backup manager overlay
      help_overlay.rs      Keyboard shortcut reference
      toast.rs             Transient notification messages
```

## Key Design Decisions

### Custom Parser

Existing hosts file parsing crates don't preserve comments, blank lines, and formatting. HostsButler uses a custom parser to guarantee round-trip fidelity: loading a file and saving it without changes produces byte-identical output.

### Document Model vs Entry List

The `HostsFile` stores a `Vec<Line>` rather than a flat `Vec<HostEntry>`. This preserves the document structure (comments, blank lines, group headers) and their positions relative to entries.

### Undo via Cloning

The undo system clones the entire `lines` vector before each mutation. This is simple and correct. For typical hosts files (tens to hundreds of lines), the memory cost is negligible.

### Session-Unique IDs

Entry IDs are assigned sequentially during parsing and are not persisted. They exist only to identify entries during a session for operations like toggle, edit, and delete.

### Platform Trait

The `Platform` trait abstracts OS differences behind a single interface. Platform detection happens once at startup via `cfg` attributes. This avoids `#[cfg]` blocks throughout the codebase. Save and CLI import paths use the trait for hosts-file writes and best-effort DNS cache flushing after successful system-hosts writes.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI framework |
| `crossterm` | Cross-platform terminal input/output |
| `serde` / `serde_json` | JSON serialisation for backup metadata and export |
| `csv` | CSV export |
| `dns-lookup` | DNS resolution via `getaddrinfo` |
| `dirs` | Platform-specific config/data directories |
| `tempfile` | Temporary files for safe write operations |
| `sha2` | SHA-256 checksums for backup integrity |
| `chrono` | Timestamps for backup metadata |
| `thiserror` | Structured error types |
| `anyhow` | Ergonomic error propagation |
| `color-eyre` | Enhanced panic reports |
| `clap` | CLI argument parsing |
| `regex` | Group header pattern matching |
| `tracing` / `tracing-subscriber` | Structured logging |
| `unicode-width` | Unicode-aware text width calculation |

### Windows-Only Dependencies

| Crate | Purpose |
|-------|---------|
| `is_elevated` | Check for Administrator privileges |
| `winapi` | Windows API bindings for UAC |

### Dev Dependencies

| Crate | Purpose |
|-------|---------|
| `pretty_assertions` | Readable test assertion diffs |
| `assert_fs` | Filesystem assertion helpers |
| `predicates` | Test predicate combinators |

## Adding a New Feature

### Adding a New Keyboard Shortcut

1. Add the key binding in `App::handle_normal_key()` (or the relevant mode handler) in `src/app.rs`
2. Add the shortcut to the status bar in `src/ui/components/status_bar.rs`
3. Add it to the help overlay in `src/ui/components/help_overlay.rs`
4. Document it in `docs/user-guide.md` and `README.md`

### Adding a New UI Component

1. Create a new file in `src/ui/components/`
2. Add a `pub fn render(f: &mut Frame, app: &App, area: Rect)` function
3. Register the module in `src/ui/components/mod.rs`
4. Call the render function from `src/ui/render.rs`

### Adding a New Platform

1. Create a new file in `src/platform/` (e.g., `freebsd.rs`)
2. Implement the `Platform` trait
3. Add a `cfg` branch in `src/platform/mod.rs` for platform detection
4. Add the platform to the CI matrix in `.github/workflows/ci.yml`

## CI Pipeline

The GitHub Actions CI workflow runs on every push to `main` and on pull requests:

1. **Formatting check**: `cargo fmt --check`
2. **Linting**: `cargo clippy -- -D warnings`
3. **Build**: `cargo build --release`
4. **Tests**: `cargo test`

This runs across `ubuntu-latest`, `macos-latest`, and `windows-latest`.
