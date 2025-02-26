pub mod errors;
pub mod node;

use std::cell::RefCell;
use std::collections::HashMap;

use anyhow::Result;
use chrono::{Days, Local, NaiveDate};
use colored::Colorize;
use nom::IResult;
use serde::{Deserialize, Serialize};

use errors::ErrorType;
use node::{Node, NodeState, NodeType};

/// Result of graph operation.
type GraphResult<T> = Result<T, ErrorType>;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Graph {
    pub(crate) nodes: Vec<Option<RefCell<Node>>>,
    pub(crate) roots: Vec<usize>,
    pub(crate) archived: Vec<usize>,
    pub(crate) dates: HashMap<String, usize>,
    pub(crate) aliases: HashMap<String, usize>,
}

impl Graph {
    /// Instantiates an empty `Graph`
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            roots: vec![],
            archived: vec![],
            dates: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// Returns the number of alive nodes present in the graph
    /// Alive as in does NOT include deleted (`None`) nodes in the graph
    pub fn node_count(&self) -> usize {
        self.nodes
            .iter()
            .fold(0, |acc, item| if item.is_some() { acc + 1 } else { acc })
    }

    /// Returns the number of root nodes in the graph
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    /// Returns the number of node aliases in the graph
    pub fn alias_count(&self) -> usize {
        self.aliases.len()
    }

    /// Returns an immutable reference to the underlying nodes `Vec`
    pub fn get_nodes(&self) -> &Vec<Option<RefCell<Node>>> {
        &self.nodes
    }

    /// Returns an immutable reference to the underlying roots `Vec`
    pub fn get_roots(&self) -> &Vec<usize> {
        &self.roots
    }

    /// Returns an immutable reference to the underlying alias `HashMap`
    pub fn get_aliases(&self) -> &HashMap<String, usize> {
        &self.aliases
    }

    /// Returns an immutable reference to the underlying dates `HashMap`
    pub fn get_dates(&self) -> &HashMap<String, usize> {
        &self.dates
    }

    /// Returns an immutable reference to the underlying archived nodes `Vec`
    pub fn get_archived(&self) -> &Vec<usize> {
        &self.archived
    }

    /// Inserts a node into the graph and sets it as a root node
    pub fn insert_root(&mut self, message: String, pseudo: bool) {
        let idx = self.nodes.len();
        let mut node = Node::new(message, idx, NodeType::Normal);
        if pseudo {
            node.state = NodeState::Pseudo;
        }
        self.nodes.push(Some(RefCell::new(node)));
        self.roots.push(idx);
    }

    /// Inserts a date node into the graph
    pub fn insert_date(&mut self, date: String) -> usize {
        let idx = self.nodes.len();
        let node = Node::new(date.clone(), idx, NodeType::Date);
        self.nodes.push(Some(RefCell::new(node)));
        self.dates.insert(date, idx);
        idx
    }

    /// Inserts a node into the graph and sets it as a child of a parent node without updating the
    /// states of its parent
    ///
    /// The parent node is represented using its node index
    pub fn insert_child_unchecked(
        &mut self,
        message: String,
        parent: usize,
        pseudo: bool,
    ) -> usize {
        let idx = self.nodes.len();
        let mut node = Node::new(message, idx, NodeType::Normal);
        if pseudo {
            node.state = NodeState::Pseudo
        }
        self.nodes.push(Some(RefCell::new(node)));
        self.link_unchecked(parent, idx);
        idx
    }

    /// Inserts a node into the graph and sets it as a child of a parent node and updates the
    /// states of its parent
    ///
    /// The parent node is represented using its node index
    pub fn insert_child(
        &mut self,
        message: String,
        parent: usize,
        pseudo: bool,
    ) -> GraphResult<()> {
        self.insert_child_unchecked(message, parent, pseudo);
        if !pseudo {
            self.update_state_recurse_parents(&[parent] as *const _, 1)?;
        }
        Ok(())
    }

