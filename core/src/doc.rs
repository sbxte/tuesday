use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use crate::graph::Graph;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod compat;

/// Update this whenever the structure of Config or Graph changes
const VERSION: u32 = 4;

const FILENAME: &str = ".tuesday";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Doc {
    pub version: u32,
    pub graph: Graph,
}

impl Doc {
    pub fn new(graph: &Graph) -> Self {
        Self {
            version: VERSION,
            graph: graph.clone(),
        }
    }
}

pub fn save_global(config: &Doc) -> Result<()> {
    save(&mut get_global_save()?, config)?;
    Ok(())
}

pub fn save(mut file: &mut File, config: &Doc) -> Result<()> {
    file.set_len(0)?;
    serde_yaml_ng::to_writer(&mut file, config)?;
    file.flush()?;
    Ok(())
}

pub fn save_local(mut path: PathBuf, config: &Doc) -> Result<()> {
    path.push(FILENAME);
    save(
        &mut OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .read(true)
            .open(path)?,
        config,
    )?;
    Ok(())
}

pub fn load(file: &mut File) -> Result<Graph> {
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;
    let graph: Graph = if bytes.is_empty() {
        Graph::new()
    } else {
        serde_yaml_ng::from_slice::<Doc>(bytes.as_slice())
            .or(compat::compat_parse(bytes.as_slice()))?
            .graph
    };
    Ok(graph)
}

pub fn try_load_local(mut path: PathBuf) -> Result<Option<Graph>> {
    path.push(FILENAME);
    if path.exists() {
        Ok(Some(load(
            &mut OpenOptions::new()
                .write(true)
                .truncate(false)
                .read(true)
                .open(path)?,
        )?))
    } else {
        Ok(None)
    }
}

pub fn load_local(mut path: PathBuf) -> Result<Graph> {
    // For when user specifies custom path
    if path.exists() && path.is_file() {
        return load(
            &mut OpenOptions::new()
                .create(true)
                .append(true)
                .read(true)
                .open(path)?,
        );
    }

    // Otherwise try using FILENAME
    path.push(FILENAME);
    let graph = if !path.exists() {
        load(
            &mut OpenOptions::new()
                .create(true)
                .append(true)
                .read(true)
                .open(path)?,
        )?
    } else {
        load(
            &mut OpenOptions::new()
                .write(true)
                .truncate(false)
                .read(true)
                .open(path)?,
        )?
    };
    Ok(graph)
}

pub fn load_global() -> Result<Graph> {
    load(&mut get_global_save()?)
}

pub fn local_exists(mut path: PathBuf) -> bool {
    path.push(FILENAME);
    path.exists()
}

pub fn get_global_save() -> Result<File> {
    let mut path = if let Some(x) = home::home_dir() {
        x
    } else {
        panic!("Home directory unavailable!");
    };
    path.push(FILENAME);
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

pub fn export_json(graph: &Graph) -> Result<String> {
    Ok(serde_json::to_string(&Doc::new(graph))?)
}

/// Imports from stdin
pub fn import_json_stdin() -> Result<Doc> {
    let mut stdin = io::stdin();
    let mut bytes = vec![];
    stdin.read_to_end(&mut bytes)?;
    let config: Doc = serde_json::from_slice(bytes.as_slice())?;
    Ok(config)
}
