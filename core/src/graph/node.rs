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
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum NodeState {
    #[default]
    None,
    Partial,
    Done,
    /// Does not count to completion
    Pseudo,
}

impl Node {
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

    pub fn map_indices(&mut self, map: &[(usize, Option<usize>)]) {
        self.index = map[self.index].1.unwrap();
        for i in self.parents.iter_mut() {
            *i = map[*i].1.unwrap();
        }
        for i in self.children.iter_mut() {
            *i = map[*i].1.unwrap();
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
        let state = format!("{}{}{}", "[".bright_blue(), self.state, "]".bright_blue());
        write!(f, "{} {} {}", state, self.message, index)
    }
}

impl fmt::Display for NodeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeState::None => write!(f, " "),
            NodeState::Partial => write!(f, "~"),
            NodeState::Done => write!(f, "x"),
            NodeState::Pseudo => write!(f, "+"),
        }
    }
}
