use std::net::IpAddr;

use dns_lookup::lookup_host;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DnsError {
    #[error("DNS lookup failed for {hostname}: {reason}")]
    LookupFailed { hostname: String, reason: String },
}

#[derive(Debug)]
pub struct DnsResult {
    pub hostname: String,
    pub resolved_ips: Vec<IpAddr>,
    pub hosts_ip: IpAddr,
    pub matches: bool,
}

pub fn test_resolution(hostname: &str, hosts_ip: IpAddr) -> DnsResult {
    match lookup_host(hostname) {
        Ok(ips) => {
            let matches = ips.contains(&hosts_ip);
            DnsResult {
                hostname: hostname.to_string(),
                resolved_ips: ips,
                hosts_ip,
                matches,
            }
        }
        Err(_) => DnsResult {
            hostname: hostname.to_string(),
            resolved_ips: Vec::new(),
            hosts_ip,
            matches: false,
        },
    }
}

pub fn test_entry_resolution(hostnames: &[String], hosts_ip: IpAddr) -> Vec<DnsResult> {
    hostnames
        .iter()
        .map(|h| test_resolution(h, hosts_ip))
        .collect()
}
