use std::path::PathBuf;
use std::process::Command;

use hostsbutler::commands::entry_cmds;
use hostsbutler::commands::file_cmds;
use hostsbutler::model::HostsFile;
use hostsbutler::parser;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load_fixture(name: &str) -> HostsFile {
    let content = std::fs::read_to_string(fixture_path(name)).unwrap();
    parser::parse_hosts_file(&content, fixture_path(name))
}

fn binary_command(home_root: &std::path::Path) -> Command {
    let home = home_root.join("home");
    let xdg = home_root.join("xdg");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&xdg).unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_hostsbutler"));
    command.env("HOME", home);
    command.env("XDG_CONFIG_HOME", xdg);
    command
}

#[test]
fn test_add_edit_delete_cycle() {
    let mut hosts = load_fixture("simple.hosts");
    let initial_count = hosts.entries().len();

    // Add
    let id = entry_cmds::add_entry(
        &mut hosts,
        "192.168.1.100",
        &["test.local".to_string()],
        Some("Testing"),
        Some("test comment"),
        true,
    )
    .unwrap();

    assert_eq!(hosts.entries().len(), initial_count + 1);
    assert!(hosts.dirty);

    // Verify the added entry
    let entry = hosts.find_entry(id).unwrap();
    assert_eq!(entry.ip.to_string(), "192.168.1.100");
    assert_eq!(entry.hostnames, vec!["test.local"]);
    assert_eq!(entry.group.as_deref(), Some("Testing"));
    assert_eq!(entry.inline_comment.as_deref(), Some("test comment"));

    // Edit
    entry_cmds::update_entry(
        &mut hosts,
        id,
        "10.0.0.1",
        &["updated.local".to_string()],
        Some("Updated Group"),
        Some("updated comment"),
        true,
    )
    .unwrap();

    let entry = hosts.find_entry(id).unwrap();
    assert_eq!(entry.ip.to_string(), "10.0.0.1");
    assert_eq!(entry.hostnames, vec!["updated.local"]);
    assert_eq!(entry.group.as_deref(), Some("Updated Group"));
    assert_eq!(entry.inline_comment.as_deref(), Some("updated comment"));

    let serialized = hosts.to_string();
    assert!(serialized.contains("## [Updated Group]"));
    assert!(serialized.contains("10.0.0.1\tupdated.local # updated comment"));

    // Delete
    entry_cmds::delete_entry(&mut hosts, id).unwrap();
    assert_eq!(hosts.entries().len(), initial_count);
}

#[test]
fn test_toggle_entry() {
    let mut hosts = load_fixture("simple.hosts");
    let id = hosts.entries()[0].id;

    assert!(hosts.entries()[0].status.is_enabled());

    // Toggle off
    entry_cmds::toggle_entry(&mut hosts, id).unwrap();
    let entry = hosts.find_entry(id).unwrap();
    assert!(!entry.status.is_enabled());

    // Toggle back on
    entry_cmds::toggle_entry(&mut hosts, id).unwrap();
    let entry = hosts.find_entry(id).unwrap();
    assert!(entry.status.is_enabled());
}

#[test]
fn test_undo_redo() {
    let mut hosts = load_fixture("simple.hosts");
    let initial_count = hosts.entries().len();

    // Add an entry
    entry_cmds::add_entry(
        &mut hosts,
        "10.0.0.1",
        &["test.local".to_string()],
        None,
        None,
        true,
    )
    .unwrap();

    assert_eq!(hosts.entries().len(), initial_count + 1);

    // Undo
    assert!(hosts.undo());
    assert_eq!(hosts.entries().len(), initial_count);

    // Redo
    assert!(hosts.redo());
    assert_eq!(hosts.entries().len(), initial_count + 1);
}

#[test]
fn test_duplicate_detection() {
    let mut hosts = load_fixture("simple.hosts");

    // Add duplicate
    entry_cmds::add_entry(
        &mut hosts,
        "127.0.0.1",
        &["localhost".to_string()],
        None,
        None,
        true,
    )
    .unwrap();

    let duplicates = hosts.find_duplicates();
    assert!(!duplicates.is_empty());
}

#[test]
fn test_search_filtering() {
    let hosts = load_fixture("complex.hosts");
    let entries = hosts.entries();

    // Search by IP
    let matches: Vec<_> = entries
        .iter()
        .filter(|e| e.matches_search("ip:192.168"))
        .collect();
    assert!(!matches.is_empty());

    // Search by hostname
    let matches: Vec<_> = entries
        .iter()
        .filter(|e| e.matches_search("host:dev"))
        .collect();
    assert!(!matches.is_empty());

    // Search by group
    let matches: Vec<_> = entries
        .iter()
        .filter(|e| e.matches_search("group:Work"))
        .collect();
    assert!(!matches.is_empty());

    // General search
    let matches: Vec<_> = entries
        .iter()
        .filter(|e| e.matches_search("jira"))
        .collect();
    assert!(!matches.is_empty());
}

#[test]
fn test_export_json_import_json() {
    let hosts = load_fixture("complex.hosts");
    let temp_dir = tempfile::tempdir().unwrap();
    let export_path = temp_dir.path().join("export.json");

    // Export
    file_cmds::export_json(&hosts, &export_path).unwrap();
    assert!(export_path.exists());

    // Import into new hosts file
    let content = "127.0.0.1\tlocalhost";
    let mut new_hosts = parser::parse_hosts_file(content, PathBuf::from("/etc/hosts"));
    let initial_count = new_hosts.entries().len();

    let imported = file_cmds::import_json(&mut new_hosts, &export_path).unwrap();
    assert!(imported > 0);
    assert!(new_hosts.entries().len() > initial_count);
}

