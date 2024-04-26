use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use crate::graph::Graph;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Update this whenever the structure of Config changes
const VERSION: u32 = 1;

const FILENAME: &str = ".tuesday";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub graph: Graph,
}

impl Config {
    pub fn new(graph: &Graph) -> Self {
        Self {
            version: VERSION,
            graph: graph.clone(),
        }
    }
}

pub fn save_global(config: &Config) -> Result<()> {
    save(&mut get_global_save()?, config)?;
    Ok(())
}

pub fn save(file: &mut File, config: &Config) -> Result<()> {
    file.set_len(0)?;
    file.write_all(bincode::serialize(config)?.as_slice())?;
    file.flush()?;
    Ok(())
}

pub fn save_local(mut path: PathBuf, config: &Config) -> Result<()> {
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
        bincode::deserialize::<Config>(bytes.as_slice())?.graph
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
    Ok(serde_json::to_string(&Config::new(graph))?)
}

/// Imports from stdin
pub fn import_json_stdin() -> Result<Graph> {
    let mut stdin = io::stdin();
    let mut bytes = vec![];
    stdin.read_to_end(&mut bytes)?;
    Ok(serde_json::from_str(bincode::deserialize(
        bytes.as_slice(),
    )?)?)
}
