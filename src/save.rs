use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use crate::graph::Graph;

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use yaml_rust2::YamlLoader;

/// Update this whenever the structure of Config or Graph changes
const VERSION: u32 = 3;

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

pub fn save(mut file: &mut File, config: &Config) -> Result<()> {
    file.set_len(0)?;
    serde_yaml_ng::to_writer(&mut file, config)?;
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
        serde_yaml_ng::from_slice::<Config>(bytes.as_slice())
            .or(parse_yaml(String::from_utf8(bytes)?.as_str()))?
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
    Ok(serde_json::to_string(&Config::new(graph))?)
}

/// Imports from stdin
pub fn import_json_stdin() -> Result<Config> {
    let mut stdin = io::stdin();
    let mut bytes = vec![];
    stdin.read_to_end(&mut bytes)?;
    let config: Config = serde_json::from_slice(bytes.as_slice())?;
    Ok(config)
}

/// Manually parse yaml config instead of using serde_derive
/// This allows format mismatch handling where otherwise serde_derive would panic
pub fn parse_yaml(input: &str) -> Result<Config> {
    let docs = YamlLoader::load_from_str(input)?;
    let doc = &docs[0];

    let graph_doc = &doc["graph"];

    // Parse nodes
    // Provide default values if any are missing
    // ~ Maps are funky, functional programming go brrrr
    let mut nodes = vec![];
    for node_doc in graph_doc["nodes"].as_vec().unwrap_or(&vec![]) {
        if node_doc.is_null() {
            nodes.push(None);
            continue;
        }

        let mut parents = vec![];
        for parent_doc in node_doc["parents"].as_vec().unwrap_or(&vec![]) {
            parents.push(
                parent_doc
                    .as_i64()
                    .expect("Parent index must be an integer") as usize,
            );
        }
        let mut children = vec![];
        for child_doc in node_doc["children"].as_vec().unwrap_or(&vec![]) {
            children.push(child_doc.as_i64().expect("Parent index must be an integer") as usize);
        }

        nodes.push(Some(RefCell::new(crate::graph::Node {
            message: node_doc["message"].as_str().unwrap_or("").to_string(),
            r#type: node_doc["type"]
                .as_str()
                .map_or(crate::graph::NodeType::default(), |n| {
                    crate::graph::NodeType::from_str(n, true)
                        .unwrap_or(crate::graph::NodeType::default())
                }),
            state: node_doc["state"]
                .as_str()
                .map_or(crate::graph::NodeState::default(), |n| {
                    crate::graph::NodeState::from_str(n, true)
                        .unwrap_or(crate::graph::NodeState::default())
                }),
            index: node_doc["index"]
                .as_i64()
                .expect("Node index must be an integer") as usize,
            alias: node_doc["alias"].as_str().map(|s| s.to_string()),
            parents,
            children,
        })));
    }

    // Roots, dates, and aliases
    let roots = graph_doc["roots"]
        .as_vec()
        .unwrap_or(&vec![])
        .iter()
        .map(|i| i.as_i64().expect("Root index must be an integer") as usize)
        .collect::<Vec<_>>();
    let dates = graph_doc["dates"]
        .as_hash()
        .unwrap_or(&yaml_rust2::yaml::Hash::new())
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().expect("Date key must be a string").to_string(),
                v.as_i64().expect("Date node index must be an integer") as usize,
            )
        })
        .collect::<HashMap<_, _>>();
    let aliases = graph_doc["aliases"]
        .as_hash()
        .unwrap_or(&yaml_rust2::yaml::Hash::new())
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().expect("Alias key must be a string").to_string(),
                v.as_i64().expect("Alias node index must be an integer") as usize,
            )
        })
        .collect::<HashMap<_, _>>();

    // Unify everything
    let config = Config {
        version: doc["version"].as_i64().expect("Version should be integer") as u32,
        graph: Graph {
            nodes,
            roots,
            dates,
            aliases,
        },
    };
    Ok(config)
}
