use std::net::IpAddr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid IP address: {0}")]
    InvalidIp(String),
    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),
    #[error("Hostname too long (max 253 characters): {0}")]
    HostnameTooLong(String),
    #[error("Hostname label too long (max 63 characters): {0}")]
    LabelTooLong(String),
    #[error("Empty hostname")]
    EmptyHostname,
    #[error("No hostnames provided")]
    NoHostnames,
}

pub fn validate_ip(ip_str: &str) -> Result<IpAddr, ValidationError> {
    ip_str
        .parse::<IpAddr>()
        .map_err(|_| ValidationError::InvalidIp(ip_str.to_string()))
}

pub fn validate_hostname(hostname: &str) -> Result<(), ValidationError> {
    if hostname.is_empty() {
        return Err(ValidationError::EmptyHostname);
    }

    if hostname.len() > 253 {
        return Err(ValidationError::HostnameTooLong(hostname.to_string()));
    }

    let labels: Vec<&str> = hostname.split('.').collect();

    for label in &labels {
        if label.is_empty() {
            return Err(ValidationError::InvalidHostname(
                "empty label in hostname".to_string(),
            ));
        }

        if label.len() > 63 {
            return Err(ValidationError::LabelTooLong(label.to_string()));
        }

        if label.starts_with('-') || label.ends_with('-') {
            return Err(ValidationError::InvalidHostname(format!(
                "label '{}' cannot start or end with hyphen",
                label
            )));
        }

        // RFC 1123: alphanumeric and hyphens
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ValidationError::InvalidHostname(format!(
                "label '{}' contains invalid characters",
                label
            )));
        }
    }

    Ok(())
}

pub fn validate_hostnames(hostnames: &[String]) -> Result<(), ValidationError> {
    if hostnames.is_empty() {
        return Err(ValidationError::NoHostnames);
    }
    for hostname in hostnames {
        validate_hostname(hostname)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ipv4() {
        assert!(validate_ip("127.0.0.1").is_ok());
        assert!(validate_ip("192.168.1.1").is_ok());
        assert!(validate_ip("0.0.0.0").is_ok());
        assert!(validate_ip("255.255.255.255").is_ok());
    }

    #[test]
    fn test_valid_ipv6() {
        assert!(validate_ip("::1").is_ok());
        assert!(validate_ip("fe80::1").is_ok());
        assert!(validate_ip("2001:db8::1").is_ok());
    }

    #[test]
    fn test_invalid_ip() {
        assert!(validate_ip("999.999.999.999").is_err());
        assert!(validate_ip("not-an-ip").is_err());
        assert!(validate_ip("").is_err());
    }

    #[test]
    fn test_valid_hostname() {
        assert!(validate_hostname("localhost").is_ok());
        assert!(validate_hostname("my-host.local").is_ok());
        assert!(validate_hostname("sub.domain.example.com").is_ok());
        assert!(validate_hostname("a").is_ok());
    }

    #[test]
    fn test_invalid_hostname() {
        assert!(validate_hostname("").is_err());
        assert!(validate_hostname("-starts-with-dash").is_err());
        assert!(validate_hostname("ends-with-dash-").is_err());
        assert!(validate_hostname("has space").is_err());
        assert!(validate_hostname("has_underscore").is_err());
    }

    #[test]
    fn test_hostname_too_long() {
        let long = "a".repeat(254);
        assert!(validate_hostname(&long).is_err());
    }

    #[test]
    fn test_label_too_long() {
        let long_label = format!("{}.com", "a".repeat(64));
        assert!(validate_hostname(&long_label).is_err());
    }

    #[test]
    fn test_validate_hostnames_empty() {
        assert!(validate_hostnames(&[]).is_err());
    }
}
