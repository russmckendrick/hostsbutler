use crate::model::HostsFile;

/// Serialize a HostsFile back to string.
/// Unmodified lines use their original raw text.
/// Modified entries (raw is empty) are re-serialized.
pub fn serialize_hosts_file(hosts: &HostsFile) -> String {
    hosts.to_string()
}

/// Serialize with Windows-style line endings.
pub fn serialize_hosts_file_crlf(hosts: &HostsFile) -> String {
    let mut content = hosts
        .lines
        .iter()
        .map(|l| l.to_line_string())
        .collect::<Vec<_>>()
        .join("\r\n");

    if hosts.trailing_newline {
        content.push_str("\r\n");
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::reader::parse_hosts_file;
    use std::path::PathBuf;

    #[test]
    fn test_serialize_round_trip() {
        let content = "# /etc/hosts\n127.0.0.1\tlocalhost\n::1\tlocalhost\n\n## [Dev]\n192.168.1.10\tdev.local";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        let output = serialize_hosts_file(&hosts);
        assert_eq!(output, content);
    }

    #[test]
    fn test_serialize_crlf() {
        let content = "127.0.0.1\tlocalhost\n::1\tlocalhost";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        let output = serialize_hosts_file_crlf(&hosts);
        assert_eq!(output, "127.0.0.1\tlocalhost\r\n::1\tlocalhost");
    }

    #[test]
    fn test_serialize_crlf_preserves_trailing_newline() {
        let content = "127.0.0.1\tlocalhost\n::1\tlocalhost\n";
        let hosts = parse_hosts_file(content, PathBuf::from("/etc/hosts"));
        let output = serialize_hosts_file_crlf(&hosts);
        assert_eq!(output, "127.0.0.1\tlocalhost\r\n::1\tlocalhost\r\n");
    }
}
