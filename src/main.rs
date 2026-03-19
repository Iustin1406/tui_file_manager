use std::sync::Arc;

use fm_application::{FileSystemPort, RenameEntryUseCase, UiDependencies};
use fm_infra::StdFileSystem;

fn main() -> Result<(), appcui::system::Error> {
    let fs: Arc<dyn FileSystemPort> = Arc::new(StdFileSystem::default());
    let rename_use_case = Arc::new(RenameEntryUseCase::new(fs.clone()));

    let ui_dependencies = UiDependencies {
        fs,
        rename_entry: rename_use_case,
    };

    fm_ui::run(ui_dependencies)
}