    /// Removes a node by `index`
    pub fn remove(&mut self, index: usize) -> GraphResult<()> {
        // Remove node if it was root
        self.roots.retain(|i| *i != index);

        // Unset alias
        let alias = self.nodes[index].as_ref().unwrap().borrow().alias.is_some();
        if alias {
            self.unset_alias(index)?;
        }

        // Delete from date hashmap first if node is a date root node
        let node_type = self.nodes[index].as_ref().unwrap().borrow().r#type;
        if node_type == NodeType::Date {
            let node_date = &self.nodes[index].as_ref().unwrap().borrow().message;
            self.dates.remove(node_date);
        }

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

    /// Removes a node from the graph and recursively removes its children, grandchildren, etc.
    pub fn remove_children_recursive(&mut self, index: usize) -> GraphResult<()> {
        self.roots.retain(|i| *i != index);
        self._remove_children_recursive(index)?;
        Ok(())
    }

    fn _remove_children_recursive(&mut self, index: usize) -> GraphResult<()> {
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

        let alias = self.nodes[index].as_ref().unwrap().borrow().alias.is_some();
        if alias {
            self.unset_alias(index)?;
        }

        // Delete from date hashmap first if node is a date root node
        let node_type = self.nodes[index].as_ref().unwrap().borrow().r#type;
        if node_type == NodeType::Date {
            let node_date = &self.nodes[index].as_ref().unwrap().borrow().message;
            self.dates.remove(node_date);
        }

        self.nodes[index] = None;
        Ok(())
    }

    /// Connects two nodes on the graph with an edge
    /// Does NOT update parent states
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

    /// Connects two nodes on the graph with an edge
    /// And updates the parents' states recursively
    pub fn link(&mut self, from: usize, to: usize) -> GraphResult<()> {
        self.link_unchecked(from, to);

        // Update parent completion
        let parents_ptr = self.nodes[to].as_ref().unwrap().borrow().parents.as_ptr();
        let parents_len = self.nodes[to].as_ref().unwrap().borrow().parents.len();
        self.update_state_recurse_parents(parents_ptr, parents_len)?;

        Ok(())
    }

    pub fn print_link(from: usize, to: usize, connect: bool) {
        let from = format!("({})", from).bright_blue();
        let to = format!("({})", to).bright_blue();
        if connect {
            println!("{} -> {}", from, to);
        } else {
            println!("{} -x- {}", from, to);
        }
    }

    /// Unlinks two nodes on the graph without updating parent states
    fn unlink_unchecked(&mut self, from: usize, to: usize) {
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

    /// Unlinks two nodes on the graph
    /// And updates parent states
    pub fn unlink(&mut self, from: usize, to: usize) -> GraphResult<()> {
        let parents_ptr = self.nodes[to].as_ref().unwrap().borrow().parents.as_ptr();
        let parents_len = self.nodes[to].as_ref().unwrap().borrow().parents.len();
        self.unlink_unchecked(from, to);
        self.update_state_recurse_parents(parents_ptr, parents_len)?;
        Ok(())
    }

    /// Clear parents of target node and other nodes that hold the target as their child
    pub fn clean_parents(&mut self, index: usize) -> GraphResult<()> {
        let parents_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .parents
            .as_ptr();

        let parents_len = self.nodes[index].as_ref().unwrap().borrow().parents.len();

        self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .parents
            .iter()
            .for_each(|i| {
                self.nodes[*i]
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .children
                    .retain(|x| *x != index);
            });

        self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .parents
            .clear();

        self.update_state_recurse_parents(parents_ptr, parents_len)?;

        Ok(())
    }

    /// Sets node state and optionally propogates changes to children and parents
    pub fn set_state(
        &mut self,
        index: usize,
        state: NodeState,
        propogate: bool,
    ) -> GraphResult<()> {
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
    ) -> GraphResult<()> {
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
    fn update_state_recurse_parents(
        &mut self,
        indices: *const usize,
        len: usize,
    ) -> GraphResult<()> {
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
                    NodeState::Done => {
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
                NodeState::Done
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

    pub fn set_archived(&mut self, index: usize, archived: bool) -> GraphResult<()> {
        let status = &mut self.nodes[index].as_ref().unwrap().borrow_mut().archived;

        // Add to list of archived nodes if necessary
        if *status != archived {
            if archived {
                self.archived.push(index);
            } else {
                self.archived.retain(|i| *i != index);
            }
        }

        // Update node archive status
        *status = archived;

        Ok(())
    }

    /// Replaces a node on the graph's message with a new provided message
    pub fn rename_node(&mut self, index: usize, message: String) -> GraphResult<()> {
        self.nodes[index].as_ref().unwrap().borrow_mut().message = message;
        Ok(())
    }

    /// Fixes desynchronized or invalid links, and remaps nodes
    /// ~ The fixer-upper method
    pub fn clean(&mut self) {
        // Clears root tracked properties and resynchronizes them based on local node states
        self.dates.clear();
        self.aliases.clear();
        self.archived.clear();

        for node in self.nodes.iter() {
            if node.is_none() {
                continue;
            }
            let node = node.as_ref().unwrap();
            let rnode = node.borrow();

            // Add aliases, dates, and archival status
            if rnode.alias.is_some() {
                self.aliases
                    .insert(rnode.alias.as_ref().unwrap().clone(), rnode.index);
            }
            if Self::is_date(&rnode.message) {
                self.dates.insert(rnode.message.clone(), rnode.index);
            }
            if rnode.archived {
                self.archived.push(rnode.index);
            }

            // Remove invalid edges
            drop(rnode);
            let mut mnode = node.borrow_mut();
            mnode.parents.retain(|i| self.nodes[*i].is_some());
            mnode.children.retain(|i| self.nodes[*i].is_some());
        }

        // Add unreachable nodes into roots
        // Nodes without parents, are not root, and not date
        self.roots.clear();
        let date_values: Vec<_> = self.dates.values().collect();
        for (i, node) in self.nodes.iter().enumerate() {
            if node.is_none() {
                continue;
            }
            let node = node.as_ref().unwrap();
            let rnode = node.borrow();

            let parents = rnode.parents.len();
            if parents > 0 {
                continue;
            }
            let index = rnode.index;
            if !self.roots.contains(&index) && !date_values.contains(&&index) {
                self.roots.push(i);
            }
        }

        // Remaps old indices to new indices
        // Compress by ignoring unused indices
        //
        // Example:
        // Old [Some(a), None, Some(b), Some(c), None, Some(d)]
        // Map indices [0->0, 2->1, 3->2, 4->3]
        // New [Some(a), Some(b), Some(c), Some(d)]
        let mut map = Vec::with_capacity(self.nodes.len());
        let mut last_used_index: usize = 0;
        for node in &self.nodes {
            match node {
                None => map.push(None), // Ignored sentinel value
                Some(_) => {
                    map.push(Some(last_used_index));
                    last_used_index += 1;
                }
            }
        }

        // Add nodes, aliases, roots, dates, and archival status
        let mut new_graph = Graph::new();
        for node in self.nodes.iter() {
            match node {
                None => continue,
                Some(node) => {
                    let mut new_node = node.borrow().clone();
                    new_node.map_indices(&map);
                    new_graph.nodes.push(Some(RefCell::new(new_node)));
                }
            }
        }
        for (alias, idx) in self.aliases.iter() {
            new_graph.aliases.insert(alias.clone(), map[*idx].unwrap());
        }
        for r in self.roots.iter() {
            new_graph.roots.push(map[*r].unwrap());
        }
        for (d, i) in self.dates.iter() {
            new_graph.dates.insert(d.clone(), map[*i].unwrap());
        }
        for a in self.archived.iter() {
            new_graph.archived.push(map[*a].unwrap());
        }

        // Replace self with new cleaned and fixed graph
        *self = new_graph;
    }

    /// Call a closure that takes a node, with given index.
    pub fn with_node(&self, index: usize, f: &mut impl FnMut(&Node)) {
        let node = self.nodes[index].as_ref().unwrap().borrow();
        f(&node);
    }

    /// Traverse nodes recusively. Calls a closure on each node traversal that takes a reference to the current node and its nesting depth.
    pub fn traverse_recurse(
        &self,
        indices: &[usize],
        skip_archived: bool,
        max_depth: u32,
        depth: u32,
        start: Option<usize>,
        f: &mut impl FnMut(&Node, u32),
    ) -> GraphResult<()> {
        // A sentinel value of 0 means infinite depth
        if max_depth != 0 && depth > max_depth {
            return Ok(());
        }

        for i in indices {
            if let Some(start) = start {
                if *i == start {
                    return Err(ErrorType::GraphLooped(start, *i));
                }
            }

            // If theres no need to show archived nodes then ignore it and its children
            if !skip_archived && self.nodes[*i].as_ref().unwrap().borrow().archived {
                continue;
            }
            if let Some(node) = &self.nodes[*i] {
                f(&node.borrow(), depth);
            }

            self.traverse_recurse(
                self.nodes[*i]
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .children
                    .as_slice(),
                false,
                max_depth,
                depth + 1,
                start,
                f,
            )?;
        }
        Ok(())
    }

    /// Returns the node index based on a identifier string.
    /// The identifier string may be an a date, an alias, or an index
    pub fn get_index(&self, id: &str) -> GraphResult<usize> {
        // Check if it is a date
        if Self::is_date(id) {
            return match self.dates.get(id) {
                Some(x) => Ok(*x),
                None => Err(ErrorType::InvalidDate(id.to_owned()))?,
            };
        }
        if Self::is_relative_date(id) {
            let date = Self::parse_relative_date(id)?;
            if self.dates.contains_key(&date) {
                return Ok(*self.dates.get(&date).unwrap());
            }
        }
        // Check if it is an alias and if so return its corresponding index
        if let Some(x) = self.aliases.get(id) {
            return Ok(*x);
        }

        // Assume it is an index already, check for validity
        let index = id
            .parse::<usize>()
            .or(Err(ErrorType::MalformedIndex(id.to_string())))?;
        if index >= self.nodes.len() || self.nodes[index].is_none() {
            return Err(ErrorType::InvalidIndex(index));
        }
        Ok(index)
    }

    // TODO: Is this needed? Should link use this instead?
    pub fn get_or_insert_index(&mut self, target: &str) -> GraphResult<usize> {
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
        match Self::_parse_date(s) {
            Ok((_, (y, m, d))) => NaiveDate::from_ymd_opt(y as i32, m, d).is_some(),
            Err(_) => false,
        }
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

    /// Returns whether the provided string is a relative date
    ///
    /// The currently available relative dates are
    /// - today
    /// - tomorrow
    /// - yesterday
    pub fn is_relative_date(s: &str) -> bool {
        s == "today" || s == "tomorrow" || s == "yesterday"
    }

    /// Parses relative dates into NaiveDate format
    /// See [is_relative_date](Self::is_relative_date) for available relative dates
    /// See also [format_naivedate](Self::format_naivedate)
    pub fn parse_relative_date(s: &str) -> GraphResult<String> {
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

    /// Formats a [NaiveDate] into a [String]
    /// The format goes %Y-%m-%d
    /// See also [NaiveDate::format]
    pub fn format_naivedate(date: NaiveDate) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    /// Sets an alias for node at `index`
    pub fn set_alias(&mut self, index: usize, alias: String) -> GraphResult<()> {
        self.aliases.insert(alias.clone(), index);
        self.nodes[index].as_ref().unwrap().borrow_mut().alias = Some(alias);
        Ok(())
    }

    /// Unsets a node at `index`'s alias
    pub fn unset_alias(&mut self, index: usize) -> GraphResult<()> {
        let alias = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .alias
            .take()
            .unwrap();
        self.aliases.remove(alias.as_str());
        Ok(())
    }

    pub fn list_aliases(&self) -> GraphResult<()> {
        for i in self.aliases.values() {
            println!("{}", self.nodes[*i].as_ref().unwrap().borrow());
        }
        Ok(())
    }
}

/// Getters for external crates to obtain indices from private fields under `Graph`.
pub trait GraphGetters {
    fn get_node(&self, index: usize) -> Node;
    fn get_root_nodes_indices(&self) -> &[usize];
    fn get_archived_node_indices(&self) -> &[usize];

    // TODO: both these uses vectors, is it worth the performance cost?
    fn get_date_nodes_indices(&self) -> Vec<usize>;
    fn get_node_children(&self, index: usize) -> Vec<usize>;
}

impl GraphGetters for Graph {
    /// Get a node of an index from graph. Note that the returned node is cloned from the original.
    fn get_node(&self, index: usize) -> Node {
        self.nodes[index].as_ref().unwrap().borrow().clone()
    }

    fn get_root_nodes_indices(&self) -> &[usize] {
        &self.roots
    }
    fn get_date_nodes_indices(&self) -> Vec<usize> {
        let x: Vec<_> = self.dates.values().copied().collect();
        x
    }
    fn get_archived_node_indices(&self) -> &[usize] {
        &self.archived
    }

    fn get_node_children(&self, index: usize) -> Vec<usize> {
        self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .children
            .to_vec()
    }
}
