use std::sync::Mutex;

use fm_domain::ClipboardEntry;

#[derive(Default)]
pub struct ClipboardState {
    entries: Mutex<Vec<ClipboardEntry>>,
}

impl ClipboardState {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }
    pub fn len(&self) -> usize {
        let guard = self.entries.lock().unwrap();
        guard.len()
    }

    pub fn set_entries(&self, entries: Vec<ClipboardEntry>) {
        let mut guard = self.entries.lock().unwrap();
        *guard = entries;
    }

    pub fn get_entries(&self) -> Vec<ClipboardEntry> {
        let guard = self.entries.lock().unwrap();
        guard.clone()
    }

    pub fn clear(&self) {
        let mut guard = self.entries.lock().unwrap();
        guard.clear();
    }

    pub fn is_empty(&self) -> bool {
        let guard = self.entries.lock().unwrap();
        guard.is_empty()
    }
}
