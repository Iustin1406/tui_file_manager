use appcui::dialogs;
use appcui::prelude::*;
use fm_application::UiDependencies;
use fm_domain::DriveEntry;

#[derive(Clone, Copy, Eq, PartialEq)]
enum DriveItemType {
    Folder,
    File,
}

#[derive(ListItem)]
struct DriveItem {
    #[Column(name: "&Name", width: 200)]
    name: String,

    id: String,
    _mime_type: String,
    item_type: DriveItemType,
}

impl From<DriveEntry> for DriveItem {
    fn from(entry: DriveEntry) -> Self {
        Self {
            name: entry.name,
            id: entry.id,
            _mime_type: entry.mime_type,
            item_type: if entry.is_folder {
                DriveItemType::Folder
            } else {
                DriveItemType::File
            },
        }
    }
}

#[Window(events = WindowEvents + TreeViewEvents<DriveItem> + ButtonEvents)]
pub struct DriveWindow {
    deps: UiDependencies,

    back_button: Handle<Button>,
    forward_button: Handle<Button>,
    location_label: Handle<Label>,
    tree: Handle<TreeView<DriveItem>>,

    current_folder_id: String,
    current_folder_name: String,

    initialized: bool,
    history: Vec<(String, String)>,
    history_index: usize,
}

