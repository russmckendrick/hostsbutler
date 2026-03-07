use std::time::Instant;

use anyhow::Context;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::backup::BackupManager;
use crate::backup::store::BackupMetadata;
use crate::commands::{backup_cmds, entry_cmds};
use crate::dns;
use crate::model::HostsFile;
use crate::parser;
use crate::platform::{Platform, detect_platform};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    AddEntry,
    EditEntry(usize),
    ConfirmDelete(usize),
    ConfirmSave,
    BackupManager,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusPanel {
    Groups,
    Table,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub is_error: bool,
    pub created_at: Instant,
}

#[derive(Debug, Clone)]
pub struct EntryForm {
    pub ip: String,
    pub hostnames: String,
    pub group: String,
    pub comment: String,
    pub enabled: bool,
    pub active_field: usize,
    pub error: Option<String>,
}

impl Default for EntryForm {
    fn default() -> Self {
        Self {
            ip: String::new(),
            hostnames: String::new(),
            group: String::new(),
            comment: String::new(),
            enabled: true,
            active_field: 0,
            error: None,
        }
    }
}

impl EntryForm {
    pub fn from_entry(entry: &crate::model::HostEntry) -> Self {
        Self {
            ip: entry.ip.to_string(),
            hostnames: entry.hostnames.join(" "),
            group: entry.group.clone().unwrap_or_default(),
            comment: entry.inline_comment.clone().unwrap_or_default(),
            enabled: entry.status.is_enabled(),
            active_field: 0,
            error: None,
        }
    }

    pub fn active_field_value_mut(&mut self) -> &mut String {
        match self.active_field {
            0 => &mut self.ip,
            1 => &mut self.hostnames,
            2 => &mut self.group,
            3 => &mut self.comment,
            _ => &mut self.ip,
        }
    }

    pub fn field_count(&self) -> usize {
        5 // ip, hostnames, group, comment, enabled
    }
}

pub struct App {
    pub hosts: HostsFile,
    pub mode: AppMode,
    pub focus: FocusPanel,
    pub running: bool,
    pub needs_full_redraw: bool,
    pub selected_entry: usize,
    pub selected_group: usize,
    pub search_query: String,
    pub filtered_entry_ids: Vec<usize>,
    pub groups_list: Vec<String>,
    pub toast: Option<Toast>,
    pub form: EntryForm,
    pub backup_list: Vec<BackupMetadata>,
    pub selected_backup: usize,
    pub platform: Box<dyn Platform>,
    pub backup_manager: BackupManager,
    pub table_scroll_offset: usize,
    pub group_scroll_offset: usize,
    pub dns_results: Vec<dns::DnsResult>,
}

impl App {
    pub fn new(hosts: HostsFile) -> Self {
        let platform = detect_platform();
        Self::new_with_platform(hosts, platform)
    }

    pub fn new_with_platform(hosts: HostsFile, platform: Box<dyn Platform>) -> Self {
        let backup_manager = BackupManager::new(&platform.config_dir());

        let mut app = Self {
            hosts,
            mode: AppMode::Normal,
            focus: FocusPanel::Table,
            running: true,
            needs_full_redraw: false,
            selected_entry: 0,
            selected_group: 0,
            search_query: String::new(),
            filtered_entry_ids: Vec::new(),
            groups_list: Vec::new(),
            toast: None,
            form: EntryForm::default(),
            backup_list: Vec::new(),
            selected_backup: 0,
            platform,
            backup_manager,
            table_scroll_offset: 0,
            group_scroll_offset: 0,
            dns_results: Vec::new(),
        };
        app.refresh_groups();
        app.refresh_filtered_entries();
        app
    }

    pub fn show_toast(&mut self, message: String, is_error: bool) {
        self.toast = Some(Toast {
            message,
            is_error,
            created_at: Instant::now(),
        });
    }

