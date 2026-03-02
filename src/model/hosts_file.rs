use std::fmt;
use std::path::PathBuf;

use chrono::{DateTime, Utc};

use super::entry::{EntryStatus, HostEntry};
use super::group::HostGroup;
use super::line::Line;

#[derive(Debug, Clone)]
pub struct HostsFile {
    pub lines: Vec<Line>,
    pub path: PathBuf,
    pub loaded_at: DateTime<Utc>,
    pub original_checksum: String,
    pub dirty: bool,
    pub trailing_newline: bool,
    next_id: usize,
    undo_stack: Vec<Vec<Line>>,
    redo_stack: Vec<Vec<Line>>,
}

impl HostsFile {
    pub fn new(path: PathBuf, lines: Vec<Line>, checksum: String) -> Self {
        let mut next_id = 0;
        for line in &lines {
            if let Line::Entry(entry) = line
                && entry.id >= next_id
            {
                next_id = entry.id + 1;
            }
        }

        Self {
            lines,
            path,
            loaded_at: Utc::now(),
            original_checksum: checksum,
            dirty: false,
            trailing_newline: false,
            next_id,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn save_undo_state(&mut self) {
        self.undo_stack.push(self.lines.clone());
        self.redo_stack.clear();
    }

    pub fn entries(&self) -> Vec<&HostEntry> {
        self.lines.iter().filter_map(|l| l.as_entry()).collect()
    }

    pub fn entries_in_group(&self, group: &str) -> Vec<&HostEntry> {
        self.entries()
            .into_iter()
            .filter(|e| e.group.as_deref() == Some(group))
            .collect()
    }

    pub fn groups(&self) -> Vec<HostGroup> {
        let mut groups: Vec<(String, usize)> = Vec::new();

        for entry in self.entries() {
            let group_name = entry.group.as_deref().unwrap_or("Ungrouped").to_string();
            if let Some(existing) = groups.iter_mut().find(|(name, _)| *name == group_name) {
                existing.1 += 1;
            } else {
                groups.push((group_name, 1));
            }
        }

        groups
            .into_iter()
            .map(|(name, count)| HostGroup::new(name, count))
            .collect()
    }

    pub fn add_entry(&mut self, mut entry: HostEntry, group: Option<&str>) -> usize {
        self.save_undo_state();
        let id = self.next_id();
        entry.id = id;
        entry.group = group.map(String::from);
        entry.raw = String::new(); // Force re-serialization

        if let Some(group_name) = group {
            // Find last entry in the group and insert after it
            let mut insert_pos = None;
            for (i, line) in self.lines.iter().enumerate().rev() {
                match line {
                    Line::Entry(e) if e.group.as_deref() == Some(group_name) => {
                        insert_pos = Some(i + 1);
                        break;
                    }
                    Line::GroupHeader { group_name: gn, .. } if gn == group_name => {
                        insert_pos = Some(i + 1);
                        break;
                    }
                    _ => {}
                }
            }

            if let Some(pos) = insert_pos {
                self.lines.insert(pos, Line::Entry(entry));
            } else {
                // Group doesn't exist yet - create it
                self.lines.push(Line::Blank(String::new()));
                self.lines.push(Line::GroupHeader {
                    raw: format!("## [{}]", group_name),
                    group_name: group_name.to_string(),
                });
                self.lines.push(Line::Entry(entry));
            }
        } else {
            self.lines.push(Line::Entry(entry));
        }

        self.dirty = true;
        id
    }

    pub fn toggle_entry(&mut self, id: usize) -> bool {
        self.save_undo_state();
        for line in &mut self.lines {
            if let Line::Entry(entry) = line
                && entry.id == id
            {
                entry.status = match &entry.status {
                    EntryStatus::Enabled => EntryStatus::Disabled {
                        comment_prefix: "# ".to_string(),
                    },
                    EntryStatus::Disabled { .. } => EntryStatus::Enabled,
                };
                entry.raw = String::new(); // Force re-serialization
                self.dirty = true;
                return true;
            }
        }
        false
    }

    pub fn update_entry(&mut self, id: usize, updated: HostEntry) -> bool {
        self.save_undo_state();
        for line in &mut self.lines {
            if let Line::Entry(entry) = line
                && entry.id == id
            {
                let old_id = entry.id;
                *entry = updated;
                entry.id = old_id;
                entry.raw = String::new(); // Force re-serialization
                self.dirty = true;
                return true;
            }
        }
        false
    }

    pub fn remove_entry(&mut self, id: usize) -> bool {
        self.save_undo_state();
        let len_before = self.lines.len();
        self.lines
            .retain(|l| !matches!(l, Line::Entry(e) if e.id == id));
        if self.lines.len() != len_before {
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn find_entry(&self, id: usize) -> Option<&HostEntry> {
        self.entries().into_iter().find(|e| e.id == id)
    }

    pub fn find_duplicates(&self) -> Vec<(usize, usize)> {
        let entries = self.entries();
        let mut duplicates = Vec::new();

        for (i, a) in entries.iter().enumerate() {
            for b in entries.iter().skip(i + 1) {
                if a.ip == b.ip
                    && a.hostnames
                        .iter()
                        .any(|h| b.hostnames.iter().any(|bh| h == bh))
                {
                    duplicates.push((a.id, b.id));
                }
            }
        }

        duplicates
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.lines.clone());
            self.lines = prev;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.lines.clone());
            self.lines = next;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn clear_undo_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl fmt::Display for HostsFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined = self
            .lines
            .iter()
            .map(|l| l.to_line_string())
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}", joined)?;
        if self.trailing_newline {
            writeln!(f)?;
        }
        Ok(())
    }
}
