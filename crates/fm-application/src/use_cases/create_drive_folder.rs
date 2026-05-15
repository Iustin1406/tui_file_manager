use std::io;
use std::sync::Arc;

use fm_domain::DriveEntry;

use crate::DrivePort;

pub struct CreateDriveFolderUseCase {
    drive: Arc<dyn DrivePort>,
}

impl CreateDriveFolderUseCase {
    pub fn new(drive: Arc<dyn DrivePort>) -> Self {
        Self { drive }
    }

    pub fn execute(&self, parent_id: &str, name: &str) -> io::Result<DriveEntry> {
        let trimmed = name.trim();

        if parent_id.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Drive parent folder id cannot be empty",
            ));
        }

        if trimmed.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Folder name cannot be empty",
            ));
        }

        if trimmed == "." || trimmed == ".." {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Folder name cannot be . or ..",
            ));
        }

        self.drive.create_folder(parent_id, trimmed)
    }
}
