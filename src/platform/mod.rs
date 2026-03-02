use std::path::PathBuf;

use thiserror::Error;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Privilege escalation failed: {0}")]
    EscalationFailed(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File conflict: hosts file was modified externally")]
    FileConflict,
    #[error("DNS flush failed: {0}")]
    DnsFlushFailed(String),
}

pub trait Platform: Send + Sync {
    fn hosts_path(&self) -> PathBuf;
    fn config_dir(&self) -> PathBuf;
    fn can_write(&self) -> bool;
    fn write_hosts(&self, content: &str) -> Result<(), PlatformError>;
    fn read_hosts(&self) -> Result<String, PlatformError>;
    fn flush_dns(&self) -> Result<(), PlatformError>;
    fn name(&self) -> &str;
    fn uses_crlf(&self) -> bool;
}

pub fn detect_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOsPlatform)
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatform)
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatform)
    }
}
