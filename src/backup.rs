use std::path::Path;

use crate::{entry::{Entry, EntryKind, ManifestEntry, ManifestKind}, scanner::create_obj};


pub fn create_manifest(entries: &[Entry], repo_path: &Path, source_root: &Path) -> Result<Vec<ManifestEntry>, std::io::Error> {

    //output vector
    let mut output: Vec<ManifestEntry> = Vec::new();

    for entry in entries {

       let kind =  match &entry.kind {
            EntryKind::File => {
                let src_file_path = source_root.join(&entry.path);
                let object_id = create_obj(repo_path, &src_file_path)?;
                ManifestKind::File { object_id }
            },
            EntryKind::Directory => ManifestKind::Directory,
            EntryKind::Symlink { target } => ManifestKind::Symlink { target: target.clone() },
            EntryKind::Other => ManifestKind::Other,
        };
        

        output.push(ManifestEntry { path: entry.path.clone(), kind, size: entry.size, modified: entry.modified});
    }

    Ok(output)
}
