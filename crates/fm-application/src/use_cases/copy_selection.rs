use std::io;
use std::path::Path;
use std::sync::Arc;

use fm_domain::{ClipboardEntry, ClipboardMode};

use crate::ClipboardState;

pub struct CopySelectionUseCase {
    clipboard: Arc<ClipboardState>,
}

impl CopySelectionUseCase {
    pub fn new(clipboard: Arc<ClipboardState>) -> Self {
        Self { clipboard }
    }

    pub fn execute(&self, source_path: &Path) -> io::Result<()> {
        if source_path.as_os_str().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Source path cannot be empty",
            ));
        }

        let entry = ClipboardEntry {
            source_path: source_path.to_path_buf(),
            mode: ClipboardMode::Copy,
        };

        self.clipboard.set_entries(vec![entry]);

        Ok(())
    }
}
