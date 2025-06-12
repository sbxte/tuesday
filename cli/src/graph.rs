//! This module contains graph operations that are only used by the CLI front-end.

use std::collections::HashMap;

use crate::blueprints::{BlueprintDoc, BlueprintGraph};
use crate::{dates::parse_datetime_extended, AppError, AppResult};
use tuecore::graph::{node::NodeType, Graph, GraphGetters};

pub trait CLIGraphOps {
    fn get_index_cli(&self, id: &str, assume_date: bool) -> AppResult<usize>;

    /// Hard copies a node. The source node will be the child of the target node.
    ///
    /// # Arguments
    /// - `from`: source node
    /// - `to`: target node
    ///
    /// # Returns
    /// The index of the newly created node.
    fn copy(&mut self, from: usize, to: usize) -> AppResult<usize>;

    /// Recursively hard copies a node. The source node will be the child of the target node.
    ///
    /// # Arguments
    /// - `from`: source node
    /// - `to`: target node
    ///
    /// # Returns
    /// The index of the newly created node.
    fn copy_recurse(&mut self, from: usize, to: usize) -> AppResult<()>;

    fn mv(&mut self, from: usize, to: usize) -> AppResult<()>;

    fn insert_blueprint_recurse(
        &mut self,
        map: &HashMap<usize, usize>,
        blueprint: &BlueprintDoc,
        blueprint_from: usize,
        node_parent: usize,
    ) -> AppResult<usize>;

    fn _insert_blueprint_recurse(
        &mut self,
        map: &HashMap<usize, usize>,
        blueprint: &BlueprintDoc,
        blueprint_from: usize,
        node_parent: usize,
    ) -> AppResult<()>;

    fn update_node_metadata_on_blueprint(
        &mut self,
        parent: usize,
        map: &HashMap<usize, usize>,
        blueprint: &BlueprintDoc,
    );
}

impl CLIGraphOps for Graph {
    fn get_index_cli(&self, id: &str, assume_date: bool) -> AppResult<usize> {
        // When user forces the ID to be interpreted as a date, just search through the dates hashmap.
        if assume_date {
            let date = parse_datetime_extended(id)?.date_naive();
            return Ok(self.get_date_index(&date)?);
        }

        // Normally, any number below the amount of dates from the current month can also be
        // interpreted as a date. However, when the user is just writing arbitrary number, it's most
        // likely that they're working with node indices. With this assumption, we parse any valid
        // usize as a node index.
        // edit: I may be wrong about this; maybe the implementation of parse_datetime is different
        // than how GNU's date does it. uhh, we'll just leave it as is.
        if id.parse::<u64>().is_ok() {
            return Ok(self.get_index(id)?);
        }

        // The second priority to our ID matching are aliases.
        if let Ok(idx) = self.get_index(id) {
            return Ok(idx);
        }

        // If none of those worked, then interpret the ID as a date.
        if let Ok(date) = parse_datetime_extended(id) {
            let idx = self.get_date_index(&date.date_naive())?;
            return Ok(idx);
        }

        // If that didn't work as well then the ID is invalid.
        Err(AppError::IndexRetrievalError(
            "Failed to match index with node".to_string(),
        ))
    }

    fn copy(&mut self, from: usize, to: usize) -> AppResult<usize> {
        // TODO: this requires copying `Node`s! Would it be better if we can get whichever field we
        // need instead, like in core/graph where node accesses are done using its index from
        // the nodes vector directly? (the nodes vector is private so we can't use it here)
        let source_node = self.get_node(from);

        let new_node = self.insert_child(source_node.title, to, source_node.data.is_pseudo())?;

        if let NodeType::Task(data) = source_node.data {
            self.set_task_state(new_node, data.state, true)?;
        };

        Ok(new_node)
    }

    fn copy_recurse(&mut self, from: usize, to: usize) -> AppResult<()> {
        let new = self.copy(from, to)?;

        // this function will stop recursing when there's no children left.
        for child in self.get_node(from).metadata.children {
            self.copy_recurse(child, new)?
        }

        Ok(())
    }

    fn mv(&mut self, from: usize, to: usize) -> AppResult<()> {
        self.clean_parents(from)?;
        self.link(to, from)?;
        Ok(())
    }

    fn _insert_blueprint_recurse(
        &mut self,
        map: &HashMap<usize, usize>,
        blueprint: &BlueprintDoc,
        blueprint_from: usize,
        node_parent: usize,
    ) -> AppResult<()> {
        let node = &blueprint.graph.nodes[blueprint_from];

        let children = node.metadata.children.clone();

        if self.get_node_checked(map[&blueprint_from]).is_none() {
            let new_id =
                self.insert_child(node.title.clone(), node_parent, node.data.is_pseudo())?;

            for child in children {
                self._insert_blueprint_recurse(map, blueprint, child, new_id)?;
            }
        };
        Ok(())
    }

    /// Note: Also call `self.update_node_metadata` at the end to properly update the indices
    /// inside the newly created nodes' metadata.
    fn insert_blueprint_recurse(
        &mut self,
        map: &HashMap<usize, usize>,
        blueprint: &BlueprintDoc,
        blueprint_from: usize,
        node_parent: usize,
    ) -> AppResult<usize> {
        let new_id = self.get_nodes().len();

        self._insert_blueprint_recurse(map, blueprint, blueprint_from, node_parent)?;

        Ok(new_id)
    }

    fn update_node_metadata_on_blueprint(
        &mut self,
        parent: usize,
        map: &HashMap<usize, usize>,
        blueprint: &BlueprintDoc,
    ) {
        // TODO: inconsistent with main's behavior
        for node in &blueprint.graph.nodes {
            // except the parent
            let mut mut_node = self.get_node_mut(map[&node.metadata.index]);
            if node.metadata.index != parent {
                mut_node.metadata.parents = node.metadata.parents.iter().map(|i| map[i]).collect();
            }
            mut_node.metadata.children = node.metadata.children.iter().map(|i| map[i]).collect();
        }
    }
}

// same algo as core/graph.rs for remapping indices except we use hashmaps here since the remapped
// indices count is almost definitely gonna be smaller
fn _insert_idx_recurse(
    blueprint_graph: &BlueprintGraph,
    graph: &Graph,
    map: &mut HashMap<usize, usize>,
    node: usize,
    current_idx: &mut usize,
) {
    if map.get(&node).is_none() {
        map.insert(node, *current_idx);
        *current_idx += 1;
        true
    } else {
        false
    };

    let node_children = &blueprint_graph.nodes[node].metadata.children;
    for child in node_children {
        _insert_idx_recurse(blueprint_graph, graph, map, *child, current_idx);
    }
}

pub fn new_graph_indices_map(
    blueprint: &BlueprintDoc,
    graph: &Graph,
    new_index: usize,
) -> HashMap<usize, usize> {
    let mut map = HashMap::new();
    _insert_idx_recurse(
        &blueprint.graph,
        graph,
        &mut map,
        blueprint.parent,
        &mut new_index.clone(),
    );
    map
}

pub fn graph_from_blueprint(blueprint: &BlueprintDoc) -> AppResult<Graph> {
    let mut graph = Graph::new();
    let map = new_graph_indices_map(blueprint, &graph, 0);
    let new_parent = &blueprint.graph.nodes[blueprint.parent];
    let parent_idx = graph.insert_root(new_parent.title.clone(), new_parent.data.is_pseudo());
    for child in &new_parent.metadata.children {
        graph.insert_blueprint_recurse(&map, blueprint, *child, parent_idx)?;
    }
    graph.update_node_metadata_on_blueprint(blueprint.parent, &map, blueprint);
    Ok(graph)
}