#[test]
fn test_export_csv() {
    let hosts = load_fixture("complex.hosts");
    let temp_dir = tempfile::tempdir().unwrap();
    let export_path = temp_dir.path().join("export.csv");

    file_cmds::export_csv(&hosts, &export_path).unwrap();
    assert!(export_path.exists());

    let content = std::fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("ip,hostnames,enabled,group,comment"));
}

#[test]
fn test_cli_import_json() {
    let temp_dir = tempfile::tempdir().unwrap();
    let hosts_path = temp_dir.path().join("hosts");
    let import_path = temp_dir.path().join("import.json");

    std::fs::write(&hosts_path, "127.0.0.1\tlocalhost\n").unwrap();
    std::fs::write(
        &import_path,
        r#"[
  {
    "ip": "10.0.0.10",
    "hostnames": ["json.test"],
    "enabled": true,
    "group": "Imported",
    "comment": "json import"
  }
]"#,
    )
    .unwrap();

    let output = binary_command(temp_dir.path())
        .args(["--file"])
        .arg(&hosts_path)
        .args(["--import"])
        .arg(&import_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Imported 1 entries"),
        "stdout was: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let content = std::fs::read_to_string(&hosts_path).unwrap();
    assert!(content.contains("## [Imported]"));
    assert!(content.contains("10.0.0.10\tjson.test # json import"));
}

#[test]
fn test_cli_import_csv() {
    let temp_dir = tempfile::tempdir().unwrap();
    let hosts_path = temp_dir.path().join("hosts");
    let import_path = temp_dir.path().join("import.csv");

    std::fs::write(&hosts_path, "127.0.0.1\tlocalhost\n").unwrap();
    std::fs::write(
        &import_path,
        "ip,hostnames,enabled,group,comment\n10.0.0.20,csv.test,false,Imported,csv import\n",
    )
    .unwrap();

    let output = binary_command(temp_dir.path())
        .args(["--file"])
        .arg(&hosts_path)
        .args(["--import"])
        .arg(&import_path)
        .output()
        .unwrap();

    assert!(output.status.success());

    let content = std::fs::read_to_string(&hosts_path).unwrap();
    assert!(content.contains("## [Imported]"));
    assert!(content.contains("# 10.0.0.20\tcsv.test # csv import"));
}

#[test]
fn test_cli_import_hosts() {
    let temp_dir = tempfile::tempdir().unwrap();
    let hosts_path = temp_dir.path().join("hosts");
    let import_path = temp_dir.path().join("import.hosts");

    std::fs::write(&hosts_path, "127.0.0.1\tlocalhost\n").unwrap();
    std::fs::write(
        &import_path,
        "## [Imported]\n10.0.0.30\thosts.test # hosts import\n",
    )
    .unwrap();

    let output = binary_command(temp_dir.path())
        .args(["--file"])
        .arg(&hosts_path)
        .args(["--import"])
        .arg(&import_path)
        .output()
        .unwrap();

    assert!(output.status.success());

    let content = std::fs::read_to_string(&hosts_path).unwrap();
    assert!(content.contains("## [Imported]"));
    assert!(content.contains("10.0.0.30\thosts.test # hosts import"));
}

#[test]
fn test_cli_import_is_blocked_in_readonly_mode() {
    let temp_dir = tempfile::tempdir().unwrap();
    let hosts_path = temp_dir.path().join("hosts");
    let import_path = temp_dir.path().join("import.json");
    let original = "127.0.0.1\tlocalhost\n";

    std::fs::write(&hosts_path, original).unwrap();
    std::fs::write(
        &import_path,
        r#"[{"ip":"10.0.0.40","hostnames":["readonly.test"],"enabled":true,"group":null,"comment":null}]"#,
    )
    .unwrap();

    let output = binary_command(temp_dir.path())
        .args(["--file"])
        .arg(&hosts_path)
        .args(["--readonly", "--import"])
        .arg(&import_path)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--readonly cannot be used with --import")
    );
    assert_eq!(std::fs::read_to_string(&hosts_path).unwrap(), original);
}

#[test]
fn test_cli_import_and_export_are_mutually_exclusive() {
    let temp_dir = tempfile::tempdir().unwrap();
    let hosts_path = temp_dir.path().join("hosts");
    let import_path = temp_dir.path().join("import.json");
    let export_path = temp_dir.path().join("export.json");

    std::fs::write(&hosts_path, "127.0.0.1\tlocalhost\n").unwrap();
    std::fs::write(&import_path, "[]").unwrap();

    let output = binary_command(temp_dir.path())
        .args(["--file"])
        .arg(&hosts_path)
        .args(["--import"])
        .arg(&import_path)
        .args(["--export"])
        .arg(&export_path)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("cannot be used with")
            || String::from_utf8_lossy(&output.stdout).contains("cannot be used with")
    );
}

#[test]
fn test_validation_rejects_invalid_entry() {
    let mut hosts = load_fixture("simple.hosts");

    // Invalid IP
    let result = entry_cmds::add_entry(
        &mut hosts,
        "not-an-ip",
        &["test.local".to_string()],
        None,
        None,
        true,
    );
    assert!(result.is_err());

    // Invalid hostname
    let result = entry_cmds::add_entry(
        &mut hosts,
        "10.0.0.1",
        &["-invalid".to_string()],
        None,
        None,
        true,
    );
    assert!(result.is_err());

    // No hostnames
    let result = entry_cmds::add_entry(&mut hosts, "10.0.0.1", &[], None, None, true);
    assert!(result.is_err());
}
