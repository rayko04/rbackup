use std::path::PathBuf;

pub enum EntryKind {
    File,
    Directory,
    Symlink{target: PathBuf},
    Other,
}

pub struct Entry {
    pub path: PathBuf,
    pub kind: EntryKind, 
    pub size: Option<u64>,
}
