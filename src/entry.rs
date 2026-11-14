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
    pub modified: Option<std::time::SystemTime>,
}

pub enum ManifestKind {
    File{object_id: String},
    Directory,
    Symlink{target: PathBuf},
    Other,
}

pub struct ManifestEntry {
    pub path: PathBuf,
    pub kind: ManifestKind,
    pub size: Option<u64>,
    pub modified: Option<std::time::SystemTime>,
}
