use date::DateData;
use serde::{Deserialize, Serialize};
use task::TaskData;

pub mod date;
pub mod task;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Node {
    pub title: String,
    pub data: NodeType,
    pub metadata: NodeMetadata,
}

impl Node {
    /// Creates a new node from a message, an index, and a node type
    pub fn new(message: String, index: usize, data: NodeType) -> Self {
        Self {
            title: message,
            data,
            metadata: NodeMetadata::new(index),
        }
    }

    /// Maps the locally stored indices (self, parents, and children) using a slice
    /// Where an index `i` gets mapped into a `map[i]` where `map[i]` **MUST BE** a `Some(usize)`
    pub fn map_indices(&mut self, map: &[Option<usize>]) {
        self.metadata.index = map[self.metadata.index].unwrap();
        for i in self.metadata.parents.iter_mut() {
            *i = map[*i].unwrap();
        }
        for i in self.metadata.children.iter_mut() {
            *i = map[*i].unwrap();
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Task(task::TaskData),
    Date(date::DateData),
    /// Does not count to completion
    Pseudo,
}

impl NodeType {
    /// Returns whether this is a task node
    pub fn is_task(&self) -> bool {
        matches!(self, NodeType::Task(_))
    }

    /// Returns this type as a task node. Returns [`None`] if type is not [`NodeType::Task`]
    pub fn as_task(&self) -> Option<&TaskData> {
        match self {
            NodeType::Task(data) => Some(data),
            _ => None,
        }
    }

    /// Returns this type as a mutable task node. Returns [`None`] if type is not [`NodeType::Task`]
    pub fn as_task_mut(&mut self) -> Option<&mut TaskData> {
        match self {
            NodeType::Task(data) => Some(data),
            _ => None,
        }
    }

    /// Returns whether this is a date node
    pub fn is_date(&self) -> bool {
        matches!(self, NodeType::Date(_))
    }

    /// Returns this type as a date node. Returns [`None`] if type is not [`NodeType::Date`]
    pub fn as_date(&self) -> Option<&DateData> {
        match self {
            NodeType::Date(data) => Some(data),
            _ => None,
        }
    }

    /// Returns this type as a mutable date node. Returns [`None`] if type is not [`NodeType::Date`]
    pub fn as_date_mut(&mut self) -> Option<&mut DateData> {
        match self {
            NodeType::Date(data) => Some(data),
            _ => None,
        }
    }

    /// Returns whether this is a pseudo node
    pub fn is_pseudo(&self) -> bool {
        matches!(self, NodeType::Pseudo)
    }
}

impl Default for NodeType {
    fn default() -> Self {
        Self::Task(Default::default())
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub archived: bool,
    pub index: usize,
    pub alias: Option<String>,
    pub parents: Vec<usize>,
    pub children: Vec<usize>,
}

impl NodeMetadata {
    /// Constructs fresh node metadata from an index
    pub fn new(index: usize) -> Self {
        Self {
            archived: false,
            index,
            alias: None,
            parents: vec![],
            children: vec![],
        }
    }
}
