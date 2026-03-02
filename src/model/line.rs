use super::entry::HostEntry;

#[derive(Debug, Clone)]
pub enum Line {
    Blank(String),
    Comment(String),
    GroupHeader { raw: String, group_name: String },
    Entry(HostEntry),
}

impl Line {
    pub fn to_line_string(&self) -> String {
        match self {
            Line::Blank(raw) => raw.clone(),
            Line::Comment(raw) => raw.clone(),
            Line::GroupHeader { raw, .. } => raw.clone(),
            Line::Entry(entry) => {
                if entry.raw.is_empty() {
                    entry.to_line_string()
                } else {
                    entry.raw.clone()
                }
            }
        }
    }

    pub fn is_entry(&self) -> bool {
        matches!(self, Line::Entry(_))
    }

    pub fn as_entry(&self) -> Option<&HostEntry> {
        match self {
            Line::Entry(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_entry_mut(&mut self) -> Option<&mut HostEntry> {
        match self {
            Line::Entry(e) => Some(e),
            _ => None,
        }
    }
}
