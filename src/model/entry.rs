use std::fmt;
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq)]
pub enum EntryStatus {
    Enabled,
    Disabled { comment_prefix: String },
}

impl EntryStatus {
    pub fn is_enabled(&self) -> bool {
        matches!(self, EntryStatus::Enabled)
    }
}

#[derive(Debug, Clone)]
pub struct HostEntry {
    pub id: usize,
    pub status: EntryStatus,
    pub ip: IpAddr,
    pub hostnames: Vec<String>,
    pub inline_comment: Option<String>,
    pub group: Option<String>,
    pub raw: String,
    pub separator: String,
}

impl HostEntry {
    pub fn new(id: usize, ip: IpAddr, hostnames: Vec<String>) -> Self {
        let raw = format!("{}\t{}", ip, hostnames.join(" "));
        Self {
            id,
            status: EntryStatus::Enabled,
            ip,
            hostnames,
            inline_comment: None,
            group: None,
            raw,
            separator: "\t".to_string(),
        }
    }

    pub fn to_line_string(&self) -> String {
        let mut line = String::new();

        if let EntryStatus::Disabled { ref comment_prefix } = self.status {
            line.push_str(comment_prefix);
        }

        line.push_str(&self.ip.to_string());
        line.push_str(&self.separator);
        line.push_str(&self.hostnames.join(" "));

        if let Some(ref comment) = self.inline_comment {
            line.push_str(" # ");
            line.push_str(comment);
        }

        line
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        if let Some(prefix) = query_lower.strip_prefix("ip:") {
            return self.ip.to_string().contains(prefix.trim());
        }
        if let Some(prefix) = query_lower.strip_prefix("host:") {
            let term = prefix.trim();
            return self
                .hostnames
                .iter()
                .any(|h| h.to_lowercase().contains(term));
        }
        if let Some(prefix) = query_lower.strip_prefix("group:") {
            let term = prefix.trim();
            return self
                .group
                .as_ref()
                .is_some_and(|g| g.to_lowercase().contains(term));
        }

        // General search across all fields
        let ip_str = self.ip.to_string();
        if ip_str.contains(&query_lower) {
            return true;
        }
        if self
            .hostnames
            .iter()
            .any(|h| h.to_lowercase().contains(&query_lower))
        {
            return true;
        }
        if self
            .group
            .as_ref()
            .is_some_and(|g| g.to_lowercase().contains(&query_lower))
        {
            return true;
        }
        if self
            .inline_comment
            .as_ref()
            .is_some_and(|c| c.to_lowercase().contains(&query_lower))
        {
            return true;
        }

        false
    }
}

impl fmt::Display for HostEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_line_string())
    }
}
