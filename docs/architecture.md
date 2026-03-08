# Architecture

This document describes the internal architecture of HostsButler.

## Overview

HostsButler follows a modified Elm architecture for its TUI:

```
crossterm events -> AppEvent -> App::handle_key() -> state mutation -> App::render()
```

The application is structured as a single Cargo crate with library modules. The binary (`main.rs`) handles CLI parsing and the event loop, while all logic lives in `lib.rs` modules for testability.

## Module Dependency Graph

```
main.rs
  |
  +-- app.rs (state machine)
  |     |
  |     +-- commands/ (entry_cmds, file_cmds, backup_cmds)
  |     +-- model/ (HostsFile, HostEntry, Line, HostGroup)
  |     +-- backup/ (BackupManager)
  |     +-- dns/ (resolution testing)
  |     +-- platform/ (Platform trait, OS implementations)
  |     +-- parser/ (reader, writer)
  |
  +-- event.rs (crossterm event loop)
  +-- tui.rs (terminal setup/teardown)
  +-- ui/ (render dispatch, components, theme, layout)
        |
        +-- render.rs (top-level render)
        +-- layout.rs (screen partitioning)
        +-- theme.rs (colour palette)
        +-- components/ (table, groups, dialogs, status bar, toast)
```

## Data Flow

### Startup

1. `main.rs` parses CLI arguments with `clap`
2. Platform detection selects the correct `Platform` implementation
3. The hosts file is read from disk (platform-specific path or CLI override)
4. The parser produces a `HostsFile` document model
5. `App::new()` initialises the application state
6. The event loop begins

### Event Loop

The event loop runs at 250ms tick intervals:

1. `terminal.draw()` calls `ui::render::render()` which dispatches to components
2. `events.next()` blocks until a key event or tick timeout
3. Key events are routed through `App::handle_key()` based on the current `AppMode`
4. State mutations trigger UI re-render on the next loop iteration

### Save Flow

1. Auto-backup of the current hosts file content
2. Serialise the `HostsFile` to string (CRLF on Windows, LF elsewhere)
3. Platform-specific write with privilege escalation if needed
4. Best-effort DNS cache flush when writing the real system hosts file
5. Clear undo history and mark file as clean

## Document Model

The hosts file is treated as a structured document, not a flat list of entries. Every line is preserved:

```rust
enum Line {
    Blank(String),                                    // Empty lines
    Comment(String),                                  // # comment lines
    GroupHeader { raw: String, group_name: String },  // ## [GroupName]
    Entry(HostEntry),                                 // IP hostname(s)
}
```

This design ensures **round-trip fidelity**: loading a file and immediately saving it produces byte-identical output. Only modified entries are re-serialised.

### Entry Identity

Each `HostEntry` receives a session-unique `id: usize` assigned by the parser. This ID is used for all operations (toggle, edit, delete) and is never persisted to disk.

### Undo/Redo

Before every mutating operation, the entire `lines` vector is cloned onto the undo stack. Undo pops from the undo stack and pushes the current state onto the redo stack. The stacks are cleared on save or reload.

## Parser Design

The parser processes the file line-by-line with group context tracking:

1. **Blank line** - empty or whitespace-only
2. **Group header** - matches `## [Name]` or `# --- Name ---`
3. **Disabled entry** - comment where stripping `#` yields a valid IP + hostname line
4. **Pure comment** - any other `#`-prefixed line
5. **Active entry** - `IP hostname [hostname2...] [# comment]`
6. **Unparseable** - falls through to `Line::Comment` (parser never fails)

### Disabled Entry Detection

```
# 127.0.0.1 myhost.local    -> Disabled entry (valid IP after #)
# This is a note             -> Pure comment (no valid IP after #)
```

The parser strips the leading `#` and whitespace, then attempts to parse as an IP address. If successful, it's a disabled entry. The original comment prefix (`"# "` vs `"#"`) is preserved for faithful round-trip.

### Formatting Preservation

Each entry stores:
- `raw: String` - the original line text (used for unmodified entries)
- `separator: String` - the whitespace between IP and hostnames (tab, spaces, etc.)

When an entry is modified, `raw` is cleared to force re-serialisation using standard format.

## Platform Abstraction

The `Platform` trait provides an OS-independent interface:

```rust
pub trait Platform: Send + Sync {
    fn hosts_path(&self) -> PathBuf;
    fn config_dir(&self) -> PathBuf;
    fn can_write(&self) -> bool;
    fn write_hosts(&self, content: &str) -> Result<(), PlatformError>;
    fn read_hosts(&self) -> Result<String, PlatformError>;
    fn flush_dns(&self) -> Result<(), PlatformError>;
    fn name(&self) -> &str;
    fn uses_crlf(&self) -> bool;
}
```

Platform detection at startup selects the correct implementation via `cfg` attributes. The write path uses a temp-file-then-copy strategy with platform-specific privilege escalation.

## UI Architecture

The UI is rendered using `ratatui` with a fixed layout:

```
+----------------------------------------------+
| Title bar (1 row)                             |
+----------+-----------------------------------+
| Groups   | Entry table                       |
| (20%)    | (80%)                              |
+----------+-----------------------------------+
| Status bar (2 rows)                           |
+----------------------------------------------+
```

Modal overlays (add/edit form, confirm dialog, backup manager, help) render on top of this base layout using `Clear` widgets and `centered_rect()` for positioning.

### Component Rendering

Each component is a standalone function in `ui/components/` that takes `&App` and a `Rect` and renders into the frame. The top-level `ui::render::render()` dispatches to components based on the current `AppMode`.

## Error Handling

- **Parser**: lenient - unparseable lines are preserved as comments, never fails
- **Validation**: returns `ValidationError` variants with descriptive messages
- **Platform**: returns `PlatformError` for I/O, permission, and escalation failures
- **Backup**: returns `BackupError` for I/O and serialisation failures
- **Commands**: use `anyhow::Result` for ergonomic error propagation
- **TUI**: validation errors display inline in form dialogs; other errors show as toast notifications
