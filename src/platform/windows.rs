use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::{Platform, PlatformError};

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn hosts_path(&self) -> PathBuf {
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
        PathBuf::from(system_root)
            .join("System32")
            .join("drivers")
            .join("etc")
            .join("hosts")
    }

    fn config_dir(&self) -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
            .join("hostsbutler")
    }

    fn can_write(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            is_elevated::is_elevated()
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    fn read_hosts(&self) -> Result<String, PlatformError> {
        let content = fs::read_to_string(self.hosts_path())?;
        // Normalize CRLF to LF for internal processing
        Ok(content.replace("\r\n", "\n"))
    }

    fn write_hosts(&self, content: &str) -> Result<(), PlatformError> {
        // Convert LF to CRLF for Windows
        let crlf_content = content.replace('\n', "\r\n");

        if self.can_write() {
            fs::write(self.hosts_path(), &crlf_content)?;
            return Ok(());
        }

        Err(PlatformError::PermissionDenied(
            "run as Administrator to modify hosts file".to_string(),
        ))
    }

    fn flush_dns(&self) -> Result<(), PlatformError> {
        let status = Command::new("ipconfig")
            .arg("/flushdns")
            .status()
            .map_err(|e| PlatformError::DnsFlushFailed(e.to_string()))?;

        if !status.success() {
            return Err(PlatformError::DnsFlushFailed(
                "ipconfig /flushdns failed".to_string(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "Windows"
    }

    fn uses_crlf(&self) -> bool {
        true
    }
}
