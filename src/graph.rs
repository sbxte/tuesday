use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use anyhow::Result;
use chrono::{Days, Local, NaiveDate};
use clap::ValueEnum;
use colored::Colorize;
use nom::IResult;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum ErrorType {
    #[error("Invalid index: {0}")]
    InvalidIndex(usize),

    #[error("Invalid alias: {0}")]
    InvalidAlias(String),

    #[error("Invalid date: {0}")]
    InvalidDate(String),

    #[error("Malformed date string: {0}")]
    MalformedDate(String),

    #[error("Graph looped back: {0}->...->{1}->{0}")]
    GraphLooped(usize, usize),
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Graph {
    nodes: Vec<Option<RefCell<Node>>>,
    roots: Vec<usize>,
    dates: HashMap<String, usize>,
    aliases: HashMap<String, usize>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Node {
    message: String,
    state: NodeState,
    index: usize,
    alias: Option<String>,
    parents: Vec<usize>,
    children: Vec<usize>,
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum NodeState {
    #[default]
    None,
    Partial,
    Complete,
    /// Does not count to completion
    Pseudo,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            roots: vec![],
            dates: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    pub fn alias_count(&self) -> usize {
        self.aliases.len()
    }

    pub fn insert_raw(&mut self, node: Node) {
        self.nodes.push(Some(RefCell::new(node)));
    }

    pub fn insert_root(&mut self, message: String, pseudo: bool) {
        let idx = self.nodes.len();
        let mut node = Node::new(message, idx);
        if pseudo {
            node.state = NodeState::Pseudo;
        }
        self.nodes.push(Some(RefCell::new(node)));
        self.roots.push(idx);
    }

    pub fn insert_date(&mut self, date: String) -> usize {
        let idx = self.nodes.len();
        let node = Node::new(date.clone(), idx);
        self.nodes.push(Some(RefCell::new(node)));
        self.dates.insert(date, idx);
        idx
    }

    pub fn insert_child_unchecked(
        &mut self,
        message: String,
        parent: usize,
        pseudo: bool,
    ) -> usize {
        let idx = self.nodes.len();
        let mut node = Node::new(message, idx);
        if pseudo {
            node.state = NodeState::Pseudo
        }
        self.nodes.push(Some(RefCell::new(node)));
        self.link_unchecked(parent, idx);
        idx
    }

    pub fn insert_child(&mut self, message: String, parent: String, pseudo: bool) -> Result<()> {
        let parent = self.get_index(&parent)?;
        self.check_index(parent)?;
        let idx = self.insert_child_unchecked(message, parent, pseudo);
        Self::display_link(parent, idx, true);
        if !pseudo {
            self.update_state_recurse_parents(&[parent] as *const _, 1)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, target: String) -> Result<()> {
        let index = self.get_index(&target)?;
        self.check_index(index)?;

        // Remove node if it was root
        self.roots.retain(|i| *i != index);

        // Unset alias
        self.unset_alias(target)?;

        // Unlink node from parents and children
        let parents_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .parents
            .as_ptr();
        let parents_len = self.nodes[index].as_ref().unwrap().borrow().parents.len();
        for i in 0..parents_len {
            let parent = unsafe { *parents_ptr.add(i) };
            self.nodes[parent]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .children
                .retain(|i| *i != index);
        }
        self.update_state_recurse_parents(parents_ptr, parents_len)?;

        let children_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .children
            .as_ptr();
        let children_len = self.nodes[index].as_ref().unwrap().borrow().children.len();
        for i in 0..children_len {
            let child = unsafe { *children_ptr.add(i) };
            self.nodes[child]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .parents
                .retain(|i| *i != index);
            if self.nodes[child]
                .as_ref()
                .unwrap()
                .borrow()
                .parents
                .is_empty()
            {
                self.roots.push(child); // Since they're now parentless, make them root
            }
        }

        self.nodes[index] = None;

        Ok(())
    }

    pub fn remove_children_recursive(&mut self, target: String) -> Result<()> {
        let index = self.get_index(&target)?;
        self.check_index(index)?;
        self.roots.retain(|i| *i != index);
        self._remove_children_recursive(index)?;
        Ok(())
    }

    fn _remove_children_recursive(&mut self, index: usize) -> Result<()> {
        let parents_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .parents
            .as_ptr();
        let parents_len = self.nodes[index].as_ref().unwrap().borrow().parents.len();
        for i in 0..parents_len {
            let parent = unsafe { *parents_ptr.add(i) };
            self.nodes[parent]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .children
                .retain(|i| *i != index);
        }
        self.update_state_recurse_parents(parents_ptr, parents_len)?;
        let children_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .children
            .as_ptr();
        let children_len = self.nodes[index].as_ref().unwrap().borrow().children.len();
        for i in 0..children_len {
            let child = unsafe { *children_ptr.add(i) };
            self.nodes[child]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .parents
                .retain(|i| *i != index);
            self._remove_children_recursive(child)?;
        }
        self.unset_alias_raw(index)?;
        self.nodes[index] = None;
        Ok(())
    }

    pub fn link_unchecked(&mut self, from: usize, to: usize) {
        self.nodes[from]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .children
            .push(to);
        self.nodes[to]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .parents
            .push(from);
        // Remove node from list of roots if it has a parent
        self.roots.retain(|i| *i != to);
    }

    pub fn link(&mut self, from: String, to: String) -> Result<()> {
        let from = self.get_index(&from)?;
        let to = self.get_index(&to)?;
        self.check_index(from)?;
        self.check_index(to)?;
        self.link_unchecked(from, to);
        Self::display_link(from, to, true);
        Ok(())
    }

    pub fn display_link(from: usize, to: usize, connect: bool) {
        let from = format!("({})", from).bright_blue();
        let to = format!("({})", to).bright_blue();
        if connect {
            println!("{} -> {}", from, to);
        } else {
            println!("{} -x- {}", from, to);
        }
    }

    pub fn unlink_unchecked(&mut self, from: usize, to: usize) {
        self.nodes[from]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .children
            .retain(|i| *i != to);
        self.nodes[to]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .parents
            .retain(|i| *i != from);
        // Add node to list of roots if it does not have a parent
        if self.nodes[to].as_ref().unwrap().borrow().parents.is_empty() {
            self.roots.push(to);
        }
    }

    pub fn unlink(&mut self, from: String, to: String) -> Result<()> {
        let from = self.get_index(&from)?;
        let to = self.get_index(&to)?;
        self.check_index(from)?;
        self.check_index(to)?;
        self.unlink_unchecked(from, to);
        Self::display_link(from, to, true);
        Ok(())
    }

    pub fn check_index(&self, index: usize) -> Result<(), ErrorType> {
        if index > self.nodes.len() {
            return Err(ErrorType::InvalidIndex(index));
        }
        if self.nodes[index].is_none() {
            return Err(ErrorType::InvalidIndex(index));
        }
        Ok(())
    }

    /// Sets node state and propogates changes to children and parents
    pub fn set_state(&mut self, target: String, state: NodeState, propogate: bool) -> Result<()> {
        let index = self.get_index(&target)?;
        self.check_index(index)?;
        self.nodes[index].as_ref().unwrap().borrow_mut().state = state;
        if !propogate {
            return Ok(());
        }
        let children_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .children
            .as_ptr();
        let children_len = self.nodes[index].as_ref().unwrap().borrow().children.len();
        self.set_state_recurse_children(children_ptr, children_len, state)?;
        let parents_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .parents
            .as_ptr();
        let parents_len = self.nodes[index].as_ref().unwrap().borrow().parents.len();
        self.update_state_recurse_parents(parents_ptr, parents_len)?;
        Ok(())
    }

    // Absolute
    fn set_state_recurse_children(
        &mut self,
        indices: *const usize,
        len: usize,
        state: NodeState,
    ) -> Result<()> {
        for i in 0..len {
            let i = unsafe { *indices.add(i) };

            if self.nodes[i].as_ref().unwrap().borrow().state == NodeState::Pseudo {
                continue;
            }
            self.nodes[i].as_ref().unwrap().borrow_mut().state = state;

            let children_ptr = self.nodes[i].as_ref().unwrap().borrow().children.as_ptr();
            let children_len = self.nodes[i].as_ref().unwrap().borrow().children.len();
            self.set_state_recurse_children(children_ptr, children_len, state)?;

            let parents_ptr = self.nodes[i].as_ref().unwrap().borrow().parents.as_ptr();
            let parents_len = self.nodes[i].as_ref().unwrap().borrow().parents.len();
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
            for child in self.nodes[i].as_ref().unwrap().borrow().children.iter() {
                match self.nodes[*child].as_ref().unwrap().borrow().state {
                    NodeState::None => continue,
                    NodeState::Partial => {
                        partial = true;
                    }
                    NodeState::Complete => {
                        partial = true;
                        count += 1;
                    }
                    NodeState::Pseudo => {
                        pseudo += 1;
                    }
                }
            }
            let is_pseudo = self.nodes[i].as_ref().unwrap().borrow().state == NodeState::Pseudo;
            self.nodes[i].as_ref().unwrap().borrow_mut().state = if is_pseudo {
                NodeState::Pseudo
            // Every child task is completed
            } else if count != 0
                && count == (self.nodes[i].as_ref().unwrap().borrow().children.len() - pseudo)
            {
                NodeState::Complete
            // At least one child task is completed or partially completed
            } else if partial {
                NodeState::Partial
            } else {
                NodeState::None
            };
            if is_pseudo {
                // No need to recurse for pseudo nodes as they do not affect parent status
                continue;
            }
            let parents_ptr = self.nodes[i].as_ref().unwrap().borrow().parents.as_ptr();
            let parents_len = self.nodes[i].as_ref().unwrap().borrow().parents.len();
            self.update_state_recurse_parents(parents_ptr, parents_len)?;
        }
        Ok(())
    }

    pub fn rename_node(&mut self, target: String, message: String) -> Result<()> {
        let index = self.get_index(&target)?;
        self.check_index(index)?;
        self.nodes[index].as_ref().unwrap().borrow_mut().message = message;
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
        let mut map = Vec::with_capacity(self.nodes.len()); // Map old indices to new indices
        let mut last_used_index: usize = 0;
        for (i, node) in self.nodes.iter().enumerate() {
            match node {
                None => map.push((i, None)), // Ignored sentinel value
                Some(_) => {
                    map.push((i, Some(last_used_index)));
                    last_used_index += 1;
                }
            }
        }

        let mut new_graph = Graph::new();
        // Map indices
        for node in self.nodes.iter() {
            match node {
                None => continue,
                Some(node) => {
                    let mut new_node = node.borrow().clone();
                    new_node.map_indices(&map);
                    new_graph.insert_raw(new_node);
                }
            }
        }

        // Map aliases
        for (alias, idx) in self.aliases.iter() {
            new_graph
                .aliases
                .insert(alias.clone(), map[*idx].1.unwrap());
        }

        // Add roots
        for r in self.roots.iter() {
            new_graph.roots.push(map[*r].1.unwrap());
        }

        println!(
            "Cleaned {} indices. Old count: {}, New count: {}",
            map.len() - new_graph.nodes.len(),
            map.len(),
            new_graph.nodes.len()
        );

        Ok(new_graph)
    }

    pub fn list_children(&self, target: String, max_depth: u32) -> Result<()> {
        let index = self.get_index(&target)?;
        self.check_index(index)?;
        // Display self as well
        println!("{}", self.nodes[index].as_ref().unwrap().borrow());
        self.list_recurse(
            self.nodes[index]
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

    pub fn list_dates(&self) -> Result<()> {
        let x: Vec<_> = self.dates.values().copied().collect();
        self.list_recurse(x.as_slice(), 1, 1, None)?;
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
                    return Err(ErrorType::GraphLooped(start, *i));
                }
            }
            Self::print_tree_indent(
                depth,
                self.nodes[*i].as_ref().unwrap().borrow().parents.len() > 1,
            );
            println!("{}", self.nodes[*i].as_ref().unwrap().borrow());
            self.list_recurse(
                self.nodes[*i]
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .children
                    .as_slice(),
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
            print!(" |  ");
        }
        if dots {
            print!(" +..");
        } else {
            print!(" +--");
        }
    }

    pub fn display_stats(&self, target: String) -> Result<()> {
        let index = self.get_index(&target)?;
        self.check_index(index)?;
        let node = self.nodes[index].as_ref().unwrap().borrow();
        println!("Message : {}", &node.message);
        println!("Parents :");
        for i in &node.parents {
            let parent = self.nodes[*i].as_ref().unwrap().borrow();
            println!("({}) [{}]", parent.index, parent.state);
        }
        println!("Children:");
        for i in &node.children {
            let child = self.nodes[*i].as_ref().unwrap().borrow();
            println!("({}) [{}]", child.index, child.state);
        }
        println!("Status  : [{}]", node.state);
        Ok(())
    }

    /// Returns the node index based on a target string.
    /// The target string may be an a date, an alias, or an index
    pub fn get_index(&self, target: &str) -> Result<usize> {
        if Self::is_date(target) {
            return match self.dates.get(target) {
                Some(x) => Ok(*x),
                None => Err(ErrorType::InvalidDate(target.to_owned()))?,
            };
        }
        if Self::is_relative_date(target) {
            let date = Self::parse_relative_date(target)?;
            if self.dates.contains_key(&date) {
                return Ok(*self.dates.get(&date).unwrap());
            }
        }
        Ok(self
            .aliases
            .get(target)
            .copied()
            .or(target.parse::<usize>().ok())
            .ok_or(ErrorType::InvalidAlias(target.to_owned()))?)
    }

    // TODO: Is this needed? Should link use this instead?
    #[allow(dead_code)]
    pub fn get_or_insert_index(&mut self, target: &str) -> Result<usize> {
        if Self::is_date(target) {
            return match self.dates.get(target) {
                Some(x) => Ok(*x),
                None => Err(ErrorType::InvalidDate(target.to_owned()))?,
            };
        }
        if Self::is_relative_date(target) {
            let date = Self::parse_relative_date(target)?;
            return if self.dates.contains_key(&date) {
                Ok(*self.dates.get(&date).unwrap())
            } else {
                Ok(self.insert_date(date))
            };
        }
        self.aliases
            .get(target)
            .copied()
            .or(target.parse::<usize>().ok())
            .ok_or(Err(ErrorType::InvalidAlias(target.to_owned()))?)
    }

    /// Format
    /// YYYY-MM-DD
    pub fn is_date(s: &str) -> bool {
        return match Self::_parse_date(s) {
            // TODO: Should negative years be allowed??? Who would want to schedule something as
            // far back as the BCEs ????
            Ok((_, (y, m, d))) => NaiveDate::from_ymd_opt(y as i32, m, d).is_some(),
            Err(_) => false,
        };
    }

    fn _parse_date(s: &str) -> IResult<&str, (u32, u32, u32)> {
        use nom::bytes::complete::tag;
        use nom::character::complete::digit1;
        use nom::combinator::map_res;
        let (s, year): (&str, u32) = map_res(digit1, |s: &str| s.parse::<u32>())(s)?;
        let (s, _) = tag("-")(s)?;
        let (s, month) = map_res(digit1, |s: &str| s.parse::<u32>())(s)?;
        let (s, _) = tag("-")(s)?;
        let (_, day) = map_res(digit1, |s: &str| s.parse::<u32>())(s)?;
        Ok(("", (year, month, day)))
    }

    pub fn is_relative_date(s: &str) -> bool {
        s == "today" || s == "tomorrow" || s == "yesterday"
    }

    pub fn parse_relative_date(s: &str) -> Result<String> {
        match s {
            "today" => Ok(Self::format_naivedate(Local::now().date_naive())),
            "tomorrow" => Ok(Self::format_naivedate(
                Local::now()
                    .checked_add_days(Days::new(1))
                    .unwrap()
                    .date_naive(),
            )),
            "yesterday" => Ok(Self::format_naivedate(
                Local::now()
                    .checked_sub_days(Days::new(1))
                    .unwrap()
                    .date_naive(),
            )),
            _ => panic!("Invalid branch on parse_relative_date!"),
        }
    }

    pub fn format_naivedate(date: NaiveDate) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    pub fn set_alias(&mut self, target: String, alias: String) -> Result<()> {
        let index = self.get_index(&target)?;
        self.check_index(index)?;
        self.aliases.insert(alias.clone(), index);
        self.nodes[index].as_ref().unwrap().borrow_mut().alias = Some(alias);
        Ok(())
    }

    pub fn unset_alias_raw(&mut self, index: usize) -> Result<()> {
        self.check_index(index)?;
        let alias = match self.nodes[index].as_ref().unwrap().borrow().alias {
            None => return Ok(()),
            Some(ref alias) => alias.clone(),
        };
        self.unset_alias(alias)?;
        Ok(())
    }

    pub fn list_aliases(&self) -> Result<()> {
        for i in self.aliases.values() {
            println!("{}", self.nodes[*i].as_ref().unwrap().borrow());
        }
        Ok(())
    }

    pub fn unset_alias(&mut self, target: String) -> Result<()> {
        if let Some(index) = self.aliases.remove(&target) {
            self.nodes[index].as_ref().unwrap().borrow_mut().alias = None;
        }
        Ok(())
    }
}

impl Node {
    fn new(message: String, index: usize) -> Self {
        Self {
            message,
            state: NodeState::None,
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
            NodeState::Complete => write!(f, "x"),
            NodeState::Pseudo => write!(f, "+"),
        }
    }
}
