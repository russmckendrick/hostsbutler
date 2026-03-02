# HostsButler - Application Plan

A cross-platform TUI application for managing the system hosts file, built in Rust.

---

## Overview

HostsButler provides a terminal-based user interface for viewing, editing, and managing the system hosts file on macOS, Linux, and Windows. It handles privilege escalation, automatic backups, entry grouping, search/filtering, and DNS resolution testing - all from a keyboard-driven interface.

---

## 1. Project Structure

Single Cargo crate with library modules for testability.

```
hostsbutler/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── docs/
│   └── plan.md
├── src/
│   ├── main.rs                # Entry point: arg parsing, privilege check, launch
│   ├── lib.rs                 # Re-exports for integration tests
│   ├── app.rs                 # Application state machine
│   ├── event.rs               # Event loop: crossterm events -> AppEvent
│   ├── tui.rs                 # Terminal setup/teardown, raw mode, alternate screen
│   ├── model/
│   │   ├── mod.rs
│   │   ├── entry.rs           # HostEntry, EntryStatus
│   │   ├── group.rs           # HostGroup
│   │   ├── hosts_file.rs      # HostsFile document model
│   │   └── line.rs            # Line enum (Entry, Comment, Blank, GroupHeader)
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── reader.rs          # Parse hosts file text -> HostsFile
│   │   └── writer.rs          # Serialize HostsFile -> text (round-trip safe)
│   ├── platform/
│   │   ├── mod.rs             # Platform trait + detection
│   │   ├── macos.rs           # macOS specifics
│   │   ├── linux.rs           # Linux specifics
│   │   └── windows.rs         # Windows specifics
│   ├── backup/
│   │   ├── mod.rs             # BackupManager
│   │   └── store.rs           # Backup metadata, listing, rotation
│   ├── dns/
│   │   └── mod.rs             # DNS resolution testing
│   ├── validation/
│   │   └── mod.rs             # IP/hostname validation, duplicate checks
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs          # Top-level layout partitioning
│   │   ├── theme.rs           # Color palette, style constants
│   │   ├── components/
│   │   │   ├── mod.rs
│   │   │   ├── table_view.rs  # Main hosts table
│   │   │   ├── group_panel.rs # Left sidebar group list
│   │   │   ├── search_bar.rs  # Search/filter input
│   │   │   ├── status_bar.rs  # Bottom bar: mode, shortcuts, messages
│   │   │   ├── entry_dialog.rs# Modal: add/edit entry form
│   │   │   ├── confirm_dialog.rs
│   │   │   ├── backup_view.rs # Backup management screen
│   │   │   ├── help_overlay.rs# Keyboard shortcut reference
│   │   │   └── toast.rs       # Transient notifications
│   │   └── render.rs          # Top-level render dispatch
│   └── commands/
│       ├── mod.rs             # Command enum
│       ├── entry_cmds.rs      # Add, edit, delete, toggle
│       ├── file_cmds.rs       # Save, reload, import, export
│       └── backup_cmds.rs     # Create backup, restore, delete
├── tests/
│   ├── parser_tests.rs
│   ├── validation_tests.rs
│   ├── backup_tests.rs
│   ├── integration_tests.rs
│   └── fixtures/
│       ├── simple.hosts
│       ├── complex.hosts
│       ├── windows.hosts
│       └── malformed.hosts
└── .github/
    └── workflows/
        └── ci.yml
```

---

## 2. Core Data Model

The hosts file is treated as a **document** - comments, blank lines, and formatting are preserved for round-trip fidelity.

### Line (document-level unit)

```rust
pub enum Line {
    Blank(String),
    Comment(String),
    GroupHeader { raw: String, group_name: String },
    Entry(HostEntry),
}
```

### HostEntry

