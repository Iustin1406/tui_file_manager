use std::io;
use std::path::Path;
use std::sync::Arc;

use fm_domain::ClipboardMode;

use crate::{ClipboardState, FileSystemPort};

pub struct PasteEntriesUseCase {
    fs: Arc<dyn FileSystemPort>,
    clipboard: Arc<ClipboardState>,
}

impl PasteEntriesUseCase {
    pub fn new(fs: Arc<dyn FileSystemPort>, clipboard: Arc<ClipboardState>) -> Self {
        Self { fs, clipboard }
    }

    pub fn execute(&self, destination_dir: &Path) -> io::Result<Vec<std::path::PathBuf>> {
        if !destination_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Destination directory does not exist",
            ));
        }

        let entries = self.clipboard.get_entries();

        if entries.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Clipboard is empty",
            ));
        }

        let mut results = Vec::new();

        for entry in entries {
            match entry.mode {
                ClipboardMode::Copy => {
                    let result_path = self.fs.copy_entry(&entry.source_path, destination_dir)?;
                    results.push(result_path);
                }

                ClipboardMode::Move => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Move not implemented yet",
                    ));
                }
            }
        }

        Ok(results)
    }
}
