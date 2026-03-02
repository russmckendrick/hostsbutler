use crate::model::{EntryStatus, HostEntry, HostsFile};
use crate::validation;

use anyhow::Result;

pub fn add_entry(
    hosts: &mut HostsFile,
    ip_str: &str,
    hostnames: &[String],
    group: Option<&str>,
    comment: Option<&str>,
    enabled: bool,
) -> Result<usize> {
    let ip = validation::validate_ip(ip_str)?;
    validation::validate_hostnames(hostnames)?;

    let mut entry = HostEntry::new(0, ip, hostnames.to_vec());
    entry.inline_comment = comment.map(String::from);

    if !enabled {
        entry.status = EntryStatus::Disabled {
            comment_prefix: "# ".to_string(),
        };
    }

    let id = hosts.add_entry(entry, group);
    Ok(id)
}

pub fn toggle_entry(hosts: &mut HostsFile, id: usize) -> Result<()> {
    if hosts.toggle_entry(id) {
        Ok(())
    } else {
        anyhow::bail!("Entry with id {} not found", id)
    }
}

pub fn update_entry(
    hosts: &mut HostsFile,
    id: usize,
    ip_str: &str,
    hostnames: &[String],
    comment: Option<&str>,
    enabled: bool,
) -> Result<()> {
    let ip = validation::validate_ip(ip_str)?;
    validation::validate_hostnames(hostnames)?;

    let mut updated = HostEntry::new(id, ip, hostnames.to_vec());
    updated.inline_comment = comment.map(String::from);
    if !enabled {
        updated.status = EntryStatus::Disabled {
            comment_prefix: "# ".to_string(),
        };
    }

    if hosts.update_entry(id, updated) {
        Ok(())
    } else {
        anyhow::bail!("Entry with id {} not found", id)
    }
}

pub fn delete_entry(hosts: &mut HostsFile, id: usize) -> Result<()> {
    if hosts.remove_entry(id) {
        Ok(())
    } else {
        anyhow::bail!("Entry with id {} not found", id)
    }
}
