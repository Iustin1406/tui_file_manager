use appcui::prelude::*;
use fm_application::UiDependencies;

pub mod desktop;
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
