//! This module contains graph operations that are only used by the CLI front-end.

use tuecore::graph::{node::NodeType, Graph, GraphGetters};
use crate::{dates::parse_datetime_extended, AppError, AppResult};

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
        if let Ok(_) = id.parse::<u64>() {
            return Ok(self.get_index(id)?)
        }

        // The second priority to our ID matching are aliases.
        if let Ok(idx) = self.get_index(id) {
            return Ok(idx)
        }

        // If none of those worked, then interpret the ID as a date.
        if let Ok(date) = parse_datetime_extended(id)  {
            let idx = self.get_date_index(&date.date_naive())?;
            return Ok(idx);
        }

        // If that didn't work as well then the ID is invalid.
        Err(AppError::IndexRetrievalError("Failed to match index with node".to_string()))
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
}
