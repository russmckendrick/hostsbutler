use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::{Platform, PlatformError};

pub struct MacOsPlatform;

impl Platform for MacOsPlatform {
    fn hosts_path(&self) -> PathBuf {
        PathBuf::from("/etc/hosts")
    }

    fn config_dir(&self) -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("hostsbutler")
    }

    fn can_write(&self) -> bool {
        // Check if we're running as root
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

        // Write to temp file, then use sudo to copy
        let temp = tempfile::NamedTempFile::new()?;
        fs::write(temp.path(), content)?;

        let status = Command::new("sudo")
            .args(["cp", &temp.path().display().to_string(), "/etc/hosts"])
            .status()?;

        if !status.success() {
            return Err(PlatformError::EscalationFailed(
                "sudo cp failed".to_string(),
            ));
        }

        Ok(())
    }

    fn flush_dns(&self) -> Result<(), PlatformError> {
        let status = Command::new("dscacheutil")
            .arg("-flushcache")
            .status()
            .map_err(|e| PlatformError::DnsFlushFailed(e.to_string()))?;

        if !status.success() {
            return Err(PlatformError::DnsFlushFailed(
                "dscacheutil -flushcache failed".to_string(),
            ));
        }

        // Also kill mDNSResponder
        let _ = Command::new("sudo")
            .args(["killall", "-HUP", "mDNSResponder"])
            .status();

        Ok(())
    }

    fn name(&self) -> &str {
        "macOS"
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
