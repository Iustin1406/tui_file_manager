#[derive(Clone, Debug)]
pub struct DriveEntry {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub is_folder: bool,
}
