use std::path::PathBuf;
use std::time::SystemTime;

use crate::NodeType;

#[derive(Clone, Debug)]
pub struct EntryProperties {
    pub name: String,
    pub path: PathBuf,
    pub node_type: NodeType,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
    pub is_hidden: bool,
}
