use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use home::home_dir;

/// Attempt to obtain save file by recursing through parent directories or searching the home
/// directory for a file with a matching filename
/// See also [`get_save_path`]
pub fn get_save(base_path: &Path, filename: &str) -> Option<File> {
    let path = get_save_path(base_path, filename)?;

    OpenOptions::new()
        .write(true)
        .truncate(false)
        .read(true)
        .open(path)
        .ok()
}

/// Attempt to locate save path by recursively check parent directory, if none are found then use home directory,
/// else return [`None`]
pub fn get_save_path(path: &Path, filename: &str) -> Option<PathBuf> {
    let path = PathBuf::from(path);
    if path.exists() {
        return Some(path);
    }

    let path = get_save_recurse_parent(path, filename);
    if let Some(path) = path {
        return Some(path);
    }

    // Attempt to search in home directory
    if let Some(x) = home_dir() {
        if x.exists() {
            return Some(x);
        }
    }

    // Just give up at this point lmao
    None
}

fn get_save_recurse_parent(mut path: PathBuf, filename: &str) -> Option<PathBuf> {
    path.push(filename);
    if path.exists() {
        return Some(path);
    }
    path.pop(); // Remove filename

    // Stop if we cant go up any further (e.g. at root directory)
    let has_parent = path.pop(); // Navigate up one level
    if !has_parent {
        None
    } else {
        get_save_recurse_parent(path, filename)
    }
}
