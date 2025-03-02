use std::fmt;

use clap::ValueEnum;
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Node {
    pub message: String,
    pub r#type: NodeType,
    pub state: NodeState,
    pub archived: bool,
    pub index: usize,
    pub alias: Option<String>,
    pub parents: Vec<usize>,
    pub children: Vec<usize>,
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum NodeType {
    #[default]
    Normal,
    Date,
    /// Does not count to completion
    Pseudo,
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum NodeState {
    #[default]
    None,
    Partial,
    Done,
}

impl Node {
    /// Creates a new node from a message, an index, and a node type
    pub fn new(message: String, index: usize, r#type: NodeType) -> Self {
        Self {
            message,
            r#type,
            state: NodeState::None,
            archived: false,
            index,
            alias: None,
            parents: vec![],
            children: vec![],
        }
    }

    /// Maps the locally stored indices (self, parents, and children) using a slice
    /// Where an index `i` gets mapped into a `map[i]` where `map[i]` **MUST BE** a `Some(usize)`
    pub fn map_indices(&mut self, map: &[Option<usize>]) {
        self.index = map[self.index].unwrap();
        for i in self.parents.iter_mut() {
            *i = map[*i].unwrap();
        }
        for i in self.children.iter_mut() {
            *i = map[*i].unwrap();
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index = if let Some(ref alias) = self.alias {
            format!("({}:{})", self.index, alias)
        } else {
            format!("({})", self.index)
        }
        .bright_blue();
        match self.r#type {
            NodeType::Normal => {
                let state = format!("{}{}{}", "[".bright_blue(), self.state, "]".bright_blue());
                write!(f, "{} {} {}", state, self.message, index)
            }
            NodeType::Date => {
                let state = format!("{}{}{}", "(".bright_blue(), self.state, ")".bright_blue());
                write!(f, "{} {} {}", state, self.message, index)
            }
            NodeType::Pseudo => {
                write!(f, "{} {} {}", "(+>".bright_blue(), self.message, index)
            }

        }
    }
}

impl fmt::Display for NodeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeState::None => write!(f, " "),
            NodeState::Partial => write!(f, "~"),
            NodeState::Done => write!(f, "x"),
        }
    }
}
