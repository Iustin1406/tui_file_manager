use std::io;
use std::path::PathBuf;

use appcui::dialogs;
use appcui::prelude::*;
use appcui::ui::pathfinder::{Flags as PathFinderFlags, PathFinder};
use fm_application::UiDependencies;
use fm_domain::{FileNode, NodeType};

#[derive(Clone, Copy, Eq, PartialEq)]
enum LocalItemType {
    Directory,
    File,
    Root,
}

#[derive(ListItem)]
struct LocalPickerItem {
    #[Column(name: "&Name", width: 200)]
    name: String,

    item_type: LocalItemType,
    full_path: PathBuf,
}

impl From<FileNode> for LocalPickerItem {
    fn from(node: FileNode) -> Self {
        let item_type = match node.node_type {
            NodeType::File => LocalItemType::File,
            NodeType::Directory => LocalItemType::Directory,
            NodeType::Root => LocalItemType::Root,
        };

        Self {
            name: node.name,
            item_type,
            full_path: node.path,
        }
    }
}

#[Window(events = TreeViewEvents<LocalPickerItem> + WindowEvents + PathFinderEvents + ButtonEvents)]
pub struct LocalPickerWindow {
    deps: UiDependencies,
    drive_parent_id: String,

    tree: Handle<TreeView<LocalPickerItem>>,
    path_caption: Handle<Label>,
    path_viewer: Handle<PathFinder>,

    current_path: PathBuf,
    syncing_path: bool,
    initialized: bool,

    back_button: Handle<Button>,
    forward_button: Handle<Button>,

    path_history: Vec<PathBuf>,
    history_index: usize,
}

impl LocalPickerWindow {
    pub fn new_for_upload(deps: UiDependencies, drive_parent_id: String) -> Self {
        let path = deps
            .fs
            .roots()
            .get(0)
            .map(|node| node.path.clone())
            .unwrap_or_else(|| PathBuf::from("/"));

        let mut window = Self {
            base: Window::new(
                "Select file to upload",
                layout!("a:c,w:70%,h:60%"),
                window::Flags::Sizeable,
            ),
            deps,
            drive_parent_id,

            tree: Handle::None,
            path_caption: Handle::None,
            path_viewer: Handle::None,

            current_path: path.clone(),
            syncing_path: false,
            initialized: false,

            back_button: Handle::None,
            forward_button: Handle::None,

            path_history: vec![path.clone()],
            history_index: 0,
        };

        window.back_button = window.add(Button::with_type(
            "<",
            layout!("l:0,t:0,w:3"),
            button::Type::Flat,
        ));

        window.forward_button = window.add(Button::with_type(
            ">",
            layout!("l:4,t:0,w:3"),
            button::Type::Flat,
        ));

        window.path_caption = window.add(Label::new("Path", layout!("l:8,t:0,w:5")));

        let path_viewer = PathFinder::new(
            &path.to_string_lossy(),
            layout!("l:14,t:0,r:1"),
            PathFinderFlags::None,
        );

        window.path_viewer = window.add(path_viewer);

        let tree = TreeView::with_capacity(
            256,
            layout!("l:0,t:1,r:0,b:0"),
            treeview::Flags::HideHeader
                | treeview::Flags::ScrollBars
                | treeview::Flags::SearchBar
                | treeview::Flags::LargeIcons,
        );

        window.tree = window.add(tree);

        window
    }

    fn update_path_viewer(&mut self) {
        let path = self.current_path.clone();
        self.update_path_viewer_to(&path);
    }

    fn update_path_viewer_to(&mut self, path: &PathBuf) {
        let handle = self.path_viewer;

        self.syncing_path = true;

        if let Some(path_viewer) = self.control_mut(handle) {
            if path_viewer.path() != path.as_path() {
                path_viewer.set_path(path);
            }
        }

        self.syncing_path = false;
    }

