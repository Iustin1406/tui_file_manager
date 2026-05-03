use std::io;
use std::sync::Arc;

use fm_domain::DriveEntry;

use crate::DrivePort;

pub struct RefreshDriveFolderUseCase {
    drive: Arc<dyn DrivePort>,
}

impl RefreshDriveFolderUseCase {
    pub fn new(drive: Arc<dyn DrivePort>) -> Self {
        Self { drive }
    }

    pub fn execute(&self, folder_id: &str) -> io::Result<Vec<DriveEntry>> {
        if folder_id.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Drive folder id cannot be empty",
            ));
        }

        self.drive.refresh_folder(folder_id)
    }
}
