use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use crate::graph::TaskGraph;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Update this whenever the structure of Config changes
const VERSION: u32 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub graph: TaskGraph,
}

impl Config {
    pub fn new(graph: &TaskGraph) -> Self {
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

pub fn load(file: &mut File) -> Result<TaskGraph> {
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;
    let graph: TaskGraph = if bytes.is_empty() {
        TaskGraph::new()
    } else {
        bincode::deserialize::<Config>(bytes.as_slice())?.graph
    };
    Ok(graph)
}

pub fn load_global() -> Result<TaskGraph> {
    load(&mut get_global_save()?)
}

pub fn get_global_save() -> Result<File> {
    let mut path = if let Some(x) = home::home_dir() {
        x
    } else {
        panic!("Home directory unavailable!");
    };
    path.push(".tuesday");
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
