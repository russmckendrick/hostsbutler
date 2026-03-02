use hostsbutler::validation::{validate_hostname, validate_hostnames, validate_ip};

#[test]
fn test_loopback_addresses() {
    assert!(validate_ip("127.0.0.1").is_ok());
    assert!(validate_ip("::1").is_ok());
}

#[test]
fn test_private_ranges() {
    assert!(validate_ip("10.0.0.1").is_ok());
    assert!(validate_ip("172.16.0.1").is_ok());
    assert!(validate_ip("192.168.0.1").is_ok());
}

#[test]
fn test_blocking_address() {
    assert!(validate_ip("0.0.0.0").is_ok());
}

#[test]
fn test_ipv6_forms() {
    assert!(validate_ip("::1").is_ok());
    assert!(validate_ip("fe80::1").is_ok());
    assert!(validate_ip("2001:0db8:85a3:0000:0000:8a2e:0370:7334").is_ok());
    assert!(validate_ip("2001:db8::1").is_ok());
}

#[test]
fn test_hostname_single_label() {
    assert!(validate_hostname("localhost").is_ok());
    assert!(validate_hostname("myhost").is_ok());
}

#[test]
fn test_hostname_multi_label() {
    assert!(validate_hostname("dev.local").is_ok());
    assert!(validate_hostname("sub.domain.example.com").is_ok());
    assert!(validate_hostname("a.b.c.d.e").is_ok());
}

#[test]
fn test_hostname_with_numbers() {
    assert!(validate_hostname("host1").is_ok());
    assert!(validate_hostname("192host").is_ok());
    assert!(validate_hostname("my-host-2").is_ok());
}

#[test]
fn test_hostname_invalid_chars() {
    assert!(validate_hostname("host_name").is_err()); // underscore
    assert!(validate_hostname("host name").is_err()); // space
    assert!(validate_hostname("host@name").is_err()); // special char
}

#[test]
fn test_hostname_hyphen_rules() {
    assert!(validate_hostname("-startdash").is_err());
    assert!(validate_hostname("enddash-").is_err());
    assert!(validate_hostname("mid-dash").is_ok());
}

#[test]
fn test_hostname_max_length() {
    let long = format!("{}.com", "a".repeat(250));
    assert!(validate_hostname(&long).is_err());
}

#[test]
fn test_validate_hostnames_multiple() {
    let hostnames = vec!["localhost".to_string(), "myhost.local".to_string()];
    assert!(validate_hostnames(&hostnames).is_ok());
}

#[test]
fn test_validate_hostnames_one_invalid() {
    let hostnames = vec!["localhost".to_string(), "-invalid".to_string()];
    assert!(validate_hostnames(&hostnames).is_err());
}
