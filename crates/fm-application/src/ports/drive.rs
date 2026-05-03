use std::io;

use fm_domain::DriveEntry;

pub trait DrivePort: Send + Sync {
    fn list_folder(&self, folder_id: &str) -> io::Result<Vec<DriveEntry>>;
}
