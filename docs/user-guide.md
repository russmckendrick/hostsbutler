# User Guide

A practical guide to using HostsButler for managing your system hosts file.

## Getting Started

### Launching

HostsButler needs read access to your hosts file. For editing and saving, it needs write access (typically root/admin):

```sh
# macOS / Linux
sudo hostsbutler

# Windows (run terminal as Administrator)
hostsbutler
```

To browse a hosts file without writing:

```sh
hostsbutler --file /path/to/hosts --readonly
```

In read-only mode, HostsButler still lets you browse, search, inspect backups, and test DNS resolution, but it blocks edits, saves, backup mutations, restores, and CLI import.

### First Launch

When you launch HostsButler, it reads your system hosts file and presents it in a two-panel layout:

- **Left panel**: Groups found in your hosts file
- **Right panel**: Table of all entries with status, IP, hostnames, group, and comments

The status bar at the bottom shows available keyboard shortcuts and the current mode.

## Navigating

### Moving Around

Use vim-style keys or arrow keys:

- `j` or `Down` - move selection down
- `k` or `Up` - move selection up
- `g` or `Home` - jump to top
- `G` or `End` - jump to bottom

### Switching Panels

Press `Tab` to switch focus between the groups panel and the entries table. The focused panel has a cyan border.

### Filtering by Group

When the groups panel is focused, use `j`/`k` to select a group. The entries table automatically filters to show only entries in that group. Select "All" to show everything.

## Managing Entries

### Adding an Entry

1. Press `a` to open the add form
2. Fill in the fields:
   - **IP Address**: e.g. `192.168.1.100`
   - **Hostnames**: space-separated, e.g. `myhost.local api.myhost.local`
   - **Group**: optional group name, e.g. `Development`
   - **Comment**: optional inline comment
   - **Enabled**: checkbox (`Space` to toggle)
3. Press `Tab` to move between fields
4. Press `Enter` to save

If validation fails (invalid IP, bad hostname format), an error message appears in the form.

### Editing an Entry

1. Navigate to the entry
2. Press `e` or `Enter`
3. Modify fields as needed
4. Press `Enter` to save or `Esc` to cancel

### Deleting an Entry

1. Navigate to the entry
2. Press `d`
3. Confirm with `y` or cancel with `n`/`Esc`

### Toggling Enable/Disable

Press `Space` on any entry to toggle it between enabled and disabled. Disabled entries are commented out in the hosts file (prefixed with `# `) and appear dimmed in the table.

## Searching

Press `/` to enter search mode. Type your query and results filter in real-time.

### Search Prefixes

Use prefixes to search specific fields:

| Prefix | Example | Searches |
|--------|---------|----------|
| `ip:` | `ip:192.168` | IP address field |
| `host:` | `host:example` | Hostname field |
| `group:` | `group:dev` | Group name |
| (none) | `myserver` | All fields |

Press `Enter` to keep the filter active, or `Esc` to clear the search and show all entries.

## Saving

Press `Ctrl+S` to save. Before writing, HostsButler:

1. Creates an automatic backup of the current file
2. Serialises the document (preserving all formatting)
3. Writes using platform-specific privilege escalation
4. Attempts to flush the DNS cache if you wrote the real system hosts file
5. Clears the undo history

If DNS cache flushing fails, the save still succeeds and HostsButler shows a warning.

The title bar shows `[Modified]` when there are unsaved changes. If you try to quit with unsaved changes, you'll be prompted to save first.

## Undo and Redo

Every change (add, edit, delete, toggle) can be undone:

- `Ctrl+Z` - undo the last change
- `Ctrl+Y` - redo

The undo history is cleared when you save or reload the file.

## Backup Manager

Press `b` to open the backup manager. From here you can:

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate backups |
| `r` / `Enter` | Restore selected backup |
| `c` | Create a manual backup |
| `d` | Delete selected backup |
| `Esc` | Close backup manager |

### Automatic Backups

A backup is created automatically before every save with the description "Auto-backup before save". Up to 20 backups are retained; older ones are automatically removed.

### Backup Storage

Backups are stored as timestamped files alongside JSON metadata:

```
backups/
  hosts_2025-01-15_14-30-22.bak
  hosts_2025-01-15_14-30-22.bak.meta.json
```

The metadata includes the creation timestamp, file size, SHA-256 checksum, and optional description.

## DNS Resolution Testing

Select an entry and press `t` to test its DNS resolution. HostsButler resolves each hostname and compares the result to the IP in the hosts file:

- **OK** - DNS resolves to the hosts file IP
- **MISMATCH** - DNS resolves to a different IP

Results appear as a toast notification at the bottom of the screen.

## Import and Export

Import is currently available from the CLI. Export remains CLI-based as well.

### Importing from the CLI

```sh
# Import from JSON
hostsbutler --file /path/to/hosts --import entries.json

# Import from CSV
hostsbutler --file /path/to/hosts --import entries.csv

# Import entries from another hosts file
hostsbutler --file /path/to/hosts --import backup.hosts
```

Imports merge entries into the target hosts file, assign new session IDs, preserve group/comment/enabled state, and then save the merged document. JSON and CSV imports are validated before insertion. Hosts imports bring in entries only, not comments or blank lines.

### Exporting from the CLI

```sh
# Export to JSON
hostsbutler --export entries.json

# Export to CSV
hostsbutler --export entries.csv

# Export as hosts file
hostsbutler --export backup.hosts
```

### JSON Format

Entries are exported as a JSON array:

```json
[
  {
    "ip": "127.0.0.1",
    "hostnames": ["localhost"],
    "enabled": true,
    "group": null,
    "comment": "loopback"
  }
]
```

### CSV Format

```csv
ip,hostnames,enabled,group,comment
127.0.0.1,localhost,true,,loopback
192.168.1.10,dev.local api.dev.local,true,Development,dev servers
```

CSV imports use the same columns. `hostnames` stays space-separated within a single field.

## Entry Grouping

### How Groups Work

Groups are defined by special comment headers in the hosts file. HostsButler recognises two formats:

```
## [Development]
192.168.1.10    dev.local

# --- Production ---
10.0.0.1    prod.example.com
```

Entries after a group header belong to that group until the next header. Entries before any header are shown as "Ungrouped".

### Creating Groups

When adding or editing an entry, type a group name in the Group field. If the group doesn't exist, HostsButler creates a new `## [GroupName]` header and inserts the entry beneath it.

### Filtering

Select a group in the left panel to filter the table. The count next to each group name shows how many entries it contains.

## Reloading

Press `Ctrl+R` to reload the current hosts file from disk. This discards any unsaved changes and re-parses the active file, including custom paths opened with `--file`.

## Help

Press `?` at any time to show the keyboard shortcut reference overlay. Press `?` or `Esc` to dismiss it.

## Tips

- **Blocking domains**: Use `0.0.0.0` as the IP to block a domain. Create a "Blocked" group to keep them organised.
- **Multiple hostnames**: A single entry can map one IP to multiple hostnames, separated by spaces.
- **Disabled entries**: Toggle entries off instead of deleting them - you can easily re-enable them later.
- **Backup before experiments**: The auto-backup feature means you can always restore if something goes wrong, but you can also create manual backups with descriptions.
