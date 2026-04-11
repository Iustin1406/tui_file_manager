use std::sync::{Arc, Mutex};

use fm_application::{
    ClipboardState, CopySelectionUseCase, DeletePermanentlyUseCase, MoveSelectionUseCase,
    MoveToTrashUseCase, PasteEntriesUseCase, RenameEntryUseCase, UiDependencies,
};
use fm_infra::StdFileSystem;

fn main() -> Result<(), appcui::system::Error> {
    let fs = Arc::new(StdFileSystem);
    let clipboard = Arc::new(ClipboardState::new());

    let rename_entry = Arc::new(RenameEntryUseCase::new(fs.clone()));
    let copy_selection = Arc::new(CopySelectionUseCase::new(clipboard.clone()));
    let move_selection = Arc::new(MoveSelectionUseCase::new(clipboard.clone()));
    let move_to_trash = Arc::new(MoveToTrashUseCase::new(fs.clone()));
    let delete_permanently = Arc::new(DeletePermanentlyUseCase::new(fs.clone()));
    let paste_entries = Arc::new(PasteEntriesUseCase::new(fs.clone(), clipboard.clone()));

    let deps = UiDependencies {
        fs,
        clipboard,
        copy_selection,
        move_selection,
        move_to_trash,
        delete_permanently,
        paste_entries,
        rename_entry,
        active_window_id: Arc::new(Mutex::new(None)),
    };

    fm_ui::run(deps)
}
