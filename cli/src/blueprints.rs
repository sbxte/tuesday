use std::fmt::Display;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs::File, io::{Read, Write}};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tuecore::graph::{node::Node, Graph, GraphGetters};

pub type BlueprintResult<T> = Result<T, BlueprintError>;

#[derive(Error, Debug)]
pub enum BlueprintError {
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(#[from] serde_yaml_ng::Error),

    #[error("Blueprint file already exists at {0}")]
    FileExists(String),

    #[error("Failed to access and/or create blueprints save directory: {0}")]
    SaveDirError(String),

    #[error("Failed to access blueprint: {0}")]
    FailedToAccess(String)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BlueprintGraph {
    pub(crate) nodes: Vec<Node>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BlueprintDoc {
    /// Author of blueprint.
    pub(crate) author: Option<String>,
    /// Which doc-version was the `tuecli` binary using when it wrote this blueprint doc.
    pub(crate) version: u32,
    /// The graph of the blueprint.
    pub(crate) graph: BlueprintGraph,
    /// The parent node index. Always set to 0.
    pub(crate) parent: usize
}

// same algo as core/graph.rs for remapping indices except we use hashmaps here since the remapped
// indices count is almost definitely gonna be smaller
fn _insert_idx_recurse(graph: &Graph, map: &mut HashMap<usize, usize>, node: usize, current_idx: &mut usize) {
    if map.get(&node).is_none() {
        map.insert(node, *current_idx);
        *current_idx += 1;
        true
    } else {
        false
    };

    let node_children = &graph.get_node(node).metadata.children;
    for child in node_children {
        _insert_idx_recurse(graph, map, *child, current_idx);
    }

}

fn new_bp_indices_map(graph: &Graph, node: usize) -> HashMap<usize, usize> {
    let mut map = HashMap::new();
    _insert_idx_recurse(graph, &mut map, node, &mut 0);
    map

}

fn _write_node_recurse(graph: &Graph, indices_map: &HashMap<usize, usize>, source_idx: usize, nodes_store: &mut Vec<Node>) {
    let node = graph.get_node(source_idx);

    let children = node.metadata.children.clone();

    if nodes_store.get(indices_map[&source_idx]).is_none() {
        nodes_store.push(node);
    }

    for child in children {
        _write_node_recurse(graph, indices_map, child, nodes_store);
    }

    let node_ref = &mut nodes_store[indices_map[&source_idx]];
    node_ref.metadata.index = indices_map[&source_idx];

    if indices_map[&source_idx] == 0 {
        node_ref.metadata.parents.clear();
    }

}

fn write_node_recurse(graph: &Graph, source_idx: usize, nodes_store: &mut Vec<Node>) {
    let map = new_bp_indices_map(graph, source_idx);
    _write_node_recurse(graph, &map, source_idx, nodes_store);

    for node in nodes_store {
        node.metadata.parents = node.metadata.parents.iter().filter_map(|i| map.get(i)).map(|i| *i).collect();
        node.metadata.children = node.metadata.children.iter().filter_map(|i| map.get(i)).map(|i| *i).collect();
    }

}

impl BlueprintGraph {
    pub fn from_idx(graph: &Graph, idx: usize) -> Self {
        let mut nodes = Vec::new();
        write_node_recurse(graph, idx, &mut nodes);

        Self {
            nodes
        }
    }
}

impl BlueprintDoc {
    pub fn from_idx(graph: &Graph, version: u32, parent: usize, author: Option<String>) -> Self {
        Self {
            graph: BlueprintGraph::from_idx(graph, parent),
            // TODO: parent is literally always 0 so why even have it
            parent: 0,
            version,
            author
        }
    }

    pub fn save_to_file(&self, file: &mut File) -> BlueprintResult<()> {
        file.set_len(0)?;
        serde_yaml_ng::to_writer(&mut *file, self)?;
        file.flush()?;
        Ok(())
    }
}

impl Display for BlueprintDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_yaml_ng::to_string(self).unwrap())
    }
}

pub fn get_doc(file: &mut File) -> BlueprintResult<BlueprintDoc> {
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;
    Ok(serde_yaml_ng::from_slice::<BlueprintDoc>(&bytes)?)

}

