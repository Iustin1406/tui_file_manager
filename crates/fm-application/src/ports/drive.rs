use fm_domain::DriveEntry;
use std::io;
use std::path::Path;

pub trait DrivePort: Send + Sync {
    // list_folder uses cached data, while refresh_folder fetches updated data from the drive
    fn list_folder(&self, folder_id: &str) -> io::Result<Vec<DriveEntry>>;
    fn refresh_folder(&self, folder_id: &str) -> io::Result<Vec<DriveEntry>>;
    fn create_folder(&self, parent_id: &str, name: &str) -> io::Result<DriveEntry>;
    fn rename_item(&self, file_id: &str, new_name: &str) -> io::Result<DriveEntry>;
    fn upload_file(&self, local_path: &Path, parent_id: &str) -> io::Result<DriveEntry>;
    fn upload_folder(&self, local_folder: &Path, parent_id: &str) -> io::Result<DriveEntry>;
}
