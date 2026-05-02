use fm_domain::{EntryProperties, FileNode, PreviewContent};
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

    fn create_dir(&self, parent_dir: &Path, name: &str) -> io::Result<PathBuf>;
    fn get_entry_properties(&self, path: &Path) -> io::Result<EntryProperties>;

    fn open_file(&self, path: &Path) -> io::Result<()>;
    fn preview_entry(
        &self,
        path: &Path,
        max_text_bytes: usize,
        max_image_width: u32,
        max_image_height: u32,
    ) -> io::Result<PreviewContent>;
}
