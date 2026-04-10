use std::{io, path::PathBuf};

use appcui::prelude::*;
use appcui::ui::pathfinder::{Flags as PathFinderFlags, PathFinder};
use fm_application::UiDependencies;
use fm_domain::{FileNode, NodeType};

#[derive(Clone, Copy, Eq, PartialEq)]
enum FileSystemType {
    Directory,
    File,
    Root,
}

#[derive(ListItem)]
struct FileSystemItem {
    #[Column(name: "&Name", width: 200)]
    name: String,
    entry_type: FileSystemType,
    full_path: PathBuf,
}

impl From<FileNode> for FileSystemItem {
    fn from(node: FileNode) -> Self {
        let entry_type = match node.node_type {
            NodeType::File => FileSystemType::File,
            NodeType::Directory => FileSystemType::Directory,
            NodeType::Root => FileSystemType::Root,
        };

        Self {
            name: node.name,
            entry_type,
            full_path: node.path,
        }
    }
}

#[Window(events = TreeViewEvents<FileSystemItem> + WindowEvents + PathFinderEvents + ButtonEvents)]
pub struct ExplorerWindow {
    id: u32,
    deps: UiDependencies,
    tree: Handle<TreeView<FileSystemItem>>,
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

impl ExplorerWindow {
    pub fn new(index: u32, deps: UiDependencies) -> Self {
        let path = deps
            .fs
            .roots()
            .get(0)
            .map(|node| node.path.clone())
            .unwrap_or_else(|| PathBuf::from("/"));

        let mut w = Self {
            base: Window::new(
                &format!("Window {}", index),
                layout!("a:c,w:70%,h:60%"),
                window::Flags::Sizeable,
            ),
            id: index,
            deps,
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

        w.back_button = w.add(Button::with_type(
            "<",
            layout!("l:0,t:0,w:3"),
            button::Type::Flat,
        ));

        w.forward_button = w.add(Button::with_type(
            ">",
            layout!("l:4,t:0,w:3"),
            button::Type::Flat,
        ));

        w.path_caption = w.add(Label::new("Path", layout!("l:8,t:0,w:5")));

        let pf = PathFinder::new(
            &path.to_string_lossy(),
            layout!("l:14,t:0,r:1"),
            PathFinderFlags::None,
        );
        w.path_viewer = w.add(pf);

        let tv = TreeView::with_capacity(
            256,
            layout!("l:0,t:1,r:0,b:0"),
            treeview::Flags::HideHeader
                | treeview::Flags::ScrollBars
                | treeview::Flags::SearchBar
                | treeview::Flags::LargeIcons,
        );

        w.tree = w.add(tv);
        w
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    fn mark_as_active(&self) {
        if let Ok(mut guard) = self.deps.active_window_id.lock() {
            *guard = Some(self.id);
        }
    }

    fn update_path_viewer(&mut self) {
        let path_clone = self.current_path.clone();
        self.update_path_viewer_to(&path_clone);
    }

    fn update_path_viewer_to(&mut self, path: &PathBuf) {
        let h = self.path_viewer;

        self.syncing_path = true;
        if let Some(pv) = self.control_mut(h) {
            if pv.path() != path.as_path() {
                pv.set_path(path);
            }
        }
        self.syncing_path = false;
    }

    fn populate_children(
        &mut self,
        path: &PathBuf,
        parent: Handle<treeview::Item<FileSystemItem>>,
    ) {
        let nodes = self.deps.fs.list_dir(path);
        let items: Vec<FileSystemItem> = nodes.into_iter().map(FileSystemItem::from).collect();

        let h = self.tree;
        if let Some(tv) = self.control_mut(h) {
            tv.add_batch(|tv| {
                for item_data in items {
                    match item_data.entry_type {
                        FileSystemType::File => {
                            tv.add_item_to_parent(
                                treeview::Item::new(item_data, false, None, ['\u{1F4C4}', ' ']),
                                parent,
                            );
                        }
                        FileSystemType::Directory | FileSystemType::Root => {
                            let mut item = treeview::Item::expandable(item_data, true);
                            item.set_icon(['📁', ' ']);
                            tv.add_item_to_parent(item, parent);
                        }
                    }
                }
            });
        }
    }

    pub fn current_path(&self) -> &std::path::Path {
        &self.current_path
    }

    pub fn refresh(&mut self) {
        self.populate_from_path();
    }

    fn populate_from_path(&mut self) {
        let current_path = self.current_path.clone();

        let root_name = current_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| current_path.to_string_lossy().to_string());

        let h = self.tree;
        if let Some(tv) = self.control_mut(h) {
            tv.clear();
        }

        let root_handle = {
            let mut item = treeview::Item::expandable(
                FileSystemItem {
                    name: root_name,
                    entry_type: FileSystemType::Directory,
                    full_path: current_path.clone(),
                },
                false,
            );
            item.set_icon(['\u{1F4BB}', ' ']);

            if let Some(tv) = self.control_mut(h) {
                tv.add_item(item)
            } else {
                return;
            }
        };

        self.populate_children(&current_path, root_handle);
        let is_initialized: bool = self.initialized;
        self.syncing_path = true;
        if let Some(tv) = self.control_mut(h) {
            if is_initialized {
                if let Some(root) = tv.item(root_handle) {
                    if let Some(first_child) = root.children().first() {
                        tv.move_cursor_to(*first_child);
                    } else {
                        tv.move_cursor_to(root_handle);
                    }
                }
            } else {
                tv.move_cursor_to(root_handle);
            }
        }
        self.syncing_path = false;

        self.update_path_viewer();
    }

    fn item_to_path(&self, item_handle: Handle<treeview::Item<FileSystemItem>>) -> Option<PathBuf> {
        let tv = self.control(self.tree)?;
        let item = tv.item(item_handle)?;

        Some(item.value().full_path.clone())
    }

    fn item_is_directory_like(&self, item_handle: Handle<treeview::Item<FileSystemItem>>) -> bool {
        let Some(tv) = self.control(self.tree) else {
            return false;
        };

        let Some(item) = tv.item(item_handle) else {
            return false;
        };

        matches!(
            item.value().entry_type,
            FileSystemType::Directory | FileSystemType::Root
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

    fn selected_item_path(&self) -> Option<PathBuf> {
        let tv = self.control(self.tree)?;
        let current_item = tv.current_item()?;
        Some(current_item.value().full_path.clone())
    }

    pub fn copy_selected(&self) -> io::Result<()> {
        let source_path = self
            .selected_item_path()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No item selected"))?;

        self.deps.copy_selection.execute(&source_path)
    }

    pub fn move_selected(&self) -> io::Result<()> {
        let source_path = self
            .selected_item_path()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No item selected"))?;

        self.deps.move_selection.execute(&source_path)
    }

    fn paste_target_directory(&self) -> PathBuf {
        let Some(tv) = self.control(self.tree) else {
            return self.current_path.clone();
        };

        let Some(current_item) = tv.current_item() else {
            return self.current_path.clone();
        };

        let item = current_item.value();

        match item.entry_type {
            FileSystemType::Directory | FileSystemType::Root => item.full_path.clone(),
            FileSystemType::File => self.current_path.clone(),
        }
    }

    pub fn paste_from_clipboard(&mut self) -> io::Result<()> {
        let destination_dir = self.paste_target_directory();

        self.deps.paste_entries.execute(&destination_dir)?;
        self.refresh();

        Ok(())
    }

    pub fn rename_selected_to(&mut self, new_name: &str) -> io::Result<()> {
        let source_path = self
            .selected_item_path()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No item selected"))?;

        let old_current_path = self.current_path.clone();

        let target_path = self.deps.rename_entry.execute(&source_path, new_name)?;

        if source_path == old_current_path {
            self.current_path = target_path;
        }

        self.refresh();

        Ok(())
    }

    pub fn selected_item_name(&self) -> Option<String> {
        let tv = self.control(self.tree)?;
        let current_item = tv.current_item()?;
        Some(current_item.value().name.clone())
    }
}

impl WindowEvents for ExplorerWindow {
    fn on_activate(&mut self) {
        self.mark_as_active();

        if !self.initialized {
            self.populate_from_path();
            self.initialized = true;
        }
    }
}

impl TreeViewEvents<FileSystemItem> for ExplorerWindow {
    fn on_current_item_changed(
        &mut self,
        _: Handle<TreeView<FileSystemItem>>,
        item_handle: Handle<treeview::Item<FileSystemItem>>,
    ) -> EventProcessStatus {
        self.mark_as_active();

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

    fn on_item_expanded(
        &mut self,
        tv: Handle<TreeView<FileSystemItem>>,
        item_handle: Handle<treeview::Item<FileSystemItem>>,
        _: bool,
    ) -> EventProcessStatus {
        self.mark_as_active();

        if let Some(tv) = self.control_mut(tv) {
            tv.clear_search();
            tv.delete_item_children(item_handle);
        }

        if let Some(path) = self.item_to_path(item_handle) {
            self.populate_children(&path, item_handle);
        }

        EventProcessStatus::Processed
    }

    fn on_item_action(
        &mut self,
        _: Handle<TreeView<FileSystemItem>>,
        item_handle: Handle<treeview::Item<FileSystemItem>>,
    ) -> EventProcessStatus {
        self.mark_as_active();

        if self.item_is_directory_like(item_handle) {
            if let Some(path) = self.item_to_path(item_handle) {
                self.navigate_to_path_with_history(path);
                return EventProcessStatus::Processed;
            }
        }

        EventProcessStatus::Ignored
    }
}

impl PathFinderEvents for ExplorerWindow {
    fn on_path_updated(&mut self, handle: Handle<PathFinder>) -> EventProcessStatus {
        self.mark_as_active();

        if self.syncing_path {
            return EventProcessStatus::Ignored;
        }

        if handle == self.path_viewer {
            if let Some(pv) = self.control(handle) {
                let new_path = pv.path().to_path_buf();

                if self.deps.fs.exists(&new_path) {
                    self.navigate_to_path_with_history(new_path);
                }
            }
        }

        EventProcessStatus::Processed
    }
}

impl ButtonEvents for ExplorerWindow {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        self.mark_as_active();

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
