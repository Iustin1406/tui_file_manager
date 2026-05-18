use std::io;
use std::path::Path;
use std::sync::Arc;

use fm_domain::DriveEntry;

use crate::DrivePort;

pub struct UploadDriveFolderUseCase {
    drive: Arc<dyn DrivePort>,
}

impl UploadDriveFolderUseCase {
    pub fn new(drive: Arc<dyn DrivePort>) -> Self {
        Self { drive }
    }

    pub fn execute(&self, local_folder: &Path, parent_id: &str) -> io::Result<DriveEntry> {
        if parent_id.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Drive parent folder id cannot be empty",
            ));
        }

        if !local_folder.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Local folder does not exist",
            ));
        }

        if !local_folder.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Selected path is not a folder",
            ));
        }

        self.drive.upload_folder(local_folder, parent_id)
    }
}