impl DriveWindow {
    pub fn new(deps: UiDependencies) -> Self {
        let root_id = "root".to_string();
        let root_name = "Google Drive".to_string();

        let mut window = Self {
            base: Window::new(
                "Google Drive",
                layout!("a:c,w:70%,h:60%"),
                window::Flags::Sizeable,
            ),
            deps,

            back_button: Handle::None,
            forward_button: Handle::None,
            location_label: Handle::None,
            tree: Handle::None,

            current_folder_id: root_id.clone(),
            current_folder_name: root_name.clone(),

            initialized: false,
            history: vec![(root_id, root_name)],
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

        window.location_label =
            window.add(Label::new("Drive: Google Drive", layout!("l:8,t:0,r:1")));

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

    fn update_location_label(&mut self) {
        let text = format!("Drive: {}", self.current_folder_name);
        let label_handle = self.location_label;

        if let Some(label) = self.control_mut(label_handle) {
            label.set_caption(&text);
        }
    }

    fn list_folder_items(&self, folder_id: &str) -> Option<Vec<DriveItem>> {
        let entries = match self.deps.list_drive_folder.execute(folder_id) {
            Ok(entries) => entries,
            Err(err) => {
                dialogs::error("Google Drive", &err.to_string());
                return None;
            }
        };

        let mut items: Vec<DriveItem> = entries.into_iter().map(DriveItem::from).collect();

        items.sort_by(|a, b| match (a.item_type, b.item_type) {
            (DriveItemType::Folder, DriveItemType::File) => std::cmp::Ordering::Less,
            (DriveItemType::File, DriveItemType::Folder) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Some(items)
    }

    fn refresh(&mut self) {
        let Some(items) = self.list_folder_items(&self.current_folder_id) else {
            return;
        };

        let tree_handle = self.tree;
        let root_name = self.current_folder_name.clone();
        let root_id = self.current_folder_id.clone();

        if let Some(tree) = self.control_mut(tree_handle) {
            tree.clear();

            let root_item = DriveItem {
                name: root_name,
                id: root_id,
                _mime_type: "application/vnd.google-apps.folder".to_string(),
                item_type: DriveItemType::Folder,
            };

            let mut root = treeview::Item::expandable(root_item, false);
            root.set_icon(['💻', ' ']);

            let root_handle = tree.add_item(root);

            tree.add_batch(|tree| {
                for item in items {
                    match item.item_type {
                        DriveItemType::Folder => {
                            let mut child = treeview::Item::expandable(item, true);
                            child.set_icon(['📁', ' ']);
                            tree.add_item_to_parent(child, root_handle);
                        }

                        DriveItemType::File => {
                            tree.add_item_to_parent(
                                treeview::Item::new(item, false, None, ['📄', ' ']),
                                root_handle,
                            );
                        }
                    }
                }
            });

            if let Some(root) = tree.item(root_handle) {
                if let Some(first_child) = root.children().first() {
                    tree.move_cursor_to(*first_child);
                } else {
                    tree.move_cursor_to(root_handle);
                }
            }
        }

        self.update_location_label();
    }

    fn item_data(
        &self,
        item_handle: Handle<treeview::Item<DriveItem>>,
    ) -> Option<(String, String, DriveItemType)> {
        let tree = self.control(self.tree)?;
        let item = tree.item(item_handle)?;
        let value = item.value();

        Some((value.id.clone(), value.name.clone(), value.item_type))
    }

    fn selected_item(&self) -> Option<(String, String, DriveItemType)> {
        let tree = self.control(self.tree)?;
        let current_item = tree.current_item()?;
        let value = current_item.value();

        Some((value.id.clone(), value.name.clone(), value.item_type))
    }

    fn populate_children(&mut self, folder_id: &str, parent: Handle<treeview::Item<DriveItem>>) {
        let Some(items) = self.list_folder_items(folder_id) else {
            return;
        };

        let tree_handle = self.tree;

        if let Some(tree) = self.control_mut(tree_handle) {
            tree.add_batch(|tree| {
                for item in items {
                    match item.item_type {
                        DriveItemType::Folder => {
                            let mut child = treeview::Item::expandable(item, true);
                            child.set_icon(['📁', ' ']);
                            tree.add_item_to_parent(child, parent);
                        }

                        DriveItemType::File => {
                            tree.add_item_to_parent(
                                treeview::Item::new(item, false, None, ['📄', ' ']),
                                parent,
                            );
                        }
                    }
                }
            });
        }
    }

    fn navigate_to_folder(&mut self, folder_id: String, folder_name: String) {
        if self.current_folder_id == folder_id {
            return;
        }

        self.current_folder_id = folder_id.clone();
        self.current_folder_name = folder_name.clone();

        self.history.truncate(self.history_index + 1);
        self.history.push((folder_id, folder_name));
        self.history_index = self.history.len() - 1;

        self.refresh();
    }

    fn go_back(&mut self) {
        if self.history_index == 0 {
            return;
        }

        self.history_index -= 1;

        if let Some((folder_id, folder_name)) = self.history.get(self.history_index).cloned() {
            self.current_folder_id = folder_id;
            self.current_folder_name = folder_name;
            self.refresh();
        }
    }

    fn go_forward(&mut self) {
        if self.history_index + 1 >= self.history.len() {
            return;
        }

        self.history_index += 1;

        if let Some((folder_id, folder_name)) = self.history.get(self.history_index).cloned() {
            self.current_folder_id = folder_id;
            self.current_folder_name = folder_name;
            self.refresh();
        }
    }
}

impl WindowEvents for DriveWindow {
    fn on_activate(&mut self) {
        if !self.initialized {
            self.refresh();
            self.initialized = true;
        }
    }
}

impl TreeViewEvents<DriveItem> for DriveWindow {
    fn on_item_expanded(
        &mut self,
        tree_handle: Handle<TreeView<DriveItem>>,
        item_handle: Handle<treeview::Item<DriveItem>>,
        _: bool,
    ) -> EventProcessStatus {
        let Some((folder_id, _, item_type)) = self.item_data(item_handle) else {
            return EventProcessStatus::Ignored;
        };

        if item_type != DriveItemType::Folder {
            return EventProcessStatus::Ignored;
        }

        if let Some(tree) = self.control_mut(tree_handle) {
            tree.clear_search();
            tree.delete_item_children(item_handle);
        }

        self.populate_children(&folder_id, item_handle);

        EventProcessStatus::Processed
    }

    fn on_item_action(
        &mut self,
        _: Handle<TreeView<DriveItem>>,
        item_handle: Handle<treeview::Item<DriveItem>>,
    ) -> EventProcessStatus {
        let Some((id, name, item_type)) = self.item_data(item_handle) else {
            return EventProcessStatus::Ignored;
        };

        match item_type {
            DriveItemType::Folder => {
                self.navigate_to_folder(id, name);
                EventProcessStatus::Processed
            }

            DriveItemType::File => {
                dialogs::message("Google Drive", "File actions will be implemented later.");
                EventProcessStatus::Processed
            }
        }
    }
}

impl ButtonEvents for DriveWindow {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        if handle == self.back_button {
            self.go_back();
            return EventProcessStatus::Processed;
        }

        if handle == self.forward_button {
            self.go_forward();
            return EventProcessStatus::Processed;
        }

        EventProcessStatus::Ignored
    }
}
