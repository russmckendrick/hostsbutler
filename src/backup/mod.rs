pub mod store;

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use thiserror::Error;

use self::store::BackupMetadata;

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
            max_backups: 20,
        }
    }

    pub fn ensure_dir(&self) -> Result<(), BackupError> {
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
        let filename = format!("hosts_{}.bak", now.format("%Y-%m-%d_%H-%M-%S"));
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