```rust
pub struct HostEntry {
    pub id: usize,                          // Session-unique identifier
    pub status: EntryStatus,                // Enabled or Disabled
    pub ip: IpAddr,                         // Parsed IP (v4 or v6)
    pub hostnames: Vec<String>,             // One or more hostnames
    pub inline_comment: Option<String>,     // Text after # on the same line
    pub group: Option<String>,              // Derived from nearest GroupHeader
    pub raw: String,                        // Original line text
    pub separator: String,                  // Original whitespace between IP and hosts
}

pub enum EntryStatus {
    Enabled,
    Disabled { comment_prefix: String },    // Preserves "# " vs "#" etc.
}
```

### HostsFile (document model)

```rust
pub struct HostsFile {
    pub lines: Vec<Line>,                   // Ordered lines preserving structure
    pub path: PathBuf,                      // Source file path
    pub loaded_at: DateTime<Utc>,           // When last read from disk
    pub original_checksum: String,          // SHA-256 for conflict detection
    pub dirty: bool,                        // Unsaved modifications flag
    next_id: usize,                         // ID counter
}
```

Key operations on `HostsFile`:
- `entries()` - All entries ordered by appearance
- `entries_in_group(group)` - Filtered by group
- `groups()` - All discovered groups with counts
- `add_entry(entry, group)` - Insert into a group
- `toggle_entry(id)` - Enable/disable
- `update_entry(id, updated)` - Edit in place
- `remove_entry(id)` - Delete
- `find_duplicates()` - Detect duplicate IP+hostname pairs
- `to_string()` - Serialize preserving formatting

### Round-trip guarantee

Unmodified lines emit their original `raw` text. Modified entries are re-serialized with standard formatting. A file loaded and immediately saved is byte-identical to the original.

---

## 3. Parser Design

### Parsing strategy

Line-by-line parsing with context tracking (current group):

1. **Blank line** - Empty or whitespace-only
2. **Group header** - Comments matching patterns like `## [GroupName]`, `# --- GroupName ---`
3. **Disabled entry** - Comment where stripping `#` yields a valid `IP hostname...` line
4. **Pure comment** - Any other `#`-prefixed line
5. **Active entry** - `IP hostname [hostname2...] [# comment]`
6. **Unparseable** - Falls through to `Line::Comment` (lenient parsing, never fails)

### Detecting disabled entries vs comments

```
# 127.0.0.1 myhost.local    -> Disabled entry (starts with valid IP after #)
# This is a note             -> Pure comment (no valid IP after #)
```

Strip leading `#` and whitespace, attempt to parse as IP. If valid, it's a disabled entry.

### Writer

Serialization preserves original formatting for unmodified lines. Modified entries use standard format: `IP\thostname1 hostname2 # comment`.

---

## 4. TUI Design

### Architecture

Modified Elm architecture with component rendering:

```
crossterm events -> AppEvent -> App::update() -> Command -> state mutation -> App::render()
```

### Layout

```
+-------------------------------------------------------------------+
| HostsButler v0.1.0                          /etc/hosts  [Modified] |
+----------+--------------------------------------------------------+
|          | Status | IP Address      | Hostname(s)       | Group   |
| Groups   | -------+-----------------+-------------------+---------|
| ------   |  [*]   | 127.0.0.1       | localhost          | System  |
| > All (9)|  [*]   | ::1             | localhost          | System  |
|   System  |  [*]   | 192.168.1.10    | db.local           | Dev     |
|   Dev (3) |  [ ]   | 10.0.0.5        | staging.app        | Dev     |
|   Work (2)|  [*]   | 10.0.0.6        | api.staging        | Dev     |
|   Blocked |  [*]   | 0.0.0.0         | ads.example.com    | Blocked |
|   (4)     |  [*]   | 0.0.0.0         | tracker.bad.com    | Blocked |
|           |  [ ]   | 0.0.0.0         | malware.site       | Blocked |
|           |  [*]   | 172.16.0.50     | jira.company.com   | Work    |
+----------+--------------------------------------------------------+
| [/] Search  [a] Add  [e] Edit  [d] Delete  [Space] Toggle  [?] Help|
+-------------------------------------------------------------------+
```