pub fn get_blueprints_listing(save_dir: &Path) -> BlueprintResult<Vec<String>> {
    // TODO: totally not cursed
    let files = read_dir(save_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter_map(|path| path.file_name().unwrap().to_string_lossy().strip_suffix(".yaml").map(|s| s.to_string()))
        .collect();
    Ok(files)
}

pub fn try_get_blueprint_from_save_dir(save_dir: &Path, name: &str) -> BlueprintResult<BlueprintDoc> {
    let files = get_blueprints_listing(save_dir)?;
    if files.contains(&name.to_string()) {
        let mut path = PathBuf::from(save_dir);
        path.push(format!("{}.yaml", name));
        let mut file = File::open(path)?;
        return Ok(get_doc(&mut file)?)
    } else {
        return Err(BlueprintError::FailedToAccess("Cannot found from save directory".to_string()))
    };
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tuecore::{doc::get_doc_ver, graph::{node::{task::TaskData, Node, NodeMetadata, NodeType}, Graph}};

    use crate::{blueprints::BlueprintGraph, config::CliConfig, display::Displayer};

    use super::{new_bp_indices_map, BlueprintDoc};

    fn example_graph() -> Graph {
        let mut example_graph = Graph::new();
        let idx = example_graph.insert_root("root (1)".to_string(), false);
        example_graph.insert_child("child (1)".to_string(), idx, false).unwrap();
        let target = example_graph.insert_child("child (2)".to_string(), idx, false).unwrap();
        example_graph.insert_child("child (1) of child (2)".to_string(), target, false).unwrap();
        let idx = example_graph.insert_child("child (2) of child (2)".to_string(), target, false).unwrap();
        example_graph.insert_child("child (1) of child (1) of child (2)".to_string(), idx, false).unwrap();
        example_graph.insert_child("child (3) of child (2)".to_string(), target, false).unwrap();

        let cfg = CliConfig::default();
        let displayer = Displayer::new(&cfg);

        displayer.list_roots(&example_graph, 0, false).unwrap();

        example_graph
    }

    #[test]
    fn test_indices_mapper() {
        let example_graph = example_graph();
        
        let mut map_should_be = HashMap::new();
        map_should_be.insert(2, 0);
        map_should_be.insert(3, 1);
        map_should_be.insert(4, 2);
        map_should_be.insert(5, 3);
        map_should_be.insert(6, 4);

        let map = new_bp_indices_map(&example_graph, 2);

        assert_eq!(map, map_should_be);
    }

    #[test]
    fn write_to_blueprint() {
        let graph = example_graph();
        let doc = BlueprintDoc::from_idx(&graph, get_doc_ver(), 2, None);

        let should_be = BlueprintDoc {
            author: None,
            version: get_doc_ver(),
            parent: 0,
            graph: BlueprintGraph {
                nodes: vec![
                    Node {
                        title: "child (2)".to_string(),
                        data: NodeType::Task(TaskData::default()),
                        metadata: NodeMetadata {
                            index: 0,
                            children: vec![1, 2, 4],
                            ..Default::default()
                        }
                    },
                    Node {
                        title: "child (1) of child (2)".to_string(),
                        data: NodeType::Task(TaskData::default()),
                        metadata: NodeMetadata {
                            index: 1,
                            parents: vec![0],
                            ..Default::default()
                        }
                    },
                    Node {
                        title: "child (2) of child (2)".to_string(),
                        data: NodeType::Task(TaskData::default()),
                        metadata: NodeMetadata {
                            index: 2,
                            children: vec![3],
                            parents: vec![0],
                            ..Default::default()
                        }
                    },
                    Node {
                        title: "child (1) of child (1) of child (2)".to_string(),
                        data: NodeType::Task(TaskData::default()),
                        metadata: NodeMetadata {
                            index: 3,
                            parents: vec![2],
                            ..Default::default()
                        }
                    },
                    Node {
                        title: "child (3) of child (2)".to_string(),
                        data: NodeType::Task(TaskData::default()),
                        metadata: NodeMetadata {
                            index: 4,
                            parents: vec![0],
                            ..Default::default()
                        }
                    }

                ]
            }
        };

        assert_eq!(doc, should_be);
    }
}
