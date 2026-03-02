use std::net::IpAddr;
use std::path::PathBuf;

use regex::Regex;
use sha2::{Digest, Sha256};

use crate::model::{EntryStatus, HostEntry, HostsFile, Line};

pub fn parse_hosts_file(content: &str, path: PathBuf) -> HostsFile {
    let checksum = compute_checksum(content);
    let mut lines = Vec::new();
    let mut next_id: usize = 0;
    let mut current_group: Option<String> = None;

    let group_re = Regex::new(r"^##\s*\[(.+)\]\s*$").expect("valid regex");
    let group_dash_re = Regex::new(r"^#\s*---+\s*(.+?)\s*---+\s*$").expect("valid regex");

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();

        // Blank line
        if trimmed.is_empty() {
            lines.push(Line::Blank(raw_line.to_string()));
            continue;
        }

        // Check for group header patterns
        if let Some(caps) = group_re.captures(trimmed) {
            let group_name = caps[1].trim().to_string();
            current_group = Some(group_name.clone());
            lines.push(Line::GroupHeader {
                raw: raw_line.to_string(),
                group_name,
            });
            continue;
        }
        if let Some(caps) = group_dash_re.captures(trimmed) {
            let group_name = caps[1].trim().to_string();
            current_group = Some(group_name.clone());
            lines.push(Line::GroupHeader {
                raw: raw_line.to_string(),
                group_name,
            });
            continue;
        }

        // Comment or disabled entry
        if trimmed.starts_with('#') {
            if let Some(entry) = try_parse_disabled_entry(raw_line, next_id, &current_group) {
                next_id += 1;
                lines.push(Line::Entry(entry));
            } else {
                lines.push(Line::Comment(raw_line.to_string()));
            }
            continue;
        }

        // Active entry
        if let Some(entry) = try_parse_active_entry(raw_line, next_id, &current_group) {
            next_id += 1;
            lines.push(Line::Entry(entry));
        } else {
            // Unparseable - treat as comment
            lines.push(Line::Comment(raw_line.to_string()));
        }
    }

    let mut hosts = HostsFile::new(path, lines, checksum);
    hosts.trailing_newline = content.ends_with('\n');
    hosts
}

fn try_parse_active_entry(
    raw: &str,
    id: usize,
    current_group: &Option<String>,
) -> Option<HostEntry> {
    let (content, inline_comment) = split_inline_comment(raw);
    let parts: Vec<&str> = content.split_whitespace().collect();

    if parts.len() < 2 {
        return None;
    }

    let ip: IpAddr = parts[0].parse().ok()?;
    let hostnames: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    // Detect separator between IP and first hostname
    let separator = detect_separator(raw, parts[0]);

    Some(HostEntry {
        id,
        status: EntryStatus::Enabled,
        ip,
        hostnames,
        inline_comment,
        group: current_group.clone(),
        raw: raw.to_string(),
        separator,
    })
}

fn try_parse_disabled_entry(
    raw: &str,
    id: usize,
    current_group: &Option<String>,
) -> Option<HostEntry> {
    let trimmed = raw.trim();

    // Find the comment prefix (e.g., "# " or "#")
    let without_hash = trimmed.strip_prefix('#')?;
    let comment_prefix = if without_hash.starts_with(' ') {
        "# ".to_string()
    } else {
        "#".to_string()
    };

    let stripped = without_hash.trim_start();

    // Try to parse as an entry
    let (content, inline_comment) = split_inline_comment(stripped);
    let parts: Vec<&str> = content.split_whitespace().collect();

    if parts.len() < 2 {
        return None;
    }

    let ip: IpAddr = parts[0].parse().ok()?;
    let hostnames: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    let separator = detect_separator(stripped, parts[0]);

    Some(HostEntry {
        id,
        status: EntryStatus::Disabled { comment_prefix },
        ip,
        hostnames,
        inline_comment,
        group: current_group.clone(),
        raw: raw.to_string(),
        separator,
    })
}

fn split_inline_comment(line: &str) -> (&str, Option<String>) {
    // Find # that's not at the start of the line and is preceded by whitespace
    if let Some(pos) = line.find(" #") {
        let content = line[..pos].trim_end();
        let comment = line[pos + 2..].trim().to_string();
        if comment.is_empty() {
            (content, None)
        } else {
            (content, Some(comment))
        }
    } else if let Some(pos) = line.find("\t#") {
        let content = line[..pos].trim_end();
        let comment = line[pos + 2..].trim().to_string();
        if comment.is_empty() {
            (content, None)
        } else {
            (content, Some(comment))
        }
    } else {
        (line.trim_end(), None)
    }
}

fn detect_separator(line: &str, ip_str: &str) -> String {
    if let Some(pos) = line.find(ip_str) {
        let after_ip = &line[pos + ip_str.len()..];
        let whitespace_len = after_ip.len() - after_ip.trim_start().len();
        if whitespace_len > 0 {
            after_ip[..whitespace_len].to_string()
        } else {
            "\t".to_string()
        }
    } else {
        "\t".to_string()
    }
}

fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_simple_entry() {
        let content = "127.0.0.1\tlocalhost";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        let entries = hosts.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ip.to_string(), "127.0.0.1");
        assert_eq!(entries[0].hostnames, vec!["localhost"]);
        assert!(entries[0].status.is_enabled());
    }

    #[test]
    fn test_parse_disabled_entry() {
        let content = "# 127.0.0.1 disabled.local";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        let entries = hosts.entries();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].status.is_enabled());
        assert_eq!(entries[0].ip.to_string(), "127.0.0.1");
    }

    #[test]
    fn test_parse_comment() {
        let content = "# This is a comment";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        assert_eq!(hosts.entries().len(), 0);
        assert_eq!(hosts.lines.len(), 1);
        assert!(matches!(&hosts.lines[0], Line::Comment(_)));
    }

    #[test]
    fn test_parse_group_header() {
        let content = "## [Development]\n192.168.1.10\tdev.local";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        assert!(
            matches!(&hosts.lines[0], Line::GroupHeader { group_name, .. } if group_name == "Development")
        );
        let entries = hosts.entries();
        assert_eq!(entries[0].group.as_deref(), Some("Development"));
    }

    #[test]
    fn test_parse_inline_comment() {
        let content = "127.0.0.1\tlocalhost # loopback";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        let entries = hosts.entries();
        assert_eq!(entries[0].inline_comment.as_deref(), Some("loopback"));
    }

    #[test]
    fn test_parse_multiple_hostnames() {
        let content = "127.0.0.1\tlocalhost myhost.local";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        let entries = hosts.entries();
        assert_eq!(entries[0].hostnames, vec!["localhost", "myhost.local"]);
    }

    #[test]
    fn test_parse_blank_lines() {
        let content = "127.0.0.1\tlocalhost\n\n::1\tlocalhost";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        assert_eq!(hosts.lines.len(), 3);
        assert!(matches!(&hosts.lines[1], Line::Blank(_)));
    }

    #[test]
    fn test_parse_ipv6() {
        let content = "::1\tlocalhost";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        let entries = hosts.entries();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].ip.is_ipv6());
    }

    #[test]
    fn test_round_trip_unmodified() {
        let content = "# Comment line\n127.0.0.1\tlocalhost\n\n## [Dev]\n192.168.1.10\tdev.local # my dev server";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        assert_eq!(hosts.to_string(), content);
    }

    #[test]
    fn test_dash_group_header() {
        let content = "# --- Production ---\n10.0.0.1\tprod.local";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        assert!(
            matches!(&hosts.lines[0], Line::GroupHeader { group_name, .. } if group_name == "Production")
        );
    }
}