    pub fn clear_stale_toast(&mut self) {
        if let Some(ref toast) = self.toast
            && toast.created_at.elapsed().as_secs() > 3
        {
            self.toast = None;
        }
    }

    pub fn refresh_groups(&mut self) {
        self.groups_list = vec!["All".to_string()];
        for group in self.hosts.groups() {
            if !self.groups_list.contains(&group.name) {
                self.groups_list.push(group.name);
            }
        }
    }

    pub fn refresh_filtered_entries(&mut self) {
        let selected_group = if self.selected_group > 0 {
            self.groups_list.get(self.selected_group).cloned()
        } else {
            None
        };

        self.filtered_entry_ids = self
            .hosts
            .entries()
            .iter()
            .filter(|e| {
                // Group filter
                if let Some(ref group) = selected_group
                    && e.group.as_deref() != Some(group.as_str())
                {
                    return false;
                }

                // Search filter
                if !self.search_query.is_empty() {
                    return e.matches_search(&self.search_query);
                }

                true
            })
            .map(|e| e.id)
            .collect();

        // Clamp selection
        if !self.filtered_entry_ids.is_empty() {
            if self.selected_entry >= self.filtered_entry_ids.len() {
                self.selected_entry = self.filtered_entry_ids.len() - 1;
            }
        } else {
            self.selected_entry = 0;
        }
    }