**Regions:**
1. **Title bar** (1 row) - App name, version, file path, dirty indicator
2. **Group panel** (20% width) - Scrollable group list with entry counts
3. **Entry table** (80% width) - Columns: Status, IP, Hostname(s), Group, Comment
4. **Status bar** (2 rows) - Context-sensitive shortcuts, search input, toast messages

### Modal Dialogs

Centered overlay widgets for add/edit forms and confirmations:

```
+-----------------------------------+
|  Add New Host Entry               |
|                                   |
|  IP Address:  [192.168.1.___    ] |
|  Hostnames:   [myhost.local     ] |
|  Group:       [Development    v ] |
|  Comment:     [My dev server    ] |
|  Enabled:     [x]                 |
|                                   |
|  [Tab] Next  [Enter] Save  [Esc] |
+-----------------------------------+
```

### Application Modes

```rust
pub enum AppMode {
    Normal,                 // Main view, keyboard navigation
    Search,                 // Search bar active
    AddEntry,               // Add entry modal open
    EditEntry(usize),       // Edit modal open for entry ID
    ConfirmDelete(usize),   // Delete confirmation
    ConfirmSave,            // Save confirmation
    BackupManager,          // Backup list view
    Help,                   // Help overlay
}
```

### Keyboard Shortcuts

| Key | Mode | Action |
|-----|------|--------|
| `j` / `↓` | Normal | Move selection down |
| `k` / `↑` | Normal | Move selection up |
| `g` / `Home` | Normal | Jump to first entry |
| `G` / `End` | Normal | Jump to last entry |
| `Tab` | Normal | Switch focus: group panel / table |
| `Enter` | Normal | Edit selected entry |
| `Space` | Normal | Toggle enable/disable |
| `a` | Normal | Add new entry |
| `e` | Normal | Edit selected entry |
| `d` | Normal | Delete (with confirmation) |
| `/` | Normal | Enter search mode |
| `Esc` | Search | Exit search, clear filter |
| `Enter` | Search | Confirm search |
| `Ctrl+S` | Any | Save file |
| `Ctrl+Z` | Normal | Undo last change |
| `Ctrl+Y` | Normal | Redo |
| `Ctrl+R` | Normal | Reload from disk |
| `b` | Normal | Open backup manager |
| `i` | Normal | Import entries |
| `x` | Normal | Export entries |
| `t` | Normal | Test DNS resolution |
| `?` | Normal | Show help |
| `q` / `Ctrl+C` | Normal | Quit (unsaved warning) |

### Color Theme

| Element | Color |
|---------|-------|
| Header background | Blue |
| Selected row | DarkGray bg, Yellow fg |
| Enabled entry | Green indicator |
| Disabled entry | DarkGray (dimmed) |
| Active group | Cyan |
| Error messages | Red |
| Success messages | Green |
| Search highlight | Yellow |
| Borders | Gray |

---

## 5. Features

### 5.1 Core CRUD
- **View**: Scrollable table with all entries, sortable columns
- **Add**: Modal form with IP, hostnames, group, comment, enabled fields
- **Edit**: Same modal pre-populated with existing values
- **Delete**: Confirmation dialog, removes line from document

### 5.2 Enable/Disable Toggle
- `Space` toggles commenting/uncommenting
- Disabled entries prefixed with `# ` in the file
- Visual indicator: `[*]` enabled, `[ ]` disabled with dimmed text

### 5.3 Entry Grouping
- Group headers in file: `## [GroupName]`
- Sidebar lists groups with entry counts
- Filter table by selecting a group
- Create new groups when adding/editing entries
- Move entries between groups

### 5.4 Search and Filter
- `/` activates search bar
- Real-time filtering as user types
- Searches across IP, hostnames, group, comments
- Prefix filters: `ip:`, `host:`, `group:`
- Highlights matching text

