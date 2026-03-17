use std::path::{Path, PathBuf};

use fm_domain::FileNode;

pub trait FileSystemPort {
    fn current_dir(&self) -> PathBuf;
    fn roots(&self) -> Vec<FileNode>;
    fn list_dir(&self, path: &Path) -> Vec<FileNode>;
    fn exists(&self, path: &Path) -> bool;
}