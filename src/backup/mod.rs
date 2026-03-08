pub mod store;

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use thiserror::Error;

use self::store::BackupMetadata;

const DEFAULT_MAX_BACKUPS: usize = 20;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Backup not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct BackupManager {
    backup_dir: PathBuf,
    max_backups: usize,
}

impl BackupManager {
    pub fn new(config_dir: &Path) -> Self {
        let backup_dir = config_dir.join("backups");
        Self {
            backup_dir,
            max_backups: DEFAULT_MAX_BACKUPS,
        }
    }

    pub fn ensure_dir(&self) -> Result<(), BackupError> {
        migrate_legacy_backup_dir(&self.backup_dir)?;
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir)?;
        }
        Ok(())
    }

    pub fn create_backup(
        &self,
        content: &str,
        description: Option<&str>,
    ) -> Result<BackupMetadata, BackupError> {
        self.ensure_dir()?;

        let now = Utc::now();
        let filename = format!("hosts_{}.bak", now.format("%Y-%m-%d_%H-%M-%S_%f"));
        let backup_path = self.backup_dir.join(&filename);

        fs::write(&backup_path, content)?;

        let metadata = BackupMetadata {
            filename: filename.clone(),
            created_at: now,
            description: description.map(String::from),
            size_bytes: content.len() as u64,
            checksum: compute_checksum(content),
        };

        let meta_path = self.backup_dir.join(format!("{}.meta.json", filename));
        let meta_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(&meta_path, meta_json)?;

        self.rotate()?;

        Ok(metadata)
    }

    pub fn list_backups(&self) -> Result<Vec<BackupMetadata>, BackupError> {
        self.ensure_dir()?;
        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && let Ok(content) = fs::read_to_string(&path)
                && let Ok(meta) = serde_json::from_str::<BackupMetadata>(&content)
                && self.backup_dir.join(&meta.filename).exists()
            {
                backups.push(meta);
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    pub fn restore_backup(&self, filename: &str) -> Result<String, BackupError> {
        let backup_path = self.backup_dir.join(filename);
        if !backup_path.exists() {
            return Err(BackupError::NotFound(filename.to_string()));
        }
        Ok(fs::read_to_string(backup_path)?)
    }

    pub fn delete_backup(&self, filename: &str) -> Result<(), BackupError> {
        let backup_path = self.backup_dir.join(filename);
        let meta_path = self.backup_dir.join(format!("{}.meta.json", filename));

        if backup_path.exists() {
            fs::remove_file(&backup_path)?;
        }
        if meta_path.exists() {
            fs::remove_file(&meta_path)?;
        }

        Ok(())
    }

    fn rotate(&self) -> Result<(), BackupError> {
        let mut backups = self.list_backups()?;

        while backups.len() > self.max_backups {
            if let Some(oldest) = backups.pop() {
                self.delete_backup(&oldest.filename)?;
            }
        }

        Ok(())
    }
}

fn compute_checksum(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(any(target_os = "macos", test))]
fn migrate_dir_if_needed(source: &Path, target: &Path) -> Result<(), BackupError> {
    if source == target || target.exists() || !source.exists() {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, target)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn migrate_legacy_backup_dir(backup_dir: &Path) -> Result<(), BackupError> {
    if backup_dir != current_macos_backup_dir().as_path() {
        return Ok(());
    }

    migrate_dir_if_needed(&legacy_macos_backup_dir(), backup_dir)
}

#[cfg(target_os = "macos")]
fn current_macos_backup_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("hostsbutler")
        .join("backups")
}

#[cfg(target_os = "macos")]
fn legacy_macos_backup_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config")
        .join("hostsbutler")
        .join("backups")
}

#[cfg(not(target_os = "macos"))]
fn migrate_legacy_backup_dir(_backup_dir: &Path) -> Result<(), BackupError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::Utc;

    use super::{BackupManager, store::BackupMetadata};

    #[test]
    fn list_backups_skips_orphaned_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = BackupManager::new(temp_dir.path());
        manager.ensure_dir().unwrap();

        let metadata = BackupMetadata {
            filename: "missing.bak".to_string(),
            created_at: Utc::now(),
            description: Some("orphan".to_string()),
            size_bytes: 12,
            checksum: "checksum".to_string(),
        };

        let meta_path = temp_dir
            .path()
            .join("backups")
            .join("missing.bak.meta.json");
        fs::write(meta_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();

        let backups = manager.list_backups().unwrap();
        assert!(backups.is_empty());
    }

    #[test]
    fn migrate_dir_if_needed_is_noop_when_target_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source_dir = temp_dir.path().join("legacy-backups");
        let target_dir = temp_dir.path().join("backups");

        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(source_dir.join("kept.txt"), "legacy").unwrap();
        fs::write(target_dir.join("current.txt"), "current").unwrap();

        super::migrate_dir_if_needed(&source_dir, &target_dir).unwrap();

        assert!(source_dir.exists());
        assert!(target_dir.exists());
        assert!(target_dir.join("current.txt").exists());
    }

    #[test]
    fn migrate_dir_if_needed_moves_legacy_backups() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source_dir = temp_dir.path().join("legacy-backups");
        let target_dir = temp_dir.path().join("backups");

        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("backup.bak"), "backup").unwrap();

        super::migrate_dir_if_needed(&source_dir, &target_dir).unwrap();

        assert!(!source_dir.exists());
        assert!(target_dir.join("backup.bak").exists());
    }
}
