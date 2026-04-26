use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeType {
    File,
    Directory,
    Root,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub node_type: NodeType,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
    pub is_hidden: bool,
}
