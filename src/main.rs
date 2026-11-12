use std::{env, eprintln, fs::{self, OpenOptions}, io::Write, path::{self, Path, PathBuf}, println};

enum EntryKind {
    File,
    Directory,
    Symlink{target: PathBuf},
    Other,
}

struct Entry {
    path: PathBuf,
    kind: EntryKind, 
    size: Option<u64>,
}

//Extract path. 
//Path is borrowed immutable, so using PathBuf
//Path is to PathBuf like str is to String
fn extract_repo(arg: &str) -> PathBuf {
    PathBuf::from(arg)
}

fn is_dir_empty(path: &Path) -> Result<bool, std::io::Error> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().transpose()?.is_none())
}

//The path is valid if:
//  1. It exists and is an empty directory.
//  2. It doesn't exist.
fn validate_path(path: &Path) -> Result<(), std::io::Error> {
    
    match fs::exists(path) {
        Ok(true) => {
            if !path.is_dir() {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path not a directory"))
            } else if !is_dir_empty(path)? {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Directory not empty"))
            } else {
                Ok(())
            }
        },
        Ok(false) => Ok(()),
        Err(e) => return Err(e),
    }
}

//Records existing directories from input path
fn check_repo_exists(path: &Path) -> Result<PathBuf, std::io::Error> {
    
    let mut existed_path = path;
    loop {
        if fs::exists(&existed_path)? {
            return Ok(existed_path.to_path_buf());
        }
        match existed_path.parent() {
            Some(cur_path) => existed_path = cur_path,
            None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No existing parent directory found")),
        }
    }
}

//Repo cleaning in case of partial initialization
fn clean_repo(dirs: &[PathBuf], files: &[PathBuf]) -> Result<(), std::io::Error> {

    for path in files {
        fs::remove_file(&path)?;
    }
    for path in dirs.iter().rev() {
        fs::remove_dir(&path)?;
    }
    Ok(())
}

// Backup repo contains:
//  1.config file
//  2.objects/
//  3.snapshots/
//  4.tmp/
// Handles partial initialization by cleaning the repo.
fn create_repo(path: &Path, existed_path: &Path) -> Result<(), std::io::Error> {
    
    let mut curr_path = existed_path.to_path_buf();
    let prefix = path.strip_prefix(&existed_path)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();

    let result = (|| -> Result<(), std::io::Error> {

        for component in prefix.components() {
            curr_path.push(component);
            fs::create_dir(&curr_path)?;
            dirs.push(curr_path.clone());
        }
        
        let mut new_mod = path.join("objects/");
        fs::create_dir(&new_mod)?;
        dirs.push(new_mod);

        new_mod = path.join("snapshots/");
        fs::create_dir(&new_mod)?;
        dirs.push(new_mod);

        new_mod = path.join("tmp/");
        fs::create_dir(&new_mod)?;
        dirs.push(new_mod);

        new_mod = path.join("config");
        let mut file = OpenOptions::new().write(true).create_new(true).open(&new_mod)?;
        files.push(new_mod);
        file.write_all(b"format_version = 1\n")?;

        Ok(())

    }) ();

    if let Err(e) = result {
        if let Err(ee) = clean_repo(&dirs, &files)  {
            eprintln!("Cleanup also failed: {ee}");   
        }
        return Err(e);
    }

    Ok(())
}

//Scans Directory recursively. It records:
//  1. Relative Path of each entry
//  2. Kind of Entry (File, Directory, Symlinks supported only)
//  3. File sizes
fn scan_path(root_path: &Path, curr_path: &Path) -> Result<Vec<Entry>, std::io::Error> {

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

fn main() {
    
    //read arguments and convert into a vector of strings
    let args: Vec<String> = env::args().collect();

    //first index stores the path of executable
    if args.len() != 3 {
        println!("Invalid Arguments.");
        println!("Usage: rbackup init <path>");
        return;
    }

    //check for 'init' cmd
    let path = if args[1] == "init" {
        //convert to absolute path
        match path::absolute(extract_repo(&args[2])) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        }
    } else {
        println!("Invalid Arguments.");
        return;
    };

    //check for validity of path
    if let Err(e) = validate_path(&path) {
        eprintln!("{e}");
        return;
    }

    //record existing repo structure beforehand
    let existed_path = match check_repo_exists(&path) {
        Ok(curr_path) => curr_path,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    //initialize repo
    if let Err(e) = create_repo(&path, &existed_path) {
        eprintln!("{e}");
        return;
    }

    //test call
    let entries = match scan_path(&path, &path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    for entry in &entries {
        println!("Path: {:?}", entry.path);
        println!("Size: {:?}", entry.size);
        if let EntryKind::Symlink { target } = &entry.kind {
            println!("Symlink target: {:?}", target);
        }
    }

}
