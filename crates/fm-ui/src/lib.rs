use appcui::prelude::*;
use fm_application::UiDependencies;

pub mod desktop;
pub mod drive_window;
pub mod local_picker_window;
pub mod preview_window;
pub mod window;

use desktop::MyDesktop;

pub fn run(deps: UiDependencies) -> Result<(), appcui::system::Error> {
    App::new()
        .desktop(MyDesktop::new(deps))
        .app_bar()
        .command_bar()
        .build()?
        .run();

    Ok(())
}
