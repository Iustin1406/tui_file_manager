use std::sync::{Arc, Mutex};

use fm_application::{
    ClipboardState, CopySelectionUseCase, CreateDirectoryUseCase, DeletePermanentlyUseCase,
    GetEntryPropertiesUseCase, ListDriveFolderUseCase, MoveSelectionUseCase, MoveToTrashUseCase,
    OpenEntryUseCase, PasteEntriesUseCase, PreviewEntryUseCase, RefreshDriveFolderUseCase,
    RenameEntryUseCase, UiDependencies,
};
use fm_infra::{GoogleDriveAdapter, StdFileSystem};

fn main() -> Result<(), appcui::system::Error> {
    let fs = Arc::new(StdFileSystem);
    let clipboard = Arc::new(ClipboardState::new());

    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let drive = Arc::new(GoogleDriveAdapter::new(
        project_root.join("config/client_secret.json"),
        project_root.join("config/token.json"),
    ));

    let list_drive_folder = Arc::new(ListDriveFolderUseCase::new(drive.clone()));
    let refresh_drive_folder = Arc::new(RefreshDriveFolderUseCase::new(drive));

    let rename_entry = Arc::new(RenameEntryUseCase::new(fs.clone()));
    let copy_selection = Arc::new(CopySelectionUseCase::new(clipboard.clone()));
    let move_selection = Arc::new(MoveSelectionUseCase::new(clipboard.clone()));
    let move_to_trash = Arc::new(MoveToTrashUseCase::new(fs.clone()));
    let delete_permanently = Arc::new(DeletePermanentlyUseCase::new(fs.clone()));
    let paste_entries = Arc::new(PasteEntriesUseCase::new(fs.clone(), clipboard.clone()));
    let create_directory = Arc::new(CreateDirectoryUseCase::new(fs.clone()));
    let get_entry_properties = Arc::new(GetEntryPropertiesUseCase::new(fs.clone()));
    let open_entry = Arc::new(OpenEntryUseCase::new(fs.clone()));
    let preview_entry = Arc::new(PreviewEntryUseCase::new(fs.clone()));

    let deps = UiDependencies {
        fs,
        clipboard,
        copy_selection,
        move_selection,
        move_to_trash,
        delete_permanently,
        paste_entries,
        rename_entry,
        active_window: Arc::new(Mutex::new(None)),
        create_directory,
        get_entry_properties,
        open_entry,
        preview_entry,
        list_drive_folder,
        refresh_drive_folder,
    };

    fm_ui::run(deps)
}
