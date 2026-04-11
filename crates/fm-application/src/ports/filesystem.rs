use fm_domain::FileNode;
use std::io;
use std::path::{Path, PathBuf};

pub trait FileSystemPort: Send + Sync {
    fn current_dir(&self) -> PathBuf;
    fn roots(&self) -> Vec<FileNode>;
    fn list_dir(&self, path: &Path) -> Vec<FileNode>;
    fn exists(&self, path: &Path) -> bool;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn copy_entry(&self, source: &Path, destination_dir: &Path) -> io::Result<PathBuf>;
    fn move_entry(&self, source: &Path, destination_dir: &Path) -> io::Result<PathBuf>;

    fn move_to_trash(&self, path: &Path) -> io::Result<()>;
    fn delete_permanently(&self, path: &Path) -> io::Result<()>;
}
