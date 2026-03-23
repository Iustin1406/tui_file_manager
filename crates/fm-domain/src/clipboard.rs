use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardMode {
    Copy,
    Move,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardEntry {
    pub source_path: PathBuf,
    pub mode: ClipboardMode,
}