### 5.5 Backup Management
- **Auto-backup** before every save
- **Manual backup** with optional description
- **Backup manager** view: list, preview diff, restore, delete
- **Storage locations**:
  - macOS: `~/Library/Application Support/hostsbutler/backups/`
  - Linux: `~/.config/hostsbutler/backups/`
  - Windows: `%APPDATA%\hostsbutler\backups\`
- **Naming**: `hosts_YYYY-MM-DD_HH-MM-SS.bak` with `.meta.json` sidecar
- **Rotation**: Keep last 20 backups (configurable)

### 5.6 Duplicate Detection
- Scan for duplicate IP+hostname pairs on load
- Warn before creating duplicates in add/edit
- Dedicated "Show duplicates" filter

### 5.7 Validation
- **IPv4**: Standard dotted-quad, reject leading zeros, range check
- **IPv6**: Full and abbreviated forms, mapped addresses
- **Hostnames**: RFC 1123 (alphanumeric, hyphens, dots, max 253 chars, labels max 63)
- Real-time validation in add/edit forms

### 5.8 Import / Export
- **Export formats**: hosts (plain text), JSON, CSV
- **Import**: Auto-detect format, preview entries, conflict resolution (skip/overwrite/add)
- Export filtered or selected entries

### 5.9 DNS Resolution Testing
- Select entry, press `t` to test
- Resolves hostname(s) via system DNS
- Shows resolved IP vs hosts file IP (match/mismatch)
- Non-blocking with result shown as toast

### 5.10 Undo / Redo
- Stack of document snapshots
- `Ctrl+Z` undo, `Ctrl+Y` redo
- Stack cleared on save or reload

### 5.11 File Conflict Detection
- Before saving, re-read file and compare checksum
- If externally modified: prompt Overwrite / Reload / Cancel

---

## 6. Cross-Platform Support

### Platform Trait

```rust
pub trait Platform: Send + Sync {
    fn hosts_path(&self) -> PathBuf;
    fn config_dir(&self) -> PathBuf;
    fn can_write(&self) -> bool;
    fn write_hosts(&self, content: &str) -> Result<(), PlatformError>;
    fn read_hosts(&self) -> Result<String, PlatformError>;
    fn flush_dns(&self) -> Result<(), PlatformError>;
    fn name(&self) -> &str;
}
```

### macOS
- **Path**: `/etc/hosts`
- **Permissions**: `root:wheel`, `0644`
- **Escalation**: `sudo cp` from temp file, or `sudo tee`
- **DNS flush**: `dscacheutil -flushcache` + `killall -HUP mDNSResponder`

### Linux
- **Path**: `/etc/hosts`
- **Permissions**: `root:root`, `0644`
- **Escalation**: Try `pkexec` first (desktop), fall back to `sudo`
- **DNS flush**: `systemctl restart systemd-resolved` or `resolvectl flush-caches`

### Windows
- **Path**: `%SystemRoot%\System32\drivers\etc\hosts`
- **Permissions**: Administrator only
- **Escalation**: Check `IsUserAnAdmin`, re-launch with UAC via `ShellExecuteW` + `runas`
- **DNS flush**: `ipconfig /flushdns`
- **Line endings**: Emit CRLF on Windows

### Safe Write Strategy (all platforms)
1. Serialize to string
2. Write to temporary file
3. Create auto-backup of current hosts file
4. Platform-specific privilege escalation to replace hosts file
5. Verify by re-reading and comparing checksum
6. Optionally flush DNS cache

---

## 7. Dependencies

```toml
[dependencies]
# TUI
ratatui = "0.30"
crossterm = "0.29"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
csv = "1"

# Networking
dns-lookup = "2"

# Filesystem & Platform
dirs = "6"
tempfile = "3"
sha2 = "0.10"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Error Handling
thiserror = "2"
anyhow = "1"
color-eyre = "0.6"

# CLI
clap = { version = "4", features = ["derive"] }

# Validation
regex = "1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Text
unicode-width = "0.2"

[target.'cfg(windows)'.dependencies]
is_elevated = "0.1"
winapi = { version = "0.3", features = ["shellapi", "winuser"] }

