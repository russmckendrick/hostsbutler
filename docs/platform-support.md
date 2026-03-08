# Platform Support

HostsButler runs on macOS, Linux, and Windows. This document covers platform-specific behaviour.

## Hosts File Locations

| Platform | Path |
|----------|------|
| macOS | `/etc/hosts` |
| Linux | `/etc/hosts` |
| Windows | `%SystemRoot%\System32\drivers\etc\hosts` |

On Windows, `%SystemRoot%` typically resolves to `C:\Windows`.

You can override the path with `--file`:

```sh
hostsbutler --file /path/to/custom/hosts
```

## Permissions and Privilege Escalation

### macOS

The hosts file is owned by `root:wheel` with mode `0644`. Reading does not require elevation. Writing requires root access.

**Escalation strategy**: HostsButler writes to a temporary file, then uses `sudo cp` to replace the hosts file. You will see a password prompt in the terminal if `sudo` requires authentication.

### Linux

The hosts file is owned by `root:root` with mode `0644`. Reading does not require elevation. Writing requires root access.

**Escalation strategy**: HostsButler first attempts `pkexec` (for desktop environments with PolicyKit). If that fails, it falls back to `sudo cp`. On headless systems, `sudo` will be used directly.

### Windows

The hosts file requires Administrator access for writing.

**Escalation strategy**: HostsButler checks for elevated privileges using the `is_elevated` crate. If the process is not running as Administrator, write operations return an error. Launch your terminal (Command Prompt, PowerShell, or Windows Terminal) as Administrator before running HostsButler.

## DNS Cache Flushing

After a successful write to the real system hosts file, HostsButler attempts to flush the DNS cache so changes take effect immediately. Flush failures are reported as warnings and do not fail the save or CLI import. Writes to custom files opened with `--file` do not trigger DNS flushing.

| Platform | Command(s) |
|----------|------------|
| macOS | `dscacheutil -flushcache` and `sudo killall -HUP mDNSResponder` |
| Linux | `resolvectl flush-caches` (primary) or `sudo systemctl restart systemd-resolved` (fallback) |
| Windows | `ipconfig /flushdns` |

## Line Endings

| Platform | Line Ending | Behaviour |
|----------|-------------|-----------|
| macOS | LF (`\n`) | Read and written as-is |
| Linux | LF (`\n`) | Read and written as-is |
| Windows | CRLF (`\r\n`) | Normalised to LF on read, converted back to CRLF on write |

This is handled transparently. The internal document model always uses LF; conversion happens at the platform boundary.

## Configuration and Backup Directories

| Platform | Config Directory |
|----------|-----------------|
| macOS | `~/Library/Application Support/hostsbutler/` |
| Linux | `~/.config/hostsbutler/` |
| Windows | `%APPDATA%\hostsbutler\` |

Backups are stored in a `backups/` subdirectory within the config directory. Up to 20 backups are retained.

The directory is created automatically on first use.

## Running as Root/Admin

### macOS and Linux

The recommended approach is to run HostsButler with `sudo`:

```sh
sudo hostsbutler
```

Alternatively, you can run without `sudo` and HostsButler will prompt for elevation when saving.

### Windows

Right-click your terminal application and select "Run as Administrator", then launch HostsButler normally:

```sh
hostsbutler
```

## CI Matrix

The CI pipeline tests on all three platforms:

| Runner | Platform |
|--------|----------|
| `ubuntu-latest` | Linux |
| `macos-latest` | macOS |
| `windows-latest` | Windows |

Each platform runs formatting checks, clippy lints, a release build, and the full test suite.
