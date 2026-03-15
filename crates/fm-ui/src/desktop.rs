use std::sync::Arc;

use appcui::prelude::*;
use appcui::ui::appbar::*;
use fm_application::FileSystemPort;

use crate::window::ExplorerWindow;

#[Desktop(events = [MenuEvents, DesktopEvents, AppBarEvents], commands = [ArrangeGrid, ArrangeVertical])]
pub struct MyDesktop {
    fs: Arc<dyn FileSystemPort>,
    next_index: u32,
    btn_add_window: Handle<appbar::Button>,
    menu_arrange: Handle<MenuButton>,
}

impl MyDesktop {
    pub fn new(fs: Arc<dyn FileSystemPort>) -> Self {
        Self {
            base: Desktop::new(),
            fs,
            next_index: 1,
            btn_add_window: Handle::None,
            menu_arrange: Handle::None,
        }
    }
}

impl DesktopEvents for MyDesktop {
    fn on_start(&mut self) {
        self.btn_add_window = self
            .appbar()
            .add(appbar::Button::new("&Add Window", 0, Side::Left));

        self.menu_arrange = self.appbar().add(MenuButton::new(
            "&Arrange",
            menu!(
                "
                class:MyDesktop,items:[
                    {'&Grid',cmd:ArrangeGrid},
                    {'Vertical',cmd:ArrangeVertical}
                ]
                "
            ),
            1,
            Side::Left,
        ));
    }
}

impl MenuEvents for MyDesktop {
    fn on_command(
        &mut self,
        _menu: Handle<Menu>,
        _item: Handle<menu::Command>,
        command: mydesktop::Commands,
    ) {
        match command {
            mydesktop::Commands::ArrangeGrid => {
                self.arrange_windows(desktop::ArrangeWindowsMethod::Grid);
            }
            mydesktop::Commands::ArrangeVertical => {
                self.arrange_windows(desktop::ArrangeWindowsMethod::Vertical);
            }
        }
    }
}

impl AppBarEvents for MyDesktop {
    fn on_update(&self, appbar: &mut AppBar) {
        appbar.show(self.btn_add_window);
        appbar.show(self.menu_arrange);
    }

    fn on_button_click(&mut self, button: Handle<appbar::Button>) {
        if button == self.btn_add_window {
            let index = self.next_index;
            self.next_index += 1;

            let fs = Arc::clone(&self.fs);
            let window = ExplorerWindow::new(index, fs);

            self.add_window(window);
        }
    }
}
