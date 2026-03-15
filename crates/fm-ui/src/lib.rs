use std::sync::Arc;

use appcui::prelude::*;
use fm_application::FileSystemPort;

pub mod desktop;
pub mod window;

use desktop::MyDesktop;

pub fn run(fs: Arc<dyn FileSystemPort>) -> Result<(), appcui::system::Error> {
    App::new()
        .desktop(MyDesktop::new(fs))
        .app_bar()
        .build()?
        .run();

    Ok(())
}
