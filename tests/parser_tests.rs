use std::path::PathBuf;

use hostsbutler::parser;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load_fixture(name: &str) -> hostsbutler::model::HostsFile {
    let content = std::fs::read_to_string(fixture_path(name)).unwrap();
    parser::parse_hosts_file(&content, fixture_path(name))
}

#[test]
fn test_simple_fixture() {
    let hosts = load_fixture("simple.hosts");
    let entries = hosts.entries();
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].ip.to_string(), "127.0.0.1");
    assert_eq!(entries[0].hostnames, vec!["localhost"]);

    assert_eq!(entries[1].ip.to_string(), "::1");
    assert!(entries[1].ip.is_ipv6());

    assert_eq!(entries[2].ip.to_string(), "255.255.255.255");
    assert_eq!(entries[2].hostnames, vec!["broadcasthost"]);
}

#[test]
fn test_complex_fixture() {
    let hosts = load_fixture("complex.hosts");
    let entries = hosts.entries();

    // Count all entries (enabled + disabled)
    let enabled: Vec<_> = entries.iter().filter(|e| e.status.is_enabled()).collect();
    let disabled: Vec<_> = entries.iter().filter(|e| !e.status.is_enabled()).collect();

    assert!(enabled.len() >= 10);
    assert!(disabled.len() >= 2);

    // Check groups
    let groups = hosts.groups();
    let group_names: Vec<&str> = groups.iter().map(|g| g.name.as_str()).collect();
    assert!(group_names.contains(&"Development"));
    assert!(group_names.contains(&"Production"));
    assert!(group_names.contains(&"Work"));
    assert!(group_names.contains(&"Blocked"));
}

#[test]
fn test_complex_round_trip() {
    let content = std::fs::read_to_string(fixture_path("complex.hosts")).unwrap();
    // Normalise CRLF so the test passes even if Git converted line endings
    let content = content.replace('\r', "");
    let hosts = parser::parse_hosts_file(&content, fixture_path("complex.hosts"));
    let output = parser::serialize_hosts_file(&hosts);
    assert_eq!(output, content);
}

#[test]
fn test_simple_round_trip() {
    let content = std::fs::read_to_string(fixture_path("simple.hosts")).unwrap();
    let content = content.replace('\r', "");
    let hosts = parser::parse_hosts_file(&content, fixture_path("simple.hosts"));
    let output = parser::serialize_hosts_file(&hosts);
    assert_eq!(output, content);
}

#[test]
fn test_malformed_fixture_lenient() {
    // Parser should never fail - malformed lines become comments
    let hosts = load_fixture("malformed.hosts");
    assert!(!hosts.lines.is_empty());

    // Should still parse valid entries
    let entries = hosts.entries();
    assert!(entries.iter().any(|e| e.ip.to_string() == "127.0.0.1"));
}

#[test]
fn test_group_membership() {
    let hosts = load_fixture("complex.hosts");

    let dev_entries = hosts.entries_in_group("Development");
    assert!(dev_entries.len() >= 2);

    for entry in &dev_entries {
        assert_eq!(entry.group.as_deref(), Some("Development"));
    }
}

#[test]
fn test_disabled_entry_detection() {
    let hosts = load_fixture("complex.hosts");
    let entries = hosts.entries();

    // Find the disabled staging entry
    let disabled_staging = entries
        .iter()
        .find(|e| e.hostnames.contains(&"staging.dev.local".to_string()));

    assert!(disabled_staging.is_some());
    assert!(!disabled_staging.unwrap().status.is_enabled());
}

#[test]
fn test_inline_comments() {
    let hosts = load_fixture("complex.hosts");
    let entries = hosts.entries();

    let dev_entry = entries
        .iter()
        .find(|e| e.hostnames.contains(&"dev.local".to_string()));

    assert!(dev_entry.is_some());
    assert!(dev_entry.unwrap().inline_comment.is_some());
}

#[test]
fn test_multiple_hostnames_per_entry() {
    let hosts = load_fixture("complex.hosts");
    let entries = hosts.entries();

    let multi = entries
        .iter()
        .find(|e| e.hostnames.contains(&"confluence.company.com".to_string()));

    assert!(multi.is_some());
    assert!(multi.unwrap().hostnames.len() >= 2);
}
