use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::{fs::{self,}, sync::atomic::{AtomicU64, Ordering},};
use crate::scanner::create_tmp_file;

#[derive(Serialize, Deserialize)]
pub enum ManifestKind {
    File{object_id: String},
    Directory,
    Symlink{target: PathBuf},
    Other,
}

#[derive(Serialize, Deserialize)]
pub struct ManifestEntry {
    pub path: PathBuf,
    pub kind: ManifestKind,
    pub size: Option<u64>,
    pub modified: Option<std::time::SystemTime>,
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn create_snap_filename(repo_path: &Path) -> Result<PathBuf, std::io::Error> {
    let snap_dir = repo_path.join("snapshots/");

    loop {
         let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
         let timestamp = std::time::SystemTime::now()
         .duration_since(std::time::UNIX_EPOCH)
         .map_err(|e| std::io::Error::other(e))?
         .as_secs();

         let filename = format!("{timestamp}-{counter}.json");
         let snap_file_path = snap_dir.join(filename);

         return Ok(snap_file_path);
    }
}

pub fn create_snaps(manifests: &[ManifestEntry], repo_path: &Path) -> Result<(), std::io::Error> {

    let mut tmp_created = Vec::new();
    let mut snap_created = Vec::new();
    let result = (|| -> Result<(), std::io::Error> {
        for manifest in manifests {

            let (tmp_file_path, tmp_file) = create_tmp_file(repo_path)?;
            //let (snap_file_path, snap_file) = create_snap_file(repo_path)?;
            tmp_created.push(tmp_file_path.clone());
            serde_json::to_writer(&tmp_file, manifest)?;

            let snap_file_path = create_snap_filename(repo_path)?;
            snap_created.push(snap_file_path.clone());
            match fs::hard_link(&tmp_file_path, &snap_file_path) {
                Ok(_) => (),
                Err(e) =>  return Err(e),
            }
        }
        Ok(())
    }) ();
    
    match result {
        Ok(_) => {
            for file_path in tmp_created {
                fs::remove_file(&file_path)?;
            }
            Ok(())
        }
        Err(e) => {
            for file_path in tmp_created {
                if let Err(ee) = fs::remove_file(&file_path) {
                    eprintln!("Cleanup also failed: {ee}");
                }
            }
            for file_path in snap_created {
                if let Err(ee) = fs::remove_file(&file_path) {
                    eprintln!("Cleanup also failed: {ee}");
                }
            }
            Err(e)
        }
    }

}
