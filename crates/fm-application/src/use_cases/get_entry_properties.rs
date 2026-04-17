use std::io;
use std::path::Path;
use std::sync::Arc;

use fm_domain::EntryProperties;

use crate::FileSystemPort;

pub struct GetEntryPropertiesUseCase {
    fs: Arc<dyn FileSystemPort>,
}

impl GetEntryPropertiesUseCase {
    pub fn new(fs: Arc<dyn FileSystemPort>) -> Self {
        Self { fs }
    }

    pub fn execute(&self, path: &Path) -> io::Result<EntryProperties> {
        if path.as_os_str().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path cannot be empty",
            ));
        }

        self.fs.get_entry_properties(path)
    }
}
