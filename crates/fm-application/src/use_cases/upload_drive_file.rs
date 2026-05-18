use std::io;
use std::path::Path;
use std::sync::Arc;

use fm_domain::DriveEntry;

use crate::DrivePort;

pub struct UploadDriveFileUseCase {
    drive: Arc<dyn DrivePort>,
}

impl UploadDriveFileUseCase {
    pub fn new(drive: Arc<dyn DrivePort>) -> Self {
        Self { drive }
    }

    pub fn execute(&self, local_path: &Path, parent_id: &str) -> io::Result<DriveEntry> {
        if parent_id.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Drive parent folder id cannot be empty",
            ));
        }

        if !local_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Local file does not exist",
            ));
        }

        if !local_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Only file upload is supported for now",
            ));
        }

        self.drive.upload_file(local_path, parent_id)
    }
}
