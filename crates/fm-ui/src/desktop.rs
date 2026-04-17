use appcui::dialogs;
use appcui::prelude::*;
use appcui::ui::appbar::*;
use fm_application::UiDependencies;

use crate::window::ExplorerWindow;

use chrono::{DateTime, Local};
use std::time::SystemTime;

#[Desktop(
    events = [MenuEvents, DesktopEvents, AppBarEvents, CommandBarEvents],
    commands = [
        FileOpen,
        FilePreview,
        FileRename,
        FileCopy,
        FileMove,
        FilePaste,
        FileMoveToTrash,
        FileDeletePermanently,
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
    deps: UiDependencies,
    next_index: u32,
    explorer_windows: Vec<Handle<ExplorerWindow>>,

    menu_file: Handle<MenuButton>,
    menu_window: Handle<MenuButton>,
    menu_view: Handle<MenuButton>,
    menu_help: Handle<MenuButton>,
}

impl MyDesktop {
    pub fn new(deps: UiDependencies) -> Self {
        Self {
            base: Desktop::new(),
            deps,
            next_index: 1,
            explorer_windows: Vec::new(),
            menu_file: Handle::None,
            menu_window: Handle::None,
            menu_view: Handle::None,
            menu_help: Handle::None,
        }
    }

    fn add_new_window(&mut self) {
        let index = self.next_index;
        self.next_index += 1;

        let window = ExplorerWindow::new(index, self.deps.clone());
        let handle = self.add_window(window);
        self.explorer_windows.push(handle);

        if let Ok(mut guard) = self.deps.active_window_id.lock() {
            *guard = Some(index);
        }
    }

    fn active_explorer_handle(&self) -> Option<Handle<ExplorerWindow>> {
        let active_id = self
            .deps
            .active_window_id
            .lock()
            .ok()
            .and_then(|guard| *guard)?;

        self.explorer_windows.iter().copied().find(|handle| {
            self.windowt(*handle)
                .map(|window| window.id() == active_id)
                .unwrap_or(false)
        })
    }

