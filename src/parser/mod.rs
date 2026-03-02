pub mod reader;
pub mod writer;

pub use reader::parse_hosts_file;
pub use writer::serialize_hosts_file;
