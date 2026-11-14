use std::{
    format, fs::{
        self, File, OpenOptions, metadata
    }, io::{self, Read, Write}, path::{
        Path, PathBuf, 
    }, println, sync::atomic::{AtomicU64, Ordering},
};
use crate::entry::{Entry, EntryKind};
use sha2::{Digest, Sha256};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

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
        let full_path = entry.path();

        //evaluate entry path and convert to relative (strip input root path)
        let entry_path = full_path.strip_prefix(root_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?.to_path_buf();

        //evaluate entry kind
        let entry_kind = entry.file_type()?;
        let (entry_kind, recursive, entry_size, modified) = if entry_kind.is_file() {

            let metadata = entry.metadata()?;
            (EntryKind::File, false, Some(metadata.len()), metadata.modified().ok())
        } else if entry_kind.is_dir() {
            let metadata = entry.metadata()?;

            (EntryKind::Directory, true, None, metadata.modified().ok())
        } else if entry_kind.is_symlink() {
            let metadata = fs::symlink_metadata(&full_path)?;

            (EntryKind::Symlink{
                target : std::fs::read_link(&full_path)?
                }, false, None, metadata.modified().ok())
        } else {
            println!("Special File: {:?} Unsupported", entry.file_name());
            (EntryKind::Other, false, None, None)
        };

        //store Entry in output vector
        discovered.push(Entry{
            path: entry_path,
            kind: entry_kind,
            size: entry_size,
            modified
        });

        //recursive directory scan, record child after parents
        if recursive {
            discovered.extend(scan_path(root_path, &full_path)?);
        }
    }

    //sort for consistency
    discovered.sort_by_key(|entry| entry.path.to_path_buf());
    Ok(discovered)
}

//Create a temporary file
fn create_tmp_file(repo_path: &Path) -> Result<(PathBuf, File), std::io::Error> {
    let tmp_dir = repo_path.join("tmp/");

    loop {
         let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
         let filename = format!("{}-{counter}.tmp", std::process::id());
         let tmp_file_path = tmp_dir.join(filename);

         match OpenOptions::new().write(true).create_new(true).open(&tmp_file_path) {
             Ok(file) => return Ok((tmp_file_path, file)),
             Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
             Err(e) => return Err(e),
         }
    }
}

//Create object file. It Includes:
//  1) Src file reading
//  2) Tmp file writing
//  3) Hashing src file data
//  4) Object file creation 
//  5) Moving tmp file to object file
//  6) tmp file cleanup in case of partial process
pub fn create_obj(repo_path: &Path, src_file_path: &Path) -> Result<String, std::io::Error> {

    //buffer for chunks
    let mut file = std::fs::File::open(src_file_path)?;
    let mut buf = [0u8; 1500];
    
    //tmp file creation for reading data
    let (tmp_file_path, mut tmp_file) = create_tmp_file(repo_path)?;

    let result = (|| -> Result<String, std::io::Error> {
        let mut hasher = Sha256::new();

        //read src file in chunks
        loop {
            let bytes_size = file.read(&mut buf)?;
            if bytes_size == 0 {
                break;
            }
            let chunk = &buf[..bytes_size];

            //update hasher for chunk
            hasher.update(&buf[..bytes_size]);

            //write into a tmp file
            tmp_file.write_all(chunk)?;
        }
        //finalize hash
        let hash = hasher.finalize();
        let hash_string = hex::encode(hash);

        //object file path using the hash
        let obj_file_path = repo_path.join("objects/".to_owned()+&hash_string);

        //checks file existence internally and handles race condition
        match fs::hard_link(&tmp_file_path, &obj_file_path) {
            Ok(()) => Ok(hash_string),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(hash_string),
            Err(e) => Err(e),
        }
    }) ();

    //tmp file cleanup
    let hash_string = match result {
        Ok(hash_string) => {
            fs::remove_file(&tmp_file_path)?;
            Ok(hash_string)
        } 
        Err(e) => {
            if let Err(ee) = fs::remove_file(&tmp_file_path) {
                eprintln!("Cleanup also failed: {ee}");
            }
            Err(e)
        }
    }?;
    Ok(hash_string)
}