    fn populate_children(
        &mut self,
        path: &PathBuf,
        parent: Handle<treeview::Item<LocalPickerItem>>,
    ) {
        let nodes = self.deps.fs.list_dir(path);
        let items: Vec<LocalPickerItem> = nodes.into_iter().map(LocalPickerItem::from).collect();

        let tree_handle = self.tree;

        if let Some(tree) = self.control_mut(tree_handle) {
            tree.add_batch(|tree| {
                for item_data in items {
                    match item_data.item_type {
                        LocalItemType::File => {
                            tree.add_item_to_parent(
                                treeview::Item::new(item_data, false, None, ['📄', ' ']),
                                parent,
                            );
                        }

                        LocalItemType::Directory | LocalItemType::Root => {
                            let mut item = treeview::Item::expandable(item_data, true);
                            item.set_icon(['📁', ' ']);
                            tree.add_item_to_parent(item, parent);
                        }
                    }
                }
            });
        }
    }

    fn populate_from_path(&mut self) {
        let current_path = self.current_path.clone();

        let root_name = current_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| current_path.to_string_lossy().to_string());

        let tree_handle = self.tree;

        if let Some(tree) = self.control_mut(tree_handle) {
            tree.clear();
        }

        let root_handle = {
            let mut item = treeview::Item::expandable(
                LocalPickerItem {
                    name: root_name,
                    item_type: LocalItemType::Directory,
                    full_path: current_path.clone(),
                },
                false,
            );

            item.set_icon(['💻', ' ']);

            if let Some(tree) = self.control_mut(tree_handle) {
                tree.add_item(item)
            } else {
                return;
            }
        };

        self.populate_children(&current_path, root_handle);

        if let Some(tree) = self.control_mut(tree_handle) {
            if let Some(root) = tree.item(root_handle) {
                if let Some(first_child) = root.children().first() {
                    tree.move_cursor_to(*first_child);
                } else {
                    tree.move_cursor_to(root_handle);
                }
            }
        }

        self.update_path_viewer();
    }

    fn item_to_path(
        &self,
        item_handle: Handle<treeview::Item<LocalPickerItem>>,
    ) -> Option<PathBuf> {
        let tree = self.control(self.tree)?;
        let item = tree.item(item_handle)?;

        Some(item.value().full_path.clone())
    }

    fn item_is_directory_like(&self, item_handle: Handle<treeview::Item<LocalPickerItem>>) -> bool {
        let Some(tree) = self.control(self.tree) else {
            return false;
        };

        let Some(item) = tree.item(item_handle) else {
            return false;
        };

        matches!(
            item.value().item_type,
            LocalItemType::Directory | LocalItemType::Root
        )
    }

    fn navigate_to_path(&mut self, path: PathBuf) {
        if self.current_path == path {
            return;
        }

        self.current_path = path;
        self.populate_from_path();
    }

    fn push_history(&mut self, path: PathBuf) {
        if self.path_history.get(self.history_index) == Some(&path) {
            return;
        }

        self.path_history.truncate(self.history_index + 1);
        self.path_history.push(path);
        self.history_index = self.path_history.len() - 1;
    }

    fn navigate_to_path_with_history(&mut self, path: PathBuf) {
        if self.current_path == path {
            return;
        }

        self.push_history(path.clone());
        self.navigate_to_path(path);
    }

    fn upload_selected_path(&mut self, local_path: PathBuf) -> io::Result<()> {
        let is_folder = local_path.is_dir();

        if local_path.is_file() {
            self.deps
                .upload_drive_file
                .execute(&local_path, &self.drive_parent_id)?;
        } else if is_folder {
            self.deps
                .upload_drive_folder
                .execute(&local_path, &self.drive_parent_id)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Selected path is neither a file nor a folder",
            ));
        }

        if let Ok(mut pending) = self.deps.pending_drive_refresh.lock() {
            pending.push(self.drive_parent_id.clone());
        }

        if !is_folder {
            dialogs::message("Upload", &format!("Uploaded:\n{}", local_path.display()));
        }

        self.close();

        Ok(())
    }
}

