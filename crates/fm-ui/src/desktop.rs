use std::sync::Arc;

use appcui::prelude::*;
use appcui::ui::appbar::*;
use fm_application::FileSystemPort;

use crate::window::ExplorerWindow;

#[Desktop(
    events = [MenuEvents, DesktopEvents, AppBarEvents, CommandBarEvents],
    commands = [
        FileOpen,
        FilePreview,
        FileRename,
        FileCopy,
        FileMove,
        FileDelete,
        FileNewDirectory,
        FileProperties,
        WindowNew,
        WindowSplitVertical,
        WindowGridLayout,
        ViewToggleHiddenFiles,
        ViewSortByName,
        ViewSortBySize,
        ViewSortByDate,
        HelpKeybindings,
        HelpAbout,
        Quit
    ]
)]
pub struct MyDesktop {
    fs: Arc<dyn FileSystemPort>,
    next_index: u32,

    menu_file: Handle<MenuButton>,
    menu_window: Handle<MenuButton>,
    menu_view: Handle<MenuButton>,
    menu_help: Handle<MenuButton>,
}

impl MyDesktop {
    pub fn new(fs: Arc<dyn FileSystemPort>) -> Self {
        Self {
            base: Desktop::new(),
            fs,
            next_index: 1,
            menu_file: Handle::None,
            menu_window: Handle::None,
            menu_view: Handle::None,
            menu_help: Handle::None,
        }
    }

    fn add_new_window(&mut self) {
        let index = self.next_index;
        self.next_index += 1;

        let fs = Arc::clone(&self.fs);
        let window = ExplorerWindow::new(index, fs);

        self.add_window(window);
    }

    fn handle_command(&mut self, command: mydesktop::Commands) {
        match command {
            mydesktop::Commands::WindowNew => {
                self.add_new_window();
            }

            mydesktop::Commands::WindowSplitVertical => {
                self.arrange_windows(desktop::ArrangeWindowsMethod::Vertical);
            }
            mydesktop::Commands::WindowGridLayout => {
                self.arrange_windows(desktop::ArrangeWindowsMethod::Grid);
            }

            mydesktop::Commands::FileOpen => {}
            mydesktop::Commands::FilePreview => {}
            mydesktop::Commands::FileRename => {}
            mydesktop::Commands::FileCopy => {}
            mydesktop::Commands::FileMove => {}
            mydesktop::Commands::FileDelete => {}
            mydesktop::Commands::FileNewDirectory => {}
            mydesktop::Commands::FileProperties => {}

            mydesktop::Commands::ViewToggleHiddenFiles => {}
            mydesktop::Commands::ViewSortByName => {}
            mydesktop::Commands::ViewSortBySize => {}
            mydesktop::Commands::ViewSortByDate => {}

            mydesktop::Commands::HelpKeybindings => {}
            mydesktop::Commands::HelpAbout => {}

            mydesktop::Commands::Quit => {
                self.close();
            }
        }
    }
}

impl DesktopEvents for MyDesktop {
    fn on_start(&mut self) {
        self.menu_file = self.appbar().add(MenuButton::new(
            "&File",
            menu!(
                "
                class:MyDesktop,items:[
                    {'&Open',cmd:FileOpen},
                    {'&Preview',cmd:FilePreview},
                    {'&Rename',cmd:FileRename},
                    {'&Copy',cmd:FileCopy},
                    {'&Move',cmd:FileMove},
                    {'&Delete',cmd:FileDelete},
                    {'New &Directory',cmd:FileNewDirectory},
                    {'&Properties',cmd:FileProperties}
                ]
                "
            ),
            0,
            Side::Left,
        ));

        self.menu_window = self.appbar().add(MenuButton::new(
            "&Window",
            menu!(
                "
                class:MyDesktop,items:[
                    {'&New Window',cmd:WindowNew},
                    {'Split &Vertical',cmd:WindowSplitVertical},
                    {'&Grid Layout',cmd:WindowGridLayout}
                ]
                "
            ),
            1,
            Side::Left,
        ));

        self.menu_view = self.appbar().add(MenuButton::new(
            "&View",
            menu!(
                "
                class:MyDesktop,items:[
                    {'Toggle &Hidden Files',cmd:ViewToggleHiddenFiles},
                    {'Sort by &Name',cmd:ViewSortByName},
                    {'Sort by &Size',cmd:ViewSortBySize},
                    {'Sort by &Date',cmd:ViewSortByDate}
                ]
                "
            ),
            2,
            Side::Left,
        ));

        self.menu_help = self.appbar().add(MenuButton::new(
            "&Help",
            menu!(
                "
                class:MyDesktop,items:[
                    {'&Keybindings',cmd:HelpKeybindings},
                    {'&About',cmd:HelpAbout}
                ]
                "
            ),
            3,
            Side::Left,
        ));
    }
}

impl AppBarEvents for MyDesktop {
    fn on_update(&self, appbar: &mut AppBar) {
        appbar.show(self.menu_file);
        appbar.show(self.menu_window);
        appbar.show(self.menu_view);
        appbar.show(self.menu_help);
    }
}

impl CommandBarEvents for MyDesktop {
    fn on_update_commandbar(&self, commandbar: &mut CommandBar) {
        commandbar.set(key!("F2"), "Rename", mydesktop::Commands::FileRename);
        commandbar.set(key!("F5"), "Copy", mydesktop::Commands::FileCopy);
        commandbar.set(key!("F6"), "Move", mydesktop::Commands::FileMove);
        commandbar.set(key!("F7"), "MkDir", mydesktop::Commands::FileNewDirectory);
        commandbar.set(key!("F8"), "Delete", mydesktop::Commands::FileDelete);
        commandbar.set(key!("F10"), "Quit", mydesktop::Commands::Quit);
    }

    fn on_event(&mut self, command_id: mydesktop::Commands) {
        self.handle_command(command_id);
    }
}

impl MenuEvents for MyDesktop {
    fn on_command(
        &mut self,
        _menu: Handle<Menu>,
        _item: Handle<menu::Command>,
        command: mydesktop::Commands,
    ) {
        self.handle_command(command);
    }
}