use std::{env, eprintln, fs::{self, OpenOptions}, io::Write, path::{Path, PathBuf}, println};

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

//Backup repo contains:
//  1.config file
//  2.objects/
//  3.snapshots/
//  4.tmp/
fn create_repo(path: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(path)?;

    fs::create_dir(path.join("objects/"))?;
    fs::create_dir(path.join("snapshots/"))?;
    fs::create_dir(path.join("tmp/"))?;

    let mut file = OpenOptions::new().write(true).create_new(true).open(path.join("config"))?;
    file.write_all(b"format_version = 1\n")?;

    Ok(())
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
        extract_repo(&args[2])
    } else {
        println!("Invalid Arguments.");
        return;
    };

    //check for validity of path
    if let Err(e) = validate_path(&path) {
        eprintln!("{e}");
        return;
    }

    //initialize repo
    if let Err(e) = create_repo(&path) {
        eprintln!("{e}");
        return;
    }

}
