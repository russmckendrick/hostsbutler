use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model::{HostEntry, HostsFile};
use crate::parser;

#[derive(Debug, Serialize, Deserialize)]
struct ExportEntry {
    ip: String,
    hostnames: Vec<String>,
    enabled: bool,
    group: Option<String>,
    comment: Option<String>,
}

impl From<&HostEntry> for ExportEntry {
    fn from(entry: &HostEntry) -> Self {
        Self {
            ip: entry.ip.to_string(),
            hostnames: entry.hostnames.clone(),
            enabled: entry.status.is_enabled(),
            group: entry.group.clone(),
            comment: entry.inline_comment.clone(),
        }
    }
}

pub fn export_json(hosts: &HostsFile, path: &Path) -> Result<()> {
    let entries: Vec<ExportEntry> = hosts
        .entries()
        .iter()
        .map(|e| ExportEntry::from(*e))
        .collect();
    let json = serde_json::to_string_pretty(&entries)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn export_csv(hosts: &HostsFile, path: &Path) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["ip", "hostnames", "enabled", "group", "comment"])?;

    for entry in hosts.entries() {
        wtr.write_record([
            &entry.ip.to_string(),
            &entry.hostnames.join(" "),
            &entry.status.is_enabled().to_string(),
            entry.group.as_deref().unwrap_or(""),
            entry.inline_comment.as_deref().unwrap_or(""),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn export_hosts(hosts: &HostsFile, path: &Path) -> Result<()> {
    let content = parser::serialize_hosts_file(hosts);
    fs::write(path, content)?;
    Ok(())
}

pub fn import_json(hosts: &mut HostsFile, path: &Path) -> Result<usize> {
    let content = fs::read_to_string(path)?;
    let entries: Vec<ExportEntry> = serde_json::from_str(&content)?;
    let count = entries.len();

    for export_entry in entries {
        let ip = export_entry.ip.parse()?;
        let mut entry = HostEntry::new(hosts.next_id(), ip, export_entry.hostnames);
        entry.inline_comment = export_entry.comment;

        if !export_entry.enabled {
            entry.status = crate::model::EntryStatus::Disabled {
                comment_prefix: "# ".to_string(),
            };
        }

        hosts.add_entry(entry, export_entry.group.as_deref());
    }

    Ok(count)
}

pub fn import_hosts(hosts: &mut HostsFile, path: &Path) -> Result<usize> {
    let content = fs::read_to_string(path)?;
    let imported = parser::parse_hosts_file(&content, path.to_path_buf());
    let entries = imported.entries();
    let count = entries.len();

    for entry in entries {
        let mut new_entry = entry.clone();
        new_entry.id = hosts.next_id();
        new_entry.raw = String::new();
        hosts.add_entry(new_entry, entry.group.as_deref());
    }

    Ok(count)
}
