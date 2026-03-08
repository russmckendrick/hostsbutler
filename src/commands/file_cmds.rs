use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::backup::BackupManager;
use crate::commands::{backup_cmds, entry_cmds};
use crate::model::{HostEntry, HostsFile};
use crate::parser;
use crate::platform::Platform;

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

#[derive(Debug, Deserialize)]
struct CsvImportEntry {
    ip: String,
    hostnames: String,
    enabled: bool,
    #[serde(default)]
    group: String,
    #[serde(default)]
    comment: String,
}

#[derive(Debug, Default)]
pub struct PersistResult {
    pub backup_warning: Option<String>,
    pub dns_flush_warning: Option<String>,
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

pub fn read_hosts_content(path: &Path, platform: &dyn Platform) -> Result<String> {
    if path == platform.hosts_path() {
        Ok(platform.read_hosts()?)
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

pub fn import_file(hosts: &mut HostsFile, path: &Path) -> Result<usize> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => import_json(hosts, path),
        Some("csv") => import_csv(hosts, path),
        _ => import_hosts(hosts, path),
    }
}

pub fn persist_hosts_with_actions<W, F>(
    hosts: &HostsFile,
    platform: &dyn Platform,
    backup_manager: &BackupManager,
    write_system_hosts: W,
    flush_dns: F,
) -> Result<PersistResult>
where
    W: FnOnce(&str) -> Result<()>,
    F: FnOnce() -> Result<()>,
{
    let backup_warning =
        backup_cmds::create_backup(backup_manager, hosts, Some("Auto-backup before save"))
            .err()
            .map(|err| format!("Backup failed: {}", err));

    let content = if platform.uses_crlf() {
        crate::parser::writer::serialize_hosts_file_crlf(hosts)
    } else {
        parser::serialize_hosts_file(hosts)
    };

    let dns_flush_warning = if hosts.path == platform.hosts_path() {
        write_system_hosts(&content)?;
        flush_dns()
            .err()
            .map(|err| format!("DNS flush failed: {}", err))
    } else {
        fs::write(&hosts.path, content)?;
        None
    };

    Ok(PersistResult {
        backup_warning,
        dns_flush_warning,
    })
}

pub fn import_json(hosts: &mut HostsFile, path: &Path) -> Result<usize> {
    let content = fs::read_to_string(path)?;
    let entries: Vec<ExportEntry> = serde_json::from_str(&content)?;
    let count = entries.len();

    for export_entry in entries {
        entry_cmds::add_entry(
            hosts,
            &export_entry.ip,
            &export_entry.hostnames,
            export_entry.group.as_deref(),
            export_entry.comment.as_deref(),
            export_entry.enabled,
        )?;
    }

    Ok(count)
}

pub fn import_csv(hosts: &mut HostsFile, path: &Path) -> Result<usize> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut count = 0;

    for row in reader.deserialize::<CsvImportEntry>() {
        let row = row?;
        let hostnames: Vec<String> = row.hostnames.split_whitespace().map(String::from).collect();
        let group = (!row.group.trim().is_empty()).then_some(row.group.trim());
        let comment = (!row.comment.trim().is_empty()).then_some(row.comment.trim());

        entry_cmds::add_entry(hosts, &row.ip, &hostnames, group, comment, row.enabled)?;
        count += 1;
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
        new_entry.id = 0;
        new_entry.raw = String::new();
        hosts.add_entry(new_entry, entry.group.as_deref());
    }

    Ok(count)
}
