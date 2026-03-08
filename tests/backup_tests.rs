use hostsbutler::backup::BackupManager;

#[test]
fn test_backup_create_and_list() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = BackupManager::new(&temp_dir.path().to_path_buf());

    let content = "127.0.0.1\tlocalhost\n::1\tlocalhost";

    // Create backup
    let meta = manager.create_backup(content, Some("Test backup")).unwrap();

    assert!(meta.filename.starts_with("hosts_"));
    assert!(meta.filename.ends_with(".bak"));
    assert_eq!(meta.description.as_deref(), Some("Test backup"));
    assert_eq!(meta.size_bytes, content.len() as u64);

    // List backups
    let backups = manager.list_backups().unwrap();
    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0].filename, meta.filename);
}

#[test]
fn test_backup_restore() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = BackupManager::new(&temp_dir.path().to_path_buf());

    let content = "127.0.0.1\tlocalhost";
    let meta = manager.create_backup(content, None).unwrap();

    let restored = manager.restore_backup(&meta.filename).unwrap();
    assert_eq!(restored, content);
}

#[test]
fn test_backup_filenames_are_unique_for_rapid_creates() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = BackupManager::new(&temp_dir.path().to_path_buf());

    let first = manager.create_backup("first", None).unwrap();
    let second = manager.create_backup("second", None).unwrap();

    assert_ne!(first.filename, second.filename);
}

#[test]
fn test_backup_delete() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = BackupManager::new(&temp_dir.path().to_path_buf());

    let content = "127.0.0.1\tlocalhost";
    let meta = manager.create_backup(content, None).unwrap();

    manager.delete_backup(&meta.filename).unwrap();

    let backups = manager.list_backups().unwrap();
    assert!(backups.is_empty());
}

#[test]
fn test_backup_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = BackupManager::new(&temp_dir.path().to_path_buf());

    let result = manager.restore_backup("nonexistent.bak");
    assert!(result.is_err());
}

#[test]
fn test_multiple_backups_ordered() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = BackupManager::new(&temp_dir.path().to_path_buf());

    manager.create_backup("first", Some("First")).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    manager.create_backup("second", Some("Second")).unwrap();

    let backups = manager.list_backups().unwrap();
    assert_eq!(backups.len(), 2);
    // Most recent first
    assert_eq!(backups[0].description.as_deref(), Some("Second"));
    assert_eq!(backups[1].description.as_deref(), Some("First"));
}

#[test]
fn test_backup_rotation_keeps_latest_twenty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = BackupManager::new(&temp_dir.path().to_path_buf());

    for index in 0..21 {
        manager
            .create_backup(
                &format!("backup-{index}"),
                Some(&format!("Backup {}", index + 1)),
            )
            .unwrap();
    }

    let backups = manager.list_backups().unwrap();
    assert_eq!(backups.len(), 20);
    assert!(
        backups
            .iter()
            .all(|backup| backup.description.as_deref() != Some("Backup 1"))
    );
    assert!(
        backups
            .iter()
            .any(|backup| backup.description.as_deref() == Some("Backup 2"))
    );
    assert!(
        backups
            .iter()
            .any(|backup| backup.description.as_deref() == Some("Backup 21"))
    );
}
