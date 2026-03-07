# HostsButler Agent Guide

HostsButler is a cross-platform Rust TUI for editing the system hosts file while preserving comments, layout, and platform-specific line endings.

## Quick Reference

- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt --check`
- Targeted test: `cargo test --test parser_tests` or `cargo test test_round_trip`

## Core Invariants

- Preserve round-trip fidelity: saving an unchanged file must remain byte-identical.
- Keep the hosts file as a document model (`Vec<Line>`), not a flat entry list.
- Restrict mutations to app handlers and command/model code; UI components should stay pure render functions.
- Keep OS-specific behavior behind `src/platform/`; avoid scattering `#[cfg]` checks elsewhere.
- Preserve LF internally and only convert back to CRLF at the Windows platform boundary.

## Detailed Guidance

- [Agent workflow](docs/agent-guide.md)
- [Architecture](docs/architecture.md)
- [Development guide](docs/development.md)
- [Hosts file format](docs/file-format.md)
- [Platform support](docs/platform-support.md)
