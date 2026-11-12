use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

fn is_dir_empty(path: &Path) -> Result<bool, std::io::Error> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().transpose()?.is_none())
}

//The path is valid if:
//  1. It exists and is an empty directory.
//  2. It doesn't exist.
pub fn validate_path(path: &Path) -> Result<(), std::io::Error> {
    
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
        Err(e) => Err(e),
    }
}

//Records existing directories from input path
pub fn check_repo_exists(path: &Path) -> Result<PathBuf, std::io::Error> {
    
    let mut existed_path = path;
    loop {
        if fs::exists(existed_path)? {
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
        fs::remove_file(path)?;
    }
    for path in dirs.iter().rev() {
        fs::remove_dir(path)?;
    }
    Ok(())
}

// Backup repo contains:
//  1.config file
//  2.objects/
//  3.snapshots/
//  4.tmp/
// Handles partial initialization by cleaning the repo.
pub fn create_repo(path: &Path, existed_path: &Path) -> Result<(), std::io::Error> {
    
    let mut curr_path = existed_path.to_path_buf();
    let prefix = path.strip_prefix(existed_path)
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
