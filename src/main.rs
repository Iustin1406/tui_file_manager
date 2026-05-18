use std::sync::{Arc, Mutex};

use fm_application::{
    ClipboardState, CopySelectionUseCase, CreateDirectoryUseCase, CreateDriveFolderUseCase,
    DeletePermanentlyUseCase, GetEntryPropertiesUseCase, ListDriveFolderUseCase,
    MoveSelectionUseCase, MoveToTrashUseCase, OpenEntryUseCase, PasteEntriesUseCase,
    PreviewEntryUseCase, RefreshDriveFolderUseCase, RenameDriveItemUseCase, RenameEntryUseCase,
    UiDependencies, UploadDriveFileUseCase, UploadDriveFolderUseCase,
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

    let use_ui = true; // change to false to run in CLI mode for testing Google Drive integration

    if !use_ui {
        println!("Running in CLI mode (Google Drive auth/debug)...");

        let list_usecase = ListDriveFolderUseCase::new(drive.clone());

        match list_usecase.execute("root") {
            Ok(entries) => {
                println!("Google Drive root:");
                println!("Entries found: {}", entries.len());

                for entry in entries.iter().take(10) {
                    println!(
                        "[{}] {} ({})",
                        if entry.is_folder { "DIR" } else { "FILE" },
                        entry.name,
                        entry.id
                    );
                }
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }

        return Ok(());
    }

    let list_drive_folder = Arc::new(ListDriveFolderUseCase::new(drive.clone()));
    let refresh_drive_folder = Arc::new(RefreshDriveFolderUseCase::new(drive.clone()));
    let create_drive_folder = Arc::new(CreateDriveFolderUseCase::new(drive.clone()));
    let rename_drive_item = Arc::new(RenameDriveItemUseCase::new(drive.clone()));
    let upload_drive_file = Arc::new(UploadDriveFileUseCase::new(drive.clone()));
    let upload_drive_folder = Arc::new(UploadDriveFolderUseCase::new(drive.clone()));

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
        create_drive_folder,
        rename_drive_item,
        upload_drive_file,
        upload_drive_folder,
        pending_drive_refresh: Arc::new(Mutex::new(Vec::new())),
    };

    fm_ui::run(deps)
}
