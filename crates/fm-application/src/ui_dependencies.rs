use std::sync::Arc;

use crate::{FileSystemPort, RenameEntryUseCase};

#[derive(Clone)]
pub struct UiDependencies {
    pub fs: Arc<dyn FileSystemPort>,
    pub rename_entry: Arc<RenameEntryUseCase>,
}