    pub fn selected_entry_id(&self) -> Option<usize> {
        self.filtered_entry_ids.get(self.selected_entry).copied()
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            AppMode::Normal => self.handle_normal_key(key),
            AppMode::Search => self.handle_search_key(key),
            AppMode::AddEntry => self.handle_form_key(key),
            AppMode::EditEntry(_) => self.handle_form_key(key),
            AppMode::ConfirmDelete(_) => self.handle_confirm_delete_key(key),
            AppMode::ConfirmSave => self.handle_confirm_save_key(key),
            AppMode::BackupManager => self.handle_backup_key(key),
            AppMode::Help => self.handle_help_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.running = false,
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                self.save_file();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => self.request_quit(),
            (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
                if self.hosts.undo() {
                    self.refresh_groups();
                    self.refresh_filtered_entries();
                    self.show_toast("Undo".to_string(), false);
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
                if self.hosts.redo() {
                    self.refresh_groups();
                    self.refresh_filtered_entries();
                    self.show_toast("Redo".to_string(), false);
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => self.reload_file(),
            (_, KeyCode::Char('q')) => self.request_quit(),
            (_, KeyCode::Char('j')) | (_, KeyCode::Down) => self.move_selection_down(),
            (_, KeyCode::Char('k')) | (_, KeyCode::Up) => self.move_selection_up(),
            (_, KeyCode::Char('g')) | (_, KeyCode::Home) => {
                if self.focus == FocusPanel::Table {
                    self.selected_entry = 0;
                    self.table_scroll_offset = 0;
                } else {
                    self.selected_group = 0;
                    self.group_scroll_offset = 0;
                }
            }
            (KeyModifiers::SHIFT, KeyCode::Char('G')) | (_, KeyCode::End) => {
                if self.focus == FocusPanel::Table {
                    if !self.filtered_entry_ids.is_empty() {
                        self.selected_entry = self.filtered_entry_ids.len() - 1;
                    }
                } else if !self.groups_list.is_empty() {
                    self.selected_group = self.groups_list.len() - 1;
                }
            }
            (_, KeyCode::Tab) => {
                self.focus = match self.focus {
                    FocusPanel::Groups => FocusPanel::Table,
                    FocusPanel::Table => FocusPanel::Groups,
                };
            }
            (_, KeyCode::Char(' ')) => {
                if let Some(id) = self.selected_entry_id()
                    && entry_cmds::toggle_entry(&mut self.hosts, id).is_ok()
                {
                    self.refresh_filtered_entries();
                    self.show_toast("Entry toggled".to_string(), false);
                }
            }
            (_, KeyCode::Enter) | (_, KeyCode::Char('e')) => {
                if let Some(id) = self.selected_entry_id()
                    && let Some(entry) = self.hosts.find_entry(id)
                {
                    self.form = EntryForm::from_entry(entry);
                    self.mode = AppMode::EditEntry(id);
                }
            }
            (_, KeyCode::Char('a')) => {
                self.form = EntryForm::default();
                self.mode = AppMode::AddEntry;
            }
            (_, KeyCode::Char('d')) => {
                if let Some(id) = self.selected_entry_id() {
                    self.mode = AppMode::ConfirmDelete(id);
                }
            }
            (_, KeyCode::Char('/')) => {
                self.mode = AppMode::Search;
            }
            (_, KeyCode::Char('b')) => self.open_backup_manager(),
            (_, KeyCode::Char('t')) => {
                if let Some(id) = self.selected_entry_id()
                    && let Some(entry) = self.hosts.find_entry(id)
                {
                    let results = dns::test_entry_resolution(&entry.hostnames, entry.ip);
                    let summary: Vec<String> = results
                        .iter()
                        .map(|r| {
                            if r.matches {
                                format!("{}: OK", r.hostname)
                            } else {
                                format!("{}: MISMATCH", r.hostname)
                            }
                        })
                        .collect();
                    self.dns_results = results;
                    self.show_toast(summary.join(", "), false);
                }
            }
            (_, KeyCode::Char('?')) => {
                self.mode = AppMode::Help;
            }
            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_query.clear();
                self.refresh_filtered_entries();
                self.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.refresh_filtered_entries();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.refresh_filtered_entries();
            }
            _ => {}
        }
    }

    fn handle_form_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Tab => {
                self.form.active_field = (self.form.active_field + 1) % self.form.field_count();
            }
            KeyCode::BackTab => {
                if self.form.active_field == 0 {
                    self.form.active_field = self.form.field_count() - 1;
                } else {
                    self.form.active_field -= 1;
                }
            }
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::NONE) || key.modifiers.is_empty() {
                    self.submit_form();
                }
            }
            KeyCode::Char(' ') if self.form.active_field == 4 => {
                self.form.enabled = !self.form.enabled;
            }
            KeyCode::Backspace => {
                if self.form.active_field < 4 {
                    self.form.active_field_value_mut().pop();
                }
            }
            KeyCode::Char(c) => {
                if self.form.active_field < 4 {
                    self.form.active_field_value_mut().push(c);
                }
            }
            _ => {}
        }
    }

    fn submit_form(&mut self) {
        let hostnames: Vec<String> = self
            .form
            .hostnames
            .split_whitespace()
            .map(String::from)
            .collect();

        let group = if self.form.group.is_empty() {
            None
        } else {
            Some(self.form.group.as_str())
        };

        let comment = if self.form.comment.is_empty() {
            None
        } else {
            Some(self.form.comment.as_str())
        };

        match &self.mode {
            AppMode::AddEntry => {
                match entry_cmds::add_entry(
                    &mut self.hosts,
                    &self.form.ip,
                    &hostnames,
                    group,
                    comment,
                    self.form.enabled,
                ) {
                    Ok(_) => {
                        self.refresh_groups();
                        self.refresh_filtered_entries();
                        self.show_toast("Entry added".to_string(), false);
                        self.mode = AppMode::Normal;
                    }
                    Err(e) => {
                        self.form.error = Some(e.to_string());
                    }
                }
            }
            AppMode::EditEntry(id) => {
                let id = *id;
                match entry_cmds::update_entry(
                    &mut self.hosts,
                    id,
                    &self.form.ip,
                    &hostnames,
                    group,
                    comment,
                    self.form.enabled,
                ) {
                    Ok(_) => {
                        self.refresh_groups();
                        self.refresh_filtered_entries();
                        self.show_toast("Entry updated".to_string(), false);
                        self.mode = AppMode::Normal;
                    }
                    Err(e) => {
                        self.form.error = Some(e.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_confirm_delete_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let AppMode::ConfirmDelete(id) = self.mode
                    && entry_cmds::delete_entry(&mut self.hosts, id).is_ok()
                {
                    self.refresh_groups();
                    self.refresh_filtered_entries();
                    self.show_toast("Entry deleted".to_string(), false);
                }
                self.mode = AppMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_confirm_save_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if self.save_file() {
                    self.running = false;
                }
            }
            KeyCode::Char('n') => {
                self.running = false;
            }
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_backup_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('b') => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_backup < self.backup_list.len().saturating_sub(1) {
                    self.selected_backup += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_backup = self.selected_backup.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('r') => {
                if let Some(backup) = self.backup_list.get(self.selected_backup) {
                    let filename = backup.filename.clone();
                    match self.rollback_to_backup(&filename) {
                        Ok(()) => {
                            self.show_toast("Rolled back to backup".to_string(), false);
                            self.mode = AppMode::Normal;
                        }
                        Err(e) => {
                            self.show_toast(format!("Rollback failed: {}", e), true);
                        }
                    }
                }
            }
            KeyCode::Char('d') => {
                if let Some(backup) = self.backup_list.get(self.selected_backup) {
                    let filename = backup.filename.clone();
                    if backup_cmds::delete_backup(&self.backup_manager, &filename).is_ok() {
                        self.backup_list =
                            backup_cmds::list_backups(&self.backup_manager).unwrap_or_default();
                        if self.selected_backup >= self.backup_list.len() {
                            self.selected_backup = self.backup_list.len().saturating_sub(1);
                        }
                        self.show_toast("Backup deleted".to_string(), false);
                    }
                }
            }
            KeyCode::Char('c') => {
                match backup_cmds::create_backup(
                    &self.backup_manager,
                    &self.hosts,
                    Some("Manual backup"),
                ) {
                    Ok(_) => {
                        self.backup_list =
                            backup_cmds::list_backups(&self.backup_manager).unwrap_or_default();
                        self.show_toast("Backup created".to_string(), false);
                    }
                    Err(e) => {
                        self.show_toast(format!("Backup failed: {}", e), true);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    fn move_selection_down(&mut self) {
        if self.focus == FocusPanel::Table {
            if self.selected_entry < self.filtered_entry_ids.len().saturating_sub(1) {
                self.selected_entry += 1;
            }
        } else if self.selected_group < self.groups_list.len().saturating_sub(1) {
            self.selected_group += 1;
            self.selected_entry = 0;
            self.refresh_filtered_entries();
        }
    }

    fn move_selection_up(&mut self) {
        if self.focus == FocusPanel::Table {
            self.selected_entry = self.selected_entry.saturating_sub(1);
        } else {
            self.selected_group = self.selected_group.saturating_sub(1);
            self.selected_entry = 0;
            self.refresh_filtered_entries();
        }
    }

    fn request_quit(&mut self) {
        if self.hosts.dirty {
            self.mode = AppMode::ConfirmSave;
        } else {
            self.running = false;
        }
    }

    fn save_file(&mut self) -> bool {
        // Auto-backup before save
        if let Err(e) = backup_cmds::create_backup(
            &self.backup_manager,
            &self.hosts,
            Some("Auto-backup before save"),
        ) {
            self.show_toast(format!("Backup failed: {}", e), true);
        }

        let content = if self.platform.uses_crlf() {
            crate::parser::writer::serialize_hosts_file_crlf(&self.hosts)
        } else {
            parser::serialize_hosts_file(&self.hosts)
        };

        match self.write_hosts_content(&content) {
            Ok(()) => {
                self.hosts.dirty = false;
                self.hosts.clear_undo_history();
                self.show_toast("File saved".to_string(), false);
                true
            }
            Err(e) => {
                self.show_toast(format!("Save failed: {}", e), true);
                false
            }
        }
    }

    fn open_backup_manager(&mut self) {
        self.backup_list = backup_cmds::list_backups(&self.backup_manager).unwrap_or_default();
        self.selected_backup = 0;
        self.mode = AppMode::BackupManager;
    }

    fn rollback_to_backup(&mut self, filename: &str) -> anyhow::Result<()> {
        backup_cmds::create_backup(
            &self.backup_manager,
            &self.hosts,
            Some("Auto-backup before rollback"),
        )
        .context("failed to create safety backup")?;

        let path = self.hosts.path.clone();
        let restored = backup_cmds::restore_backup(&self.backup_manager, filename, path)?;
        let content = if self.platform.uses_crlf() {
            crate::parser::writer::serialize_hosts_file_crlf(&restored)
        } else {
            parser::serialize_hosts_file(&restored)
        };

        self.write_hosts_content(&content)?;
        self.hosts = restored;
        self.hosts.dirty = false;
        self.hosts.clear_undo_history();
        self.refresh_groups();
        self.refresh_filtered_entries();
        self.backup_list = backup_cmds::list_backups(&self.backup_manager).unwrap_or_default();
        self.selected_backup = self
            .selected_backup
            .min(self.backup_list.len().saturating_sub(1));

        Ok(())
    }

    fn write_hosts_content(&mut self, content: &str) -> anyhow::Result<()> {
        if self.hosts.path == self.platform.hosts_path() {
            if self.platform.can_write() {
                self.platform.write_hosts(content)?;
            } else {
                let result = crate::tui::suspend(|| self.platform.write_hosts(content));
                self.needs_full_redraw = true;
                result.context("privileged write failed")?;
            }

            return Ok(());
        }

        std::fs::write(&self.hosts.path, content)
            .with_context(|| format!("failed to write {}", self.hosts.path.display()))?;
        Ok(())
    }

    fn reload_file(&mut self) {
        match self.platform.read_hosts() {
            Ok(content) => {
                let path = self.hosts.path.clone();
                self.hosts = parser::parse_hosts_file(&content, path);
                self.refresh_groups();
                self.refresh_filtered_entries();
                self.show_toast("File reloaded".to_string(), false);
            }
            Err(e) => {
                self.show_toast(format!("Reload failed: {}", e), true);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use crossterm::event::KeyCode;

    use super::*;
    use crate::model::{HostEntry, Line};
    use crate::platform::PlatformError;

    struct MockPlatform {
        hosts_path: PathBuf,
        config_dir: PathBuf,
        can_write: bool,
        write_error: Option<String>,
        writes: Mutex<Vec<String>>,
    }

    impl Platform for MockPlatform {
        fn hosts_path(&self) -> PathBuf {
            self.hosts_path.clone()
        }

        fn config_dir(&self) -> PathBuf {
            self.config_dir.clone()
        }

        fn can_write(&self) -> bool {
            self.can_write
        }

        fn write_hosts(&self, content: &str) -> std::result::Result<(), PlatformError> {
            self.writes.lock().unwrap().push(content.to_string());
            if let Some(error) = &self.write_error {
                Err(PlatformError::PermissionDenied(error.clone()))
            } else {
                Ok(())
            }
        }

        fn read_hosts(&self) -> std::result::Result<String, PlatformError> {
            Ok("127.0.0.1 localhost\n".to_string())
        }

        fn flush_dns(&self) -> std::result::Result<(), PlatformError> {
            Ok(())
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn uses_crlf(&self) -> bool {
            false
        }
    }

    fn sample_hosts(path: PathBuf) -> HostsFile {
        let entry = HostEntry::new(
            0,
            "127.0.0.1".parse::<IpAddr>().unwrap(),
            vec!["localhost".to_string()],
        );
        HostsFile::new(path, vec![Line::Entry(entry)], "checksum".to_string())
    }

    #[test]
    fn confirm_save_does_not_exit_when_save_fails() {
        let temp = tempfile::tempdir().unwrap();
        let path = PathBuf::from("/etc/hosts");
        let platform = Box::new(MockPlatform {
            hosts_path: path.clone(),
            config_dir: temp.path().to_path_buf(),
            can_write: true,
            write_error: Some("denied".to_string()),
            writes: Mutex::new(Vec::new()),
        });

        let mut app = App::new_with_platform(sample_hosts(path), platform);
        app.hosts.dirty = true;
        app.mode = AppMode::ConfirmSave;

        app.handle_key(KeyCode::Enter.into());

        assert!(app.running);
        assert_eq!(app.mode, AppMode::ConfirmSave);
        assert!(app.hosts.dirty);
        assert!(
            app.toast
                .as_ref()
                .is_some_and(|toast| toast.message.contains("Save failed"))
        );
    }

    #[test]
    fn confirm_save_exits_after_successful_save() {
        let temp = tempfile::tempdir().unwrap();
        let path = PathBuf::from("/etc/hosts");
        let platform = Box::new(MockPlatform {
            hosts_path: path.clone(),
            config_dir: temp.path().to_path_buf(),
            can_write: true,
            write_error: None,
            writes: Mutex::new(Vec::new()),
        });

        let mut app = App::new_with_platform(sample_hosts(path), platform);
        app.hosts.dirty = true;
        app.mode = AppMode::ConfirmSave;

        app.handle_key(KeyCode::Char('y').into());

        assert!(!app.running);
        assert_eq!(app.mode, AppMode::ConfirmSave);
        assert!(!app.hosts.dirty);
    }

    #[test]
    fn save_uses_overridden_hosts_path_directly() {
        let temp = tempfile::tempdir().unwrap();
        let custom_hosts = temp.path().join("hosts");
        let platform = Box::new(MockPlatform {
            hosts_path: PathBuf::from("/etc/hosts"),
            config_dir: temp.path().to_path_buf(),
            can_write: true,
            write_error: None,
            writes: Mutex::new(Vec::new()),
        });

        let mut app = App::new_with_platform(sample_hosts(custom_hosts.clone()), platform);
        app.hosts.dirty = true;

        assert!(app.save_file());
        assert!(custom_hosts.exists());
        assert!(
            std::fs::read_to_string(&custom_hosts)
                .unwrap()
                .contains("127.0.0.1")
        );
    }

    #[test]
    fn backup_manager_rolls_back_selected_backup_to_disk() {
        let temp = tempfile::tempdir().unwrap();
        let custom_hosts = temp.path().join("hosts");
        let platform = Box::new(MockPlatform {
            hosts_path: PathBuf::from("/etc/hosts"),
            config_dir: temp.path().to_path_buf(),
            can_write: true,
            write_error: None,
            writes: Mutex::new(Vec::new()),
        });

        let mut app = App::new_with_platform(sample_hosts(custom_hosts.clone()), platform);
        app.backup_manager
            .create_backup("10.0.0.1\trollback.test\n", Some("Known good"))
            .unwrap();

        app.handle_key(KeyCode::Char('b').into());
        app.handle_key(KeyCode::Enter.into());

        assert_eq!(app.mode, AppMode::Normal);
        assert!(!app.hosts.dirty);
        assert_eq!(app.hosts.entries()[0].ip.to_string(), "10.0.0.1");
        assert_eq!(app.hosts.entries()[0].hostnames, vec!["rollback.test"]);
        assert_eq!(
            std::fs::read_to_string(&custom_hosts).unwrap(),
            "10.0.0.1\trollback.test\n"
        );

        let backups = app.backup_manager.list_backups().unwrap();
        assert_eq!(backups.len(), 2);
        assert!(
            backups
                .iter()
                .any(|backup| backup.description.as_deref() == Some("Auto-backup before rollback"))
        );
    }
}
