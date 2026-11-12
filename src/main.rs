mod entry;
mod scanner;
mod repository;

use crate::{
    entry::EntryKind, 
    scanner::scan_path, 
    repository::{
        create_repo, 
        check_repo_exists, 
        validate_path
    }
};

use std::{env, eprintln, path::{self, PathBuf}, println};

//Extract path. 
//Path is borrowed immutable, so using PathBuf
//Path is to PathBuf like str is to String
fn extract_repo(arg: &str) -> PathBuf {
    PathBuf::from(arg)
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
