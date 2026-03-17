use std::path::PathBuf;

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
}
