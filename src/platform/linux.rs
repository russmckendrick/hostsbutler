use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::{Platform, PlatformError};

pub struct LinuxPlatform;

impl Platform for LinuxPlatform {
    fn hosts_path(&self) -> PathBuf {
        PathBuf::from("/etc/hosts")
    }

    fn config_dir(&self) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("hostsbutler")
    }

    fn can_write(&self) -> bool {
        libc::geteuid() == 0
    }

    fn read_hosts(&self) -> Result<String, PlatformError> {
        Ok(fs::read_to_string(self.hosts_path())?)
    }

    fn write_hosts(&self, content: &str) -> Result<(), PlatformError> {
        if self.can_write() {
            fs::write(self.hosts_path(), content)?;
            return Ok(());
        }

        // Write to temp file first
        let temp = tempfile::NamedTempFile::new()?;
        fs::write(temp.path(), content)?;

        // Try pkexec first (desktop), fall back to sudo
        let status = Command::new("pkexec")
            .args(["cp", &temp.path().display().to_string(), "/etc/hosts"])
            .status();

        match status {
            Ok(s) if s.success() => return Ok(()),
            _ => {}
        }

        // Fallback to sudo
        let status = Command::new("sudo")
            .args(["cp", &temp.path().display().to_string(), "/etc/hosts"])
            .status()?;

        if !status.success() {
            return Err(PlatformError::EscalationFailed(
                "privilege escalation failed".to_string(),
            ));
        }

        Ok(())
    }

    fn flush_dns(&self) -> Result<(), PlatformError> {
        // Try systemd-resolved first
        let status = Command::new("resolvectl").arg("flush-caches").status();

        match status {
            Ok(s) if s.success() => return Ok(()),
            _ => {}
        }

        // Try systemctl restart
        let status = Command::new("sudo")
            .args(["systemctl", "restart", "systemd-resolved"])
            .status()
            .map_err(|e| PlatformError::DnsFlushFailed(e.to_string()))?;

        if !status.success() {
            return Err(PlatformError::DnsFlushFailed(
                "failed to flush DNS cache".to_string(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "Linux"
    }

    fn uses_crlf(&self) -> bool {
        false
    }
}

mod libc {
    unsafe extern "C" {
        pub safe fn geteuid() -> u32;
    }
}
