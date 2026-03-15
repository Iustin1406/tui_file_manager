use std::sync::Arc;

use fm_infra::StdFileSystem;

fn main() -> Result<(), appcui::system::Error> {
    let fs = Arc::new(StdFileSystem::default());
    fm_ui::run(fs)
}
