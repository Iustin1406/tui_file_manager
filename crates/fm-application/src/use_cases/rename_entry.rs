use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::ports::filesystem::FileSystemPort;

pub struct RenameEntryUseCase {
    fs: Arc<dyn FileSystemPort>,
}

impl RenameEntryUseCase {
    pub fn new(fs: Arc<dyn FileSystemPort>) -> Self {
        Self { fs }
    }

    pub fn execute(&self, source_path: &Path, new_name: &str) -> io::Result<PathBuf> {
        let trimmed_name = new_name.trim();

        if trimmed_name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "New name cannot be empty",
            ));
        }

        if trimmed_name == "." || trimmed_name == ".." {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "New name cannot be . or ..",
            ));
        }

        // Ensure the new name does not contain path separators
        if trimmed_name.contains(std::path::MAIN_SEPARATOR) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "New name must be a single entry name, not a path",
            ));
        }

        // On Windows, also check for both types of separators
        #[cfg(windows)]
        if trimmed_name.contains('/') || trimmed_name.contains('\\') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "New name must not contain path separators",
            ));
        }

        let parent = source_path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Entry has no parent directory")
        })?;

        let target_path = parent.join(trimmed_name);

        if target_path == source_path {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "New name is the same as the current name",
            ));
        }

        if self.fs.exists(&target_path) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "An entry with this name already exists",
            ));
        }

        self.fs.rename(source_path, &target_path)?;

        Ok(target_path)
    }
}
