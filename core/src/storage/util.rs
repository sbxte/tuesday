use std::fs::{File, OpenOptions};

use anyhow::Result;

pub fn get_global_save(filename: &str) -> Result<File> {
    let mut path = if let Some(x) = home::home_dir() {
        x
    } else {
        panic!("Home directory unavailable!");
    };
    path.push(filename);
    if !path.exists() {
        Ok(OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?)
    } else {
        Ok(OpenOptions::new()
            .write(true)
            .truncate(false)
            .read(true)
            .open(path)?)
    }
}
