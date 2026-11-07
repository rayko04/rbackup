use std::{env, eprintln, fs, path::{Path, PathBuf}, println};

fn extract_repo(arg: &str) -> PathBuf {
    PathBuf::from(arg)
}

fn is_dir_empty(path: &Path) -> Result<bool, std::io::Error> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().is_none())
}

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

fn main() {
    
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Invalid Arguments.");
        println!("Usage: rbackup init <path>");
        return;
    }

    let path = if args[1] == "init" {
        extract_repo(&args[2])
    } else {
        println!("Wrong arguments passed.");
        return;
    };

    if let Err(e) = validate_path(&path) {
        eprintln!("{e}");
        return;
    }

}
