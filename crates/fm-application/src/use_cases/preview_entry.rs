use std::io;
use std::path::Path;
use std::sync::Arc;

use fm_domain::PreviewContent;

use crate::FileSystemPort;

pub struct PreviewEntryUseCase {
    fs: Arc<dyn FileSystemPort>,
}

impl PreviewEntryUseCase {
    pub fn new(fs: Arc<dyn FileSystemPort>) -> Self {
        Self { fs }
    }

    pub fn execute(&self, path: &Path) -> io::Result<PreviewContent> {
        if path.as_os_str().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path cannot be empty",
            ));
        }

        const MAX_TEXT_BYTES: usize = 128 * 1024;
        const MAX_IMAGE_WIDTH: u32 = 120;
        const MAX_IMAGE_HEIGHT: u32 = 40;

        self.fs
            .preview_entry(path, MAX_TEXT_BYTES, MAX_IMAGE_WIDTH, MAX_IMAGE_HEIGHT)
    }
}
