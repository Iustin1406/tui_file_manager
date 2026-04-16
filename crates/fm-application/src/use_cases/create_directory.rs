use std::io;
use std::path::Path;
use std::sync::Arc;

use crate::FileSystemPort;

pub struct CreateDirectoryUseCase {
    fs: Arc<dyn FileSystemPort>,
}

impl CreateDirectoryUseCase {
    pub fn new(fs: Arc<dyn FileSystemPort>) -> Self {
        Self { fs }
    }

    pub fn execute(&self, parent_dir: &Path, name: &str) -> io::Result<std::path::PathBuf> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Directory name cannot be empty",
            ));
        }

        if trimmed == "." || trimmed == ".." {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Directory name cannot be . or ..",
            ));
        }

        if trimmed.contains(std::path::MAIN_SEPARATOR) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Directory name must be a single entry name, not a path",
            ));
        }

        #[cfg(windows)]
        if trimmed.contains('/') || trimmed.contains('\\') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Directory name must not contain path separators",
            ));
        }

        self.fs.create_dir(parent_dir, trimmed)
    }
}
