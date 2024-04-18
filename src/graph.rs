use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum ErrorType {
    #[error("Invalid index: {0}")]
    InvalidIndex(usize),

    #[error("Invalid alias: {0}")]
    InvalidAlias(String),

    #[error("Graph looped: {0}")]
    GraphLooped(usize),
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct TaskGraph {
    data: Vec<Option<RefCell<TaskNode>>>,
    roots: Vec<usize>,
    aliases: HashMap<String, usize>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct TaskNode {
    message: String,
    state: TaskState,
    index: usize,
    alias: Option<String>,
    parents: Vec<usize>,
    children: Vec<usize>,
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum TaskState {
    #[default]
    None,
    Partial,
    Complete,
    /// Does not count to completion
    Pseudo,
}

impl TaskGraph {
    pub fn new() -> Self {
        Self {
            data: vec![],
            roots: vec![],
            aliases: HashMap::new(),
        }
    }

    pub fn insert_raw(&mut self, node: TaskNode) {
        self.data.push(Some(RefCell::new(node)));
    }

    pub fn insert_root(&mut self, message: String, pseudo: bool) {
        let idx = self.data.len();
        let mut node = TaskNode::new(message, idx);
        if pseudo {
            node.state = TaskState::Pseudo;
        }
        self.data.push(Some(RefCell::new(node)));
        self.roots.push(idx);
    }

    pub fn insert_child_unchecked(&mut self, message: String, parent: usize, pseudo: bool) {
        let idx = self.data.len();
        let mut node = TaskNode::new(message, idx);
        if pseudo {
            node.state = TaskState::Pseudo
        }
        self.data.push(Some(RefCell::new(node)));
        self.link_unchecked(parent, idx);
    }

    pub fn insert_child(
        &mut self,
        message: String,
        parent: String,
        pseudo: bool,
    ) -> Result<(), ErrorType> {
        let parent = self.parse_alias(&parent)?;
        self.check_index(parent)?;
        self.insert_child_unchecked(message, parent, pseudo);
        Ok(())
    }

    pub fn remove(&mut self, target: String) -> Result<()> {
        let index = self.parse_alias(&target)?;
        self.check_index(index)?;

        // Remove node if it was root
        self.roots.retain(|i| *i != index);

        // Unset alias
        self.unset_alias(target)?;

        // Unlink node from parents and children
        let parents_ptr = self.data[index].as_ref().unwrap().borrow().parents.as_ptr();
        let parents_len = self.data[index].as_ref().unwrap().borrow().parents.len();
        for i in 0..parents_len {
            let parent = unsafe { *parents_ptr.add(i) };
            self.data[parent]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .children
                .retain(|i| *i != index);
        }
        let children_ptr = self.data[index]
            .as_ref()
            .unwrap()
            .borrow()
            .children
            .as_ptr();
        let children_len = self.data[index].as_ref().unwrap().borrow().children.len();
        for i in 0..children_len {
            let child = unsafe { *children_ptr.add(i) };
            self.data[child]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .parents
                .retain(|i| *i != index);
            if self.data[child]
                .as_ref()
                .unwrap()
                .borrow()
                .parents
                .is_empty()
            {
                self.roots.push(child); // Since they're now parentless, make them root
            }
        }

        self.data[index] = None;

        Ok(())
    }

    pub fn remove_children_recursive(&mut self, target: String) -> Result<()> {
        let index = self.parse_alias(&target)?;
        self.check_index(index)?;
        self.roots.retain(|i| *i != index);
        self._remove_children_recursive(index)?;
        Ok(())
    }

    fn _remove_children_recursive(&mut self, index: usize) -> Result<()> {
        let parents_ptr = self.data[index].as_ref().unwrap().borrow().parents.as_ptr();
        let parents_len = self.data[index].as_ref().unwrap().borrow().parents.len();
        for i in 0..parents_len {
            let parent = unsafe { *parents_ptr.add(i) };
            self.data[parent]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .children
                .retain(|i| *i != index);
        }
        let children_ptr = self.data[index]
            .as_ref()
            .unwrap()
            .borrow()
            .children
            .as_ptr();
        let children_len = self.data[index].as_ref().unwrap().borrow().children.len();
        for i in 0..children_len {
            let child = unsafe { *children_ptr.add(i) };
            self.data[child]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .parents
                .retain(|i| *i != index);
            self._remove_children_recursive(child)?;
        }
        self.unset_alias_raw(index)?;
        self.data[index] = None;
        Ok(())
    }

    pub fn link_unchecked(&mut self, from: usize, to: usize) {
        self.data[from]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .children
            .push(to);
        self.data[to]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .parents
            .push(from);
        // Remove node from list of roots if it has a parent
        self.roots.retain(|i| *i != to);
    }

    pub fn link(&mut self, from: String, to: String) -> Result<()> {
        let from = self.parse_alias(&from)?;
        let to = self.parse_alias(&to)?;
        self.check_index(from)?;
        self.check_index(to)?;
        self.link_unchecked(from, to);
        Ok(())
    }

    pub fn unlink_unchecked(&mut self, from: usize, to: usize) {
        self.data[from]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .children
            .retain(|i| *i != to);
        self.data[to]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .parents
            .retain(|i| *i != from);
        // Add node to list of roots if it does not have a parent
        if self.data[to].as_ref().unwrap().borrow().parents.is_empty() {
            self.roots.push(to);
        }
    }

    pub fn unlink(&mut self, from: String, to: String) -> Result<()> {
        let from = self.parse_alias(&from)?;
        let to = self.parse_alias(&to)?;
        self.check_index(from)?;
        self.check_index(to)?;
        self.unlink_unchecked(from, to);
        Ok(())
    }

    pub fn check_index(&self, index: usize) -> Result<(), ErrorType> {
        if index > self.data.len() {
            return Err(ErrorType::InvalidIndex(index));
        }
        if self.data[index].is_none() {
            return Err(ErrorType::InvalidIndex(index));
        }
        Ok(())
    }

    /// Sets node state and propogates changes to children and parents
    pub fn set_state(&mut self, target: String, state: TaskState, propogate: bool) -> Result<()> {
        let index = self.parse_alias(&target)?;
        self.check_index(index)?;
        self.data[index].as_ref().unwrap().borrow_mut().state = state;
        if !propogate {
            return Ok(());
        }
        let children_ptr = self.data[index]
            .as_ref()
            .unwrap()
            .borrow()
            .children
            .as_ptr();
        let children_len = self.data[index].as_ref().unwrap().borrow().children.len();
        self.set_state_recurse_children(children_ptr, children_len, state)?;
        let parents_ptr = self.data[index].as_ref().unwrap().borrow().parents.as_ptr();
        let parents_len = self.data[index].as_ref().unwrap().borrow().parents.len();
        self.update_state_recurse_parents(parents_ptr, parents_len)?;
        Ok(())
    }

    // Absolute
    fn set_state_recurse_children(
        &mut self,
        indices: *const usize,
        len: usize,
        state: TaskState,
    ) -> Result<()> {
        for i in 0..len {
            let i = unsafe { *indices.add(i) };

            if self.data[i].as_ref().unwrap().borrow().state == TaskState::Pseudo {
                continue;
            }
            self.data[i].as_ref().unwrap().borrow_mut().state = state;

            let children_ptr = self.data[i].as_ref().unwrap().borrow().children.as_ptr();
            let children_len = self.data[i].as_ref().unwrap().borrow().children.len();
            self.set_state_recurse_children(children_ptr, children_len, state)?;

            let parents_ptr = self.data[i].as_ref().unwrap().borrow().parents.as_ptr();
            let parents_len = self.data[i].as_ref().unwrap().borrow().parents.len();
            self.update_state_recurse_parents(parents_ptr, parents_len)?;
        }
        Ok(())
    }

    // Check individually (because partially completed state)
    fn update_state_recurse_parents(&mut self, indices: *const usize, len: usize) -> Result<()> {
        for i in 0..len {
            let i = unsafe { *indices.add(i) };
            let mut count = 0;
            let mut pseudo = 0;
            let mut partial = false;
            for child in self.data[i].as_ref().unwrap().borrow().children.iter() {
                match self.data[*child].as_ref().unwrap().borrow().state {
                    TaskState::None => continue,
                    TaskState::Partial => {
                        partial = true;
                    }
                    TaskState::Complete => {
                        partial = true;
                        count += 1;
                    }
                    TaskState::Pseudo => {
                        pseudo += 1;
                    }
                }
            }
            let is_pseudo = self.data[i].as_ref().unwrap().borrow().state == TaskState::Pseudo;
            self.data[i].as_ref().unwrap().borrow_mut().state = if is_pseudo {
                TaskState::Pseudo
            // Every child task is completed
            } else if count != 0
                && count == (self.data[i].as_ref().unwrap().borrow().children.len() - pseudo)
            {
                TaskState::Complete
            // At least one child task is completed or partially completed
            } else if partial {
                TaskState::Partial
            } else {
                TaskState::None
            };
            if is_pseudo {
                // No need to recurse for pseudo nodes as they do not affect parent status
                continue;
            }
            let parents_ptr = self.data[i].as_ref().unwrap().borrow().parents.as_ptr();
            let parents_len = self.data[i].as_ref().unwrap().borrow().parents.len();
            self.update_state_recurse_parents(parents_ptr, parents_len)?;
        }
        Ok(())
    }

    pub fn rename_node(&mut self, target: String, message: String) -> Result<()> {
        let index = self.parse_alias(&target)?;
        self.check_index(index)?;
        self.data[index].as_ref().unwrap().borrow_mut().message = message;
        Ok(())
    }

    /// Remaps old indices to new indices
    /// Compress by ignoring unused indices
    ///
    /// Example:
    /// Old [Some(a), None, Some(b), Some(c), None, Some(d)]
    /// Map indices [0->0, 2->1, 3->2, 4->3]
    /// New [Some(a), Some(b), Some(c), Some(d)]
    pub fn clean(&mut self) -> Result<Self> {
        let mut map = Vec::with_capacity(self.data.len()); // Map old indices to new indices
        let mut last_used_index: usize = 0;
        for (i, node) in self.data.iter().enumerate() {
            match node {
                None => map.push((i, None)), // Ignored sentinel value
                Some(_) => {
                    map.push((i, Some(last_used_index)));
                    last_used_index += 1;
                }
            }
        }

        let mut new_graph = TaskGraph::new();
        for node in self.data.iter() {
            match node {
                None => continue,
                Some(node) => {
                    let mut new_node = node.borrow().clone();
                    new_node.map_indices(&map);
                    if let Some(ref alias) = new_node.alias {
                        let old = self.aliases.get(alias).unwrap();
                        *self.aliases.get_mut(alias).unwrap() = map[*old].1.unwrap();
                    }
                    new_graph.insert_raw(new_node);
                }
            }
        }
        // Add roots
        for r in self.roots.iter() {
            new_graph.roots.push(map[*r].1.unwrap());
        }

        println!(
            "Cleaned {} indices. Old count: {}, New count: {}",
            map.len() - new_graph.data.len(),
            map.len(),
            new_graph.data.len()
        );

        Ok(new_graph)
    }

    pub fn list_children(&self, target: String, max_depth: u32) -> Result<()> {
        let index = self.parse_alias(&target)?;
        self.check_index(index)?;
        // Display self as well
        println!("{}", self.data[index].as_ref().unwrap().borrow());
        self.list_recurse(
            self.data[index]
                .as_ref()
                .unwrap()
                .borrow()
                .children
                .as_slice(),
            max_depth,
            1,
            Some(index),
        )?;
        Ok(())
    }

    pub fn list_roots(&self) -> Result<()> {
        let roots = &self.roots;
        self.list_recurse(roots, 1, 1, None)?;
        Ok(())
    }

    fn list_recurse(
        &self,
        indices: &[usize],
        max_depth: u32,
        depth: u32,
        start: Option<usize>,
    ) -> Result<(), ErrorType> {
        // A sentinel value of -1 means infinite depth
        if max_depth != 0 && depth > max_depth {
            return Ok(());
        }

        for i in indices {
            if let Some(start) = start {
                if *i == start {
                    return Err(ErrorType::GraphLooped(start));
                }
            }
            Self::print_tree_indent(
                depth,
                self.data[*i].as_ref().unwrap().borrow().parents.len() > 1,
            );
            println!("{}", self.data[*i].as_ref().unwrap().borrow());
            self.list_recurse(
                self.data[*i].as_ref().unwrap().borrow().children.as_slice(),
                max_depth,
                depth + 1,
                start,
            )?;
        }
        Ok(())
    }

    fn print_tree_indent(depth: u32, dots: bool) {
        if depth == 0 {
            return;
        }

        for _ in 0..(depth - 1) {
            print!(" |   ");
        }
        if dots {
            print!(" + ..");
        } else {
            print!(" +---");
        }
    }

    pub fn display_stats(&self, target: String) -> Result<()> {
        let index = self.parse_alias(&target)?;
        self.check_index(index)?;
        let node = self.data[index].as_ref().unwrap().borrow();
        println!("Message : {}", &node.message);
        println!("Parents :");
        for i in &node.parents {
            let parent = self.data[*i].as_ref().unwrap().borrow();
            println!("({}) [{}]", parent.index, parent.state);
        }
        println!("Children:");
        for i in &node.children {
            let child = self.data[*i].as_ref().unwrap().borrow();
            println!("({}) [{}]", child.index, child.state);
        }
        println!("Status  : [{}]", node.state);
        Ok(())
    }

    pub fn parse_alias(&self, alias: &String) -> Result<usize, ErrorType> {
        self.aliases
            .get(alias)
            .copied()
            .or(alias.parse::<usize>().ok())
            .ok_or(ErrorType::InvalidAlias(alias.to_owned()))
    }

    pub fn set_alias(&mut self, target: String, alias: String) -> Result<()> {
        let index = self.parse_alias(&target)?;
        self.check_index(index)?;
        self.aliases.insert(alias.to_owned(), index);
        self.data[index].as_ref().unwrap().borrow_mut().alias = Some(alias);
        Ok(())
    }

    pub fn unset_alias_raw(&mut self, index: usize) -> Result<()> {
        self.check_index(index)?;
        let alias = match self.data[index].as_ref().unwrap().borrow().alias {
            None => return Ok(()),
            Some(ref alias) => alias.clone(),
        };
        self.unset_alias(alias)?;
        Ok(())
    }

    pub fn list_aliases(&self) -> Result<()> {
        for i in self.aliases.values() {
            println!("{}", self.data[*i].as_ref().unwrap().borrow());
        }
        Ok(())
    }

    pub fn unset_alias(&mut self, target: String) -> Result<()> {
        if let Some(index) = self.aliases.remove(&target) {
            self.data[index].as_ref().unwrap().borrow_mut().alias = None;
        }
        Ok(())
    }
}

impl TaskNode {
    fn new(message: String, index: usize) -> Self {
        Self {
            message,
            state: TaskState::None,
            index,
            alias: None,
            parents: vec![],
            children: vec![],
        }
    }

    fn map_indices(&mut self, map: &[(usize, Option<usize>)]) {
        self.index = map[self.index].1.unwrap();
        for i in self.parents.iter_mut() {
            *i = map[*i].1.unwrap();
        }
        for i in self.children.iter_mut() {
            *i = map[*i].1.unwrap();
        }
    }
}

impl fmt::Display for TaskNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index = if let Some(ref alias) = self.alias {
            format!("{}:{}", self.index, alias)
        } else {
            format!("{}", self.index)
        };
        write!(f, "[{}] {} ({})", self.state, self.message, index)
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskState::None => write!(f, " "),
            TaskState::Partial => write!(f, "~"),
            TaskState::Complete => write!(f, "x"),
            TaskState::Pseudo => write!(f, "+"),
        }
    }
}
