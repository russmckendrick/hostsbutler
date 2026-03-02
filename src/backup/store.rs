use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub filename: String,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub size_bytes: u64,
    pub checksum: String,
}
