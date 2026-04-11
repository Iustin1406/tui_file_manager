use std::io;
use std::path::Path;
use std::sync::Arc;

use crate::FileSystemPort;

pub struct MoveToTrashUseCase {
    fs: Arc<dyn FileSystemPort>,
}

impl MoveToTrashUseCase {
    pub fn new(fs: Arc<dyn FileSystemPort>) -> Self {
        Self { fs }
    }

    pub fn execute(&self, path: &Path) -> io::Result<()> {
        if path.as_os_str().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path cannot be empty",
            ));
        }

        self.fs.move_to_trash(path)
    }
}
