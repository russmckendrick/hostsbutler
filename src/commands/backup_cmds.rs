use std::path::PathBuf;

use anyhow::Result;

use crate::backup::BackupManager;
use crate::backup::store::BackupMetadata;
use crate::model::HostsFile;
use crate::parser;

pub fn create_backup(
    manager: &BackupManager,
    hosts: &HostsFile,
    description: Option<&str>,
) -> Result<BackupMetadata> {
    let content = parser::serialize_hosts_file(hosts);
    let metadata = manager.create_backup(&content, description)?;
    Ok(metadata)
}

pub fn list_backups(manager: &BackupManager) -> Result<Vec<BackupMetadata>> {
    let backups = manager.list_backups()?;
    Ok(backups)
}

pub fn restore_backup(manager: &BackupManager, filename: &str, path: PathBuf) -> Result<HostsFile> {
    let content = manager.restore_backup(filename)?;
    let hosts = parser::parse_hosts_file(&content, path);
    Ok(hosts)
}

pub fn delete_backup(manager: &BackupManager, filename: &str) -> Result<()> {
    manager.delete_backup(filename)?;
    Ok(())
}
