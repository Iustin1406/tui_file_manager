use std::io;

use fm_domain::DriveEntry;

pub trait DrivePort: Send + Sync {
    // list_folder uses cached data, while refresh_folder fetches updated data from the drive
    fn list_folder(&self, folder_id: &str) -> io::Result<Vec<DriveEntry>>;
    fn refresh_folder(&self, folder_id: &str) -> io::Result<Vec<DriveEntry>>;
    fn create_folder(&self, parent_id: &str, name: &str) -> io::Result<DriveEntry>;
}
