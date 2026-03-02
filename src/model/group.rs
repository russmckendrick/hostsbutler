#[derive(Debug, Clone, PartialEq)]
pub struct HostGroup {
    pub name: String,
    pub entry_count: usize,
}

impl HostGroup {
    pub fn new(name: String, entry_count: usize) -> Self {
        Self { name, entry_count }
    }
}