[dev-dependencies]
pretty_assertions = "1"
criterion = "0.5"
assert_fs = "1"
predicates = "3"
insta = "1"
```

**Key choices:**
- **ratatui + crossterm**: Dominant Rust TUI stack, works on all platforms
- **Custom parser**: Existing crates don't preserve formatting for round-trip fidelity
- **thiserror + anyhow**: Structured errors in library code, ergonomic propagation in app code
- **dns-lookup**: Thin `getaddrinfo` wrapper, sufficient for hostname verification
- **color-eyre**: Clean panic reports with terminal restoration

---

## 8. Error Handling

### Error Types

- **ParseError**: I/O failures, malformed lines (lenient - unparseable lines become comments)
- **ModelError**: Entry not found, duplicate entry, invalid hostname/IP
- **PlatformError**: Permission denied, escalation failed, file conflict, DNS flush failure
- **BackupError**: Backup not found, checksum mismatch, I/O errors

### Patterns

- Parser is **lenient**: never fails entirely, unparseable lines preserved as comments
- Permission errors show clear toast with actionable message
- File conflicts prompt: Overwrite / Reload / Cancel
- Validation errors shown inline in modal forms
- Panic hook restores terminal before crash report

---

## 9. Testing Strategy

### Unit Tests
- Parser round-trip tests with fixture files
- Validation edge cases (IPv4, IPv6, hostnames)
- Model CRUD operations
- Backup create/restore/rotation

### Snapshot Tests (insta)
- Parse each fixture, snapshot the structure
- Serialize each fixture, snapshot the output

### Integration Tests
- Full cycle: parse -> modify -> save -> reload -> verify (using temp files)
- Import from JSON/CSV, verify merged output

### Test Fixtures
- `simple.hosts` - Basic entries
- `complex.hosts` - Groups, comments, mixed IPv4/v6, disabled entries
- `windows.hosts` - CRLF line endings
- `malformed.hosts` - Edge cases, invalid lines

### Benchmarks
- Parse/serialize 10,000-line hosts file

---

## 10. Build & Distribution

### CI (GitHub Actions)
- Matrix: `ubuntu-latest`, `macos-latest`, `windows-latest`
- Steps: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --release`

### Release Profile
```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

### Distribution
1. **GitHub Releases** - Pre-built binaries (macOS x86_64 + aarch64, Linux x86_64/aarch64, Windows x86_64)
2. **Cargo** - `cargo install hostsbutler`
3. **Homebrew** - Tap formula for macOS/Linux
4. **AUR** - Arch Linux package
5. **Scoop** - Windows package manager

---

## 11. Implementation Phases

### Phase 1: Foundation (Core Logic)
1. Project skeleton: `Cargo.toml`, module stubs, CI
2. Data model (`model/`)
3. Parser and writer (`parser/`) with round-trip tests
4. Validation (`validation/`)
5. Platform detection and hosts file reading (`platform/`, read-only)
6. Unit tests and fixture files

### Phase 2: File Operations
7. Privilege-escalated writing (`platform/` write path)
8. Backup manager (`backup/`)
9. DNS resolution testing (`dns/`)
10. Import/export (`commands/file_cmds.rs`)

### Phase 3: TUI Shell
11. Terminal setup/teardown (`tui.rs`)
12. Event loop (`event.rs`)
13. Layout skeleton (`ui/layout.rs`)
14. Main table view (`ui/components/table_view.rs`)
15. Status bar (`ui/components/status_bar.rs`)

### Phase 4: TUI Features
16. Group panel
17. Search bar
18. Add/edit entry dialog
19. Confirm dialog
20. Backup management view
21. Help overlay
22. Toast notifications

### Phase 5: Polish
23. Undo/redo
24. File conflict detection
25. CLI arguments (file path override, read-only mode, export subcommand)
26. Performance testing with large files
27. Cross-platform testing
28. README and documentation
