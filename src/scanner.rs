use std::{fs, path::Path};
use crate::entry::{Entry, EntryKind};

//Scans Directory recursively. It records:
//  1. Relative Path of each entry
//  2. Kind of Entry (File, Directory, Symlinks supported only)
//  3. File sizes
pub fn scan_path(root_path: &Path, curr_path: &Path) -> Result<Vec<Entry>, std::io::Error> {

    //output vector
    let mut discovered: Vec<Entry> = Vec::new();

    //iterator over entries
    let entries = fs::read_dir(curr_path)?;

    for entry in entries {
        let entry = entry?;

        //evaluate entry path and convert to relative (strip input root path)
        let entry_path = entry.path().strip_prefix(root_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?.to_path_buf();

        //evaluate entry kind
        let entry_kind = entry.file_type()?;
        let (entry_kind, recursive, entry_size) = if entry_kind.is_file() {

            //calculate file size
            let entry_size = entry.metadata()?.len();
            (EntryKind::File, false, Some(entry_size))
        } else if entry_kind.is_dir() {
            (EntryKind::Directory, true, None)
        } else if entry_kind.is_symlink() {
            (EntryKind::Symlink{
                target : std::fs::read_link(entry.path())?
                }, false, None)
        } else {
            println!("Special File: {:?} Unsupported", entry.file_name());
            (EntryKind::Other, false, None)
        };

        //store Entry in output vector
        discovered.push(Entry{
            path: entry_path,
            kind: entry_kind,
            size: entry_size,
        });

        //recursive directory scan, record child after parents
        if recursive {
            discovered.extend(scan_path(root_path ,&entry.path())?);
        }
    }

    //sort for consistency
    discovered.sort_by_key(|entry| entry.path.to_path_buf());
    Ok(discovered)
}