    fn refresh_all_windows(&mut self) {
        let handles: Vec<Handle<ExplorerWindow>> = self.explorer_windows.to_vec();

        for handle in handles {
            if let Some(window) = self.window_mut(handle) {
                window.refresh();
            }
        }
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
            mydesktop::Commands::FileRename => {
                self.rename_in_active_window();
            }
            mydesktop::Commands::FileCopy => {
                self.copy_in_active_window();
            }
            mydesktop::Commands::FileMove => {
                self.move_in_active_window();
            }
            mydesktop::Commands::FilePaste => {
                self.paste_in_active_window();
            }
            mydesktop::Commands::FileMoveToTrash => {
                self.move_to_trash_in_active_window();
            }
            mydesktop::Commands::FileDeletePermanently => {
                self.delete_permanently_in_active_window();
            }
            mydesktop::Commands::FileNewDirectory => {
                self.create_directory_in_active_window();
            }
            mydesktop::Commands::FileProperties => {
                self.show_properties_in_active_window();
            }

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

    fn copy_in_active_window(&mut self) {
        let Some(explorer_handle) = self.active_explorer_handle() else {
            dialogs::error("Copy", "No active ExplorerWindow");
            return;
        };

        let Some(window) = self.window_mut(explorer_handle) else {
            dialogs::error("Copy", "Active ExplorerWindow handle is invalid");
            return;
        };

        if let Err(err) = window.copy_selected() {
            dialogs::error("Copy", &err.to_string());
        }
    }

    fn move_in_active_window(&mut self) {
        let Some(explorer_handle) = self.active_explorer_handle() else {
            dialogs::error("Move", "No active ExplorerWindow");
            return;
        };

        let Some(window) = self.window_mut(explorer_handle) else {
            dialogs::error("Move", "Active ExplorerWindow handle is invalid");
            return;
        };

        if let Err(err) = window.move_selected() {
            dialogs::error("Move", &err.to_string());
        }
    }

    fn move_to_trash_in_active_window(&mut self) {
        let Some(explorer_handle) = self.active_explorer_handle() else {
            dialogs::error("Move to Trash", "No active ExplorerWindow");
            return;
        };

        let Some(window) = self.window_mut(explorer_handle) else {
            dialogs::error("Move to Trash", "Active ExplorerWindow handle is invalid");
            return;
        };

        let confirmed = dialogs::validate("Move to Trash", "Move selected item to Trash?");

        if !confirmed {
            return;
        }

        if let Err(err) = window.move_selected_to_trash() {
            dialogs::error("Move to Trash", &err.to_string());
            return;
        }

        self.refresh_all_windows();
    }

    fn delete_permanently_in_active_window(&mut self) {
        let Some(explorer_handle) = self.active_explorer_handle() else {
            dialogs::error("Delete Permanently", "No active ExplorerWindow");
            return;
        };

        let Some(window) = self.window_mut(explorer_handle) else {
            dialogs::error(
                "Delete Permanently",
                "Active ExplorerWindow handle is invalid",
            );
            return;
        };

        let confirmed = dialogs::validate(
            "Delete Permanently",
            "Delete selected item permanently?\nThis action cannot be undone.",
        );

        if !confirmed {
            return;
        }

        if let Err(err) = window.delete_selected_permanently() {
            dialogs::error("Delete Permanently", &err.to_string());
            return;
        }

        self.refresh_all_windows();
    }
    fn paste_in_active_window(&mut self) {
        if self.deps.clipboard.is_empty() {
            dialogs::error("Paste", "Clipboard is empty at desktop level");
            return;
        }

        let Some(explorer_handle) = self.active_explorer_handle() else {
            dialogs::error("Paste", "No active ExplorerWindow");
            return;
        };

        let Some(window) = self.window_mut(explorer_handle) else {
            dialogs::error("Paste", "Active ExplorerWindow handle is invalid");
            return;
        };

        if let Err(err) = window.paste_from_clipboard() {
            dialogs::error("Paste", &err.to_string());
            return;
        }

        self.refresh_all_windows();
    }

    fn rename_in_active_window(&mut self) {
        let Some(explorer_handle) = self.active_explorer_handle() else {
            dialogs::error("Rename", "No active ExplorerWindow");
            return;
        };

        let Some(window) = self.window_mut(explorer_handle) else {
            dialogs::error("Rename", "Active ExplorerWindow handle is invalid");
            return;
        };

        let initial_name = window.selected_item_name();
        let result = dialogs::input::<String>(
            "Rename",
            "New name:",
            initial_name,
            Some(validate_rename_input),
        );

        let Some(new_name) = result else {
            return;
        };

        if let Err(err) = window.rename_selected_to(&new_name) {
            dialogs::error("Rename", &err.to_string());
            return;
        }

        self.refresh_all_windows();
    }

    fn show_properties_in_active_window(&mut self) {
        let Some(explorer_handle) = self.active_explorer_handle() else {
            dialogs::error("Properties", "No active ExplorerWindow");
            return;
        };

        let Some(window) = self.window_mut(explorer_handle) else {
            dialogs::error("Properties", "Active ExplorerWindow handle is invalid");
            return;
        };

        let properties = match window.selected_item_properties() {
            Ok(props) => props,
            Err(err) => {
                dialogs::error("Properties", &err.to_string());
                return;
            }
        };

        let entry_type = match properties.node_type {
            fm_domain::NodeType::File => "File",
            fm_domain::NodeType::Directory => "Directory",
            fm_domain::NodeType::Root => "Root",
        };

        let size_text = format_size(properties.size);
        let modified_text = format_modified_time(properties.modified);

        let hidden_text = if properties.is_hidden { "Yes" } else { "No" };

        let message = format!(
            "Name: {}\nPath: {}\nType: {}\nSize: {}\nModified: {}\nHidden: {}",
            properties.name,
            properties.path.display(),
            entry_type,
            size_text,
            modified_text,
            hidden_text,
        );

        dialogs::message("Properties", &message);
    }

    fn create_directory_in_active_window(&mut self) {
        let Some(explorer_handle) = self.active_explorer_handle() else {
            dialogs::error("New Directory", "No active ExplorerWindow");
            return;
        };

        let Some(window) = self.window_mut(explorer_handle) else {
            dialogs::error("New Directory", "Active ExplorerWindow handle is invalid");
            return;
        };

        let result = dialogs::input::<String>(
            "New Directory",
            "Directory name:",
            Some(String::new()),
            Some(validate_new_directory_input),
        );

        let Some(directory_name) = result else {
            return;
        };

        if let Err(err) = window.create_directory(&directory_name) {
            dialogs::error("New Directory", &err.to_string());
            return;
        }

        self.refresh_all_windows();
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
                    {'Mo&ve',cmd:FileMove},
                    {'&Paste',cmd:FilePaste},
                    {'Move to &Trash',cmd:FileMoveToTrash},
                    {'Delete &Permanently',cmd:FileDeletePermanently},
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
        commandbar.set(key!("F7"), "Paste", mydesktop::Commands::FilePaste);
        commandbar.set(key!("F8"), "Delete", mydesktop::Commands::FileMoveToTrash);
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

fn validate_rename_input(value: &String) -> Result<(), String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("New name cannot be empty".to_string());
    }

    if trimmed == "." || trimmed == ".." {
        return Err("New name cannot be . or ..".to_string());
    }

    if trimmed.contains(std::path::MAIN_SEPARATOR) {
        return Err("New name must be a single entry name, not a path".to_string());
    }

    #[cfg(windows)]
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err("New name must not contain path separators".to_string());
    }

    Ok(())
}

fn validate_new_directory_input(value: &String) -> Result<(), String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("Directory name cannot be empty".to_string());
    }

    if trimmed == "." || trimmed == ".." {
        return Err("Directory name cannot be . or ..".to_string());
    }

    if trimmed.contains(std::path::MAIN_SEPARATOR) {
        return Err("Directory name must be a single entry name, not a path".to_string());
    }

    #[cfg(windows)]
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err("Directory name must not contain path separators".to_string());
    }

    Ok(())
}

fn format_modified_time(value: Option<SystemTime>) -> String {
    let Some(system_time) = value else {
        return "N/A".to_string();
    };

    let datetime: DateTime<Local> = DateTime::<Local>::from(system_time);
    datetime.format("%A, %-d %B %Y at %H:%M").to_string()
}

fn format_size(size: Option<u64>) -> String {
    let Some(bytes) = size else {
        return "N/A".to_string();
    };

    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let bytes_f = bytes as f64;

    let readable_format = if bytes_f < KB {
        format!("{} B", bytes)
    } else if bytes_f < MB {
        format!("{:.2} KB", bytes_f / KB)
    } else if bytes_f < GB {
        format!("{:.2} MB", bytes_f / MB)
    } else if bytes_f < TB {
        format!("{:.2} GB", bytes_f / GB)
    } else {
        format!("{:.2} TB", bytes_f / TB)
    };

    format!("{} ({} bytes)", readable_format, bytes)
}
