# Hosts File Format

This document describes how HostsButler parses, interprets, and writes the hosts file.

## Standard Format

A hosts file consists of lines in the format:

```
IP_ADDRESS    HOSTNAME [HOSTNAME2 ...] [# COMMENT]
```

For example:

```
127.0.0.1       localhost
::1             localhost
192.168.1.10    dev.local api.dev.local    # development server
```

## Line Types

HostsButler classifies each line into one of four types:

### Active Entry

A line starting with a valid IP address followed by one or more hostnames:

```
192.168.1.10    myhost.local
```

### Disabled Entry

A commented-out line that would be a valid entry if the `#` were removed:

```
# 192.168.1.10    myhost.local
```

HostsButler detects these by stripping the leading `#` (and optional space) and attempting to parse the remainder as an IP address. If successful, it's treated as a disabled entry rather than a plain comment.

### Comment

Any `#`-prefixed line that is not a disabled entry or group header:

```
# This is a comment about the next section
```

### Blank Line

Empty lines or lines containing only whitespace:

```

```

### Group Header

Special comment formats that HostsButler interprets as group markers:

```
## [Development]
# --- Production ---
```

Two patterns are recognised:

| Pattern | Example |
|---------|---------|
| `## [Name]` | `## [Development]` |
| `# --- Name ---` | `# --- Production ---` |

Entries following a group header are assigned to that group until the next header.

## Parsing Rules

### IP Addresses

Both IPv4 and IPv6 addresses are supported:

```
127.0.0.1       localhost           # IPv4
::1             localhost           # IPv6
fe80::1         link-local.host     # IPv6 link-local
2001:db8::1     example.host        # IPv6 documentation range
0.0.0.0         blocked.domain      # Block address
```

### Multiple Hostnames

A single line can map one IP to multiple hostnames, separated by whitespace:

```
192.168.1.10    host1.local host2.local host3.local
```

### Inline Comments

Text after `#` (preceded by a space or tab) on an entry line is treated as an inline comment:

```
192.168.1.10    dev.local    # Development server
```

### Whitespace

The separator between IP and hostnames is preserved. Common formats include:

```
127.0.0.1	localhost           # Tab separator
127.0.0.1       localhost           # Space separator (8 spaces)
127.0.0.1 localhost                 # Single space
```

HostsButler detects and preserves the original separator for unmodified entries. New or modified entries use a tab separator.

## Round-Trip Fidelity

HostsButler guarantees that loading a file and saving it without changes produces byte-identical output. This is achieved by:

1. Storing the original line text (`raw` field) for each line
2. Using `raw` for serialisation of unmodified lines
3. Only re-serialising entries that have been modified (where `raw` is cleared)
4. Preserving trailing newlines

### Modified Entry Format

When an entry is modified (added, edited, or toggled), it is serialised in a standard format:

```
IP\tHOSTNAME1 HOSTNAME2 # COMMENT
```

For disabled entries:

```
# IP\tHOSTNAME1 HOSTNAME2 # COMMENT
```

## Hostname Validation

Hostnames are validated according to RFC 1123:

- Maximum total length: 253 characters
- Maximum label length: 63 characters (labels are dot-separated segments)
- Allowed characters: ASCII alphanumeric and hyphens
- Labels cannot start or end with a hyphen
- Labels cannot be empty (no consecutive dots)

### Valid Examples

```
localhost
my-host.local
sub.domain.example.com
host1
a.b.c.d.e
```

### Invalid Examples

```
-starts-with-dash
ends-with-dash-
has space
has_underscore
too..many.dots
```

## IP Address Validation

IP addresses are parsed using Rust's `std::net::IpAddr`:

### Valid IPv4

```
127.0.0.1
0.0.0.0
192.168.1.1
255.255.255.255
10.0.0.1
```

### Valid IPv6

```
::1
fe80::1
2001:db8::1
2001:0db8:85a3:0000:0000:8a2e:0370:7334
```

## Export Formats

### JSON

```json
[
  {
    "ip": "127.0.0.1",
    "hostnames": ["localhost"],
    "enabled": true,
    "group": null,
    "comment": "loopback"
  },
  {
    "ip": "192.168.1.10",
    "hostnames": ["dev.local", "api.dev.local"],
    "enabled": true,
    "group": "Development",
    "comment": "dev servers"
  }
]
```

### CSV

```csv
ip,hostnames,enabled,group,comment
127.0.0.1,localhost,true,,loopback
192.168.1.10,dev.local api.dev.local,true,Development,dev servers
```

Hostnames are space-separated within the hostnames column. Empty optional fields are represented as empty strings.

### Hosts Format

The native hosts file format, identical to the source file for unmodified entries.

## Import Behaviour

### JSON Import

Entries from the JSON array are added to the current hosts file. Each entry receives a new session ID. Group assignments are preserved.

### Hosts File Import

The imported file is parsed using the same parser. All entries are extracted and added to the current file with new session IDs. Comments and blank lines from the imported file are not carried over.

## Windows Line Endings

On Windows, HostsButler:

1. Converts CRLF (`\r\n`) to LF (`\n`) when reading
2. Processes everything internally using LF
3. Converts LF back to CRLF when writing

This is transparent to the user. On macOS and Linux, LF is used throughout.
