use std::io;
use std::sync::Arc;

use fm_domain::DriveEntry;

use crate::DrivePort;

pub struct RenameDriveItemUseCase {
    drive: Arc<dyn DrivePort>,
}

impl RenameDriveItemUseCase {
    pub fn new(drive: Arc<dyn DrivePort>) -> Self {
        Self { drive }
    }

    pub fn execute(&self, file_id: &str, new_name: &str) -> io::Result<DriveEntry> {
        let trimmed = new_name.trim();

        if file_id.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Drive item id cannot be empty",
            ));
        }

        if trimmed.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "New name cannot be empty",
            ));
        }

        if trimmed == "." || trimmed == ".." {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "New name cannot be . or ..",
            ));
        }

        self.drive.rename_item(file_id, trimmed)
    }
}
