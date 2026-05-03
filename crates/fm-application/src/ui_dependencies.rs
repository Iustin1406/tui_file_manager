use std::sync::{Arc, Mutex};

use crate::{
    ClipboardState, CopySelectionUseCase, CreateDirectoryUseCase, DeletePermanentlyUseCase,
    FileSystemPort, GetEntryPropertiesUseCase, ListDriveFolderUseCase, MoveSelectionUseCase,
    MoveToTrashUseCase, OpenEntryUseCase, PasteEntriesUseCase, PreviewEntryUseCase,
    RenameEntryUseCase,
};

#[derive(Clone)]
pub struct UiDependencies {
    pub fs: Arc<dyn FileSystemPort>,
    pub rename_entry: Arc<RenameEntryUseCase>,
    pub clipboard: Arc<ClipboardState>,
    pub copy_selection: Arc<CopySelectionUseCase>,
    pub move_selection: Arc<MoveSelectionUseCase>,
    pub paste_entries: Arc<PasteEntriesUseCase>,
    pub active_window_id: Arc<Mutex<Option<u32>>>,
    pub move_to_trash: Arc<MoveToTrashUseCase>,
    pub delete_permanently: Arc<DeletePermanentlyUseCase>,
    pub create_directory: Arc<CreateDirectoryUseCase>,
    pub get_entry_properties: Arc<GetEntryPropertiesUseCase>,
    pub open_entry: Arc<OpenEntryUseCase>,
    pub preview_entry: Arc<PreviewEntryUseCase>,

    pub list_drive_folder: Arc<ListDriveFolderUseCase>,
}