impl WindowEvents for LocalPickerWindow {
    fn on_activate(&mut self) {
        if !self.initialized {
            self.populate_from_path();
            self.initialized = true;
        }
    }
}

impl TreeViewEvents<LocalPickerItem> for LocalPickerWindow {
    fn on_item_expanded(
        &mut self,
        tree_handle: Handle<TreeView<LocalPickerItem>>,
        item_handle: Handle<treeview::Item<LocalPickerItem>>,
        _: bool,
    ) -> EventProcessStatus {
        if let Some(tree) = self.control_mut(tree_handle) {
            tree.clear_search();
            tree.delete_item_children(item_handle);
        }

        if let Some(path) = self.item_to_path(item_handle) {
            self.populate_children(&path, item_handle);
        }

        EventProcessStatus::Processed
    }

    fn on_current_item_changed(
        &mut self,
        _: Handle<TreeView<LocalPickerItem>>,
        item_handle: Handle<treeview::Item<LocalPickerItem>>,
    ) -> EventProcessStatus {
        if self.syncing_path {
            return EventProcessStatus::Ignored;
        }

        if let Some(path) = self.item_to_path(item_handle) {
            self.update_path_viewer_to(&path);
            EventProcessStatus::Processed
        } else {
            EventProcessStatus::Ignored
        }
    }

    // checks if the selected item is a file or folder, and either uploads it or navigates into it
    fn on_item_action(
        &mut self,
        _: Handle<TreeView<LocalPickerItem>>,
        item_handle: Handle<treeview::Item<LocalPickerItem>>,
    ) -> EventProcessStatus {
        let Some(path) = self.item_to_path(item_handle) else {
            return EventProcessStatus::Ignored;
        };

        if self.item_is_directory_like(item_handle) {
            let upload_folder = dialogs::validate(
                "Upload Folder",
                &format!(
                    "Upload this folder recursively?\n\n{}\n\nChoose No to open it.",
                    path.display()
                ),
            );

            if upload_folder {
                if let Err(err) = self.upload_selected_path(path) {
                    dialogs::error("Upload", &err.to_string());
                }

                return EventProcessStatus::Processed;
            }

            self.navigate_to_path_with_history(path);
            return EventProcessStatus::Processed;
        }

        let confirmed = dialogs::validate(
            "Upload to Google Drive",
            &format!("Upload this file?\n{}", path.display()),
        );

        if !confirmed {
            return EventProcessStatus::Processed;
        }

        if let Err(err) = self.upload_selected_path(path) {
            dialogs::error("Upload", &err.to_string());
        }

        EventProcessStatus::Processed
    }
}

impl PathFinderEvents for LocalPickerWindow {
    fn on_path_updated(&mut self, handle: Handle<PathFinder>) -> EventProcessStatus {
        if self.syncing_path {
            return EventProcessStatus::Ignored;
        }

        if handle == self.path_viewer {
            if let Some(path_viewer) = self.control(handle) {
                let new_path = path_viewer.path().to_path_buf();

                if self.deps.fs.exists(&new_path) {
                    self.navigate_to_path_with_history(new_path);
                }
            }
        }

        EventProcessStatus::Processed
    }
}

impl ButtonEvents for LocalPickerWindow {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        if handle == self.back_button {
            if self.history_index > 0 {
                self.history_index -= 1;

                if let Some(path) = self.path_history.get(self.history_index) {
                    self.navigate_to_path(path.clone());
                }
            }

            return EventProcessStatus::Processed;
        }

        if handle == self.forward_button {
            if self.history_index + 1 < self.path_history.len() {
                self.history_index += 1;

                if let Some(path) = self.path_history.get(self.history_index) {
                    self.navigate_to_path(path.clone());
                }
            }

            return EventProcessStatus::Processed;
        }

        EventProcessStatus::Ignored
    }
}
