pub mod errors;
pub mod node;

use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use errors::ErrorType;
use node::{date::DateData, date::HashMapFormatter, task, Node, NodeType};

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
    pub fn get_nodes(&self) -> &[Option<RefCell<Node>>] {
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

    /// Inserts a node into the graph and sets it as a root node.
    ///
    /// # Arguments
    /// - message: string containing the node message.
    /// - pseudo: whether the node is a pseudonode or not.
    ///
    /// # Returns
    /// A usize containing the index of the newly added node.
    pub fn insert_root(&mut self, message: String, pseudo: bool) -> usize {
        let idx = self.nodes.len();
        let mut node = Node::new(message, idx, Default::default());
        if pseudo {
            node.data = NodeType::Pseudo;
        }
        self.nodes.push(Some(RefCell::new(node)));
        self.roots.push(idx);
        idx
    }

    /// Inserts a date node into the graph.
    ///
    /// # Arguments
    /// - message: string containing the node message.
    /// - date: the date data of the node.
    ///
    /// # Returns
    /// A usize containing the index of the newly added node.
    pub fn insert_date(&mut self, message: String, date: NaiveDate) -> usize {
        let idx = self.nodes.len();
        let date_data = DateData { date };
        let node = Node::new(message, idx, NodeType::Date(date_data.clone()));
        self.nodes.push(Some(RefCell::new(node)));
        self.dates.insert(date_data.format_for_hashmap(), idx);
        idx
    }

    /// Inserts a node into the graph and sets it as a child of a parent node without updating the
    /// states of its parent. The parent node is represented using its node index.
    ///
    /// # Arguments
    /// - message: string containing the node message.
    /// - parent: parent index of the node.
    /// - pseudo: whether node is a pseudonode or not.
    ///
    /// # Returns
    /// A usize containing the index of the newly added node.
    pub fn insert_child_unchecked(
        &mut self,
        message: String,
        parent: usize,
        pseudo: bool,
    ) -> usize {
        let idx = self.nodes.len();
        let mut node = Node::new(message, idx, Default::default());
        if pseudo {
            node.data = NodeType::Pseudo;
        }
        self.nodes.push(Some(RefCell::new(node)));
        self.link_unchecked(parent, idx);
        idx
    }

    /// Inserts a node into the graph and sets it as a child of a parent node and updates the
    /// states of its parent. The parent node is represented using its node index.
    ///
    /// # Arguments
    /// - message: string containing the node message.
    /// - parent: parent index of the node.
    /// - pseudo: whether node is a pseudonode or not.
    ///
    /// # Returns
    /// A usize containing the index of the newly added node.
    pub fn insert_child(
        &mut self,
        message: String,
        parent: usize,
        pseudo: bool,
    ) -> GraphResult<usize> {
        let idx = self.insert_child_unchecked(message, parent, pseudo);
        if !pseudo {
            self.update_state_recurse_parents(&[parent] as *const _, 1)?;
        }
        Ok(idx)
    }

    /// Removes a node by `index`
    pub fn remove(&mut self, index: usize) -> GraphResult<()> {
        // Remove node if it was root
        self.roots.retain(|i| *i != index);

        // Unset alias
        let alias = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .alias
            .is_some();
        if alias {
            self.unset_alias(index)?;
        }

        // Delete from date hashmap first if node is a date root node
        if let NodeType::Date(data) = &self.nodes[index].as_ref().unwrap().borrow().data {
            self.dates.remove(&data.date.hashmap_format());
        }

        // Unlink node from parents and children
        let parents_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .as_ptr();
        let parents_len = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .len();
        for i in 0..parents_len {
            let parent = unsafe { *parents_ptr.add(i) };
            self.nodes[parent]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .metadata
                .children
                .retain(|i| *i != index);
        }
        self.update_state_recurse_parents(parents_ptr, parents_len)?;

        let children_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .children
            .as_ptr();
        let children_len = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .children
            .len();
        for i in 0..children_len {
            let child = unsafe { *children_ptr.add(i) };
            self.nodes[child]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .metadata
                .parents
                .retain(|i| *i != index);
            if self.nodes[child]
                .as_ref()
                .unwrap()
                .borrow()
                .metadata
                .parents
                .is_empty()
            {
                // Since they're now parentless, make them root.
                // This is only applicable to non-date nodes.
                // Delete from date hashmap first if node is a date root node
                if !&self.nodes[child].as_ref().unwrap().borrow().data.is_date() {
                    self.roots.push(child);
                }
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
            .metadata
            .parents
            .as_ptr();
        let parents_len = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .len();
        for i in 0..parents_len {
            let parent = unsafe { *parents_ptr.add(i) };
            self.nodes[parent]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .metadata
                .children
                .retain(|i| *i != index);
        }
        self.update_state_recurse_parents(parents_ptr, parents_len)?;
        let children_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .children
            .as_ptr();
        let children_len = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .children
            .len();
        for i in 0..children_len {
            let child = unsafe { *children_ptr.add(i) };
            self.nodes[child]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .metadata
                .parents
                .retain(|i| *i != index);
            self._remove_children_recursive(child)?;
        }

        let alias = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .alias
            .is_some();
        if alias {
            self.unset_alias(index)?;
        }

        // Delete from date hashmap first if node is a date root node
        if let NodeType::Date(data) = &self.nodes[index].as_ref().unwrap().borrow().data {
            self.dates.remove(&data.date.hashmap_format());
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
            .metadata
            .children
            .push(to);
        self.nodes[to]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .metadata
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
        let parents_ptr = self.nodes[to]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .as_ptr();
        let parents_len = self.nodes[to]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .len();
        self.update_state_recurse_parents(parents_ptr, parents_len)?;

        Ok(())
    }

    /// Unlinks two nodes on the graph without updating parent states
    fn unlink_unchecked(&mut self, from: usize, to: usize) {
        self.nodes[from]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .metadata
            .children
            .retain(|i| *i != to);
        self.nodes[to]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .metadata
            .parents
            .retain(|i| *i != from);
        // Add node to list of roots if it does not have a parent
        if self.nodes[to]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .is_empty()
        {
            // This is only applicable to non-date nodes.
            if let NodeType::Task(_) = self.nodes[to].as_ref().unwrap().borrow().data {
                self.roots.push(to);
            } else if let NodeType::Pseudo = self.nodes[to].as_ref().unwrap().borrow().data {
                self.roots.push(to);
            }
        }
    }

    /// Unlinks two nodes on the graph
    /// And updates parent states
    pub fn unlink(&mut self, from: usize, to: usize) -> GraphResult<()> {
        let parents_ptr = self.nodes[to]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .as_ptr();
        let parents_len = self.nodes[to]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .len();
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
            .metadata
            .parents
            .as_ptr();

        let parents_len = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .len();

        self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .metadata
            .parents
            .iter()
            .for_each(|i| {
                self.nodes[*i]
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .metadata
                    .children
                    .retain(|x| *x != index);
            });

        self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .metadata
            .parents
            .clear();

        self.update_state_recurse_parents(parents_ptr, parents_len)?;

        Ok(())
    }

    /// Sets task node state and optionally propogates changes to children and parents
    pub fn set_task_state(
        &mut self,
        index: usize,
        state: task::TaskState,
        propogate: bool,
    ) -> GraphResult<()> {
        match self.nodes[index].as_ref().unwrap().borrow_mut().data {
            NodeType::Task(ref mut d) => d.state = state,
            _ => return Err(ErrorType::NotTaskNode(index)),
        };

        if !propogate {
            return Ok(());
        }

        let children_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .children
            .as_ptr();
        let children_len = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .children
            .len();
        self.set_task_state_recurse(children_ptr, children_len, state)?;
        let parents_ptr = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .as_ptr();
        let parents_len = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow()
            .metadata
            .parents
            .len();
        self.update_state_recurse_parents(parents_ptr, parents_len)?;
        Ok(())
    }

    // Absolute
    fn set_task_state_recurse(
        &mut self,
        indices: *const usize,
        len: usize,
        state: task::TaskState,
    ) -> GraphResult<()> {
        for i in 0..len {
            let i = unsafe { *indices.add(i) };

            if self.nodes[i].as_ref().unwrap().borrow().data == NodeType::Pseudo {
                continue;
            }

            match self.nodes[i].as_ref().unwrap().borrow_mut().data {
                NodeType::Task(ref mut d) => d.state = state,
                _ => return Err(ErrorType::NotTaskNode(i)),
            };

            let children_ptr = self.nodes[i]
                .as_ref()
                .unwrap()
                .borrow()
                .metadata
                .children
                .as_ptr();
            let children_len = self.nodes[i]
                .as_ref()
                .unwrap()
                .borrow()
                .metadata
                .children
                .len();
            self.set_task_state_recurse(children_ptr, children_len, state)?;

            let parents_ptr = self.nodes[i]
                .as_ref()
                .unwrap()
                .borrow()
                .metadata
                .parents
                .as_ptr();
            let parents_len = self.nodes[i]
                .as_ref()
                .unwrap()
                .borrow()
                .metadata
                .parents
                .len();
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
        use task::TaskState;

        for i in 0..len {
            let i = unsafe { *indices.add(i) };
            let mut count = 0;
            let mut pseudo = 0;
            let mut partial = false;
            for child in self.nodes[i]
                .as_ref()
                .unwrap()
                .borrow()
                .metadata
                .children
                .iter()
            {
                let node = self.nodes[*child].as_ref().unwrap().borrow();
                match &node.data {
                    NodeType::Pseudo => {
                        pseudo += 1;
                    }
                    NodeType::Task(data) => match data.state {
                        TaskState::None => continue,
                        TaskState::Partial => {
                            partial = true;
                        }
                        TaskState::Done => {
                            partial = true;
                            count += 1;
                        }
                    },
                    _ => {} // Other node types should not count towards completion
                }
            }

            let mut current = self.nodes[i].as_ref().unwrap().borrow_mut();
            let completed = count > 0 && count == current.metadata.children.len() - pseudo;

            if let Some(task) = current.data.as_task_mut() {
                // Every child task is completed
                task.state = if completed {
                    TaskState::Done
                // At least one child task is completed or partially completed
                } else if partial {
                    TaskState::Partial
                } else {
                    TaskState::None
                };
            };

            if current.data.is_pseudo() {
                // No need to recurse for pseudo nodes as they do not affect parent status
                continue;
            }

            let parents_ptr = current.metadata.parents.as_ptr();
            let parents_len = current.metadata.parents.len();

            // Drop now or else the binding gets dropped at the end of the scope.
            // And as we need to mutably borrow self for updating parent states,
            // this causes borrowck to cry in agony
            std::mem::drop(current);

            self.update_state_recurse_parents(parents_ptr, parents_len)?;
        }
        Ok(())
    }

    pub fn set_archived(&mut self, index: usize, archived: bool) -> GraphResult<()> {
        let status = &mut self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .metadata
            .archived;

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
        self.nodes[index].as_ref().unwrap().borrow_mut().title = message;
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
            if rnode.metadata.alias.is_some() {
                self.aliases.insert(
                    rnode.metadata.alias.as_ref().unwrap().clone(),
                    rnode.metadata.index,
                );
            }
            if let NodeType::Date(data) = &rnode.data {
                self.dates
                    .insert(data.format_for_hashmap(), rnode.metadata.index);
            }

            if rnode.metadata.archived {
                self.archived.push(rnode.metadata.index);
            }

            // Remove invalid edges
            drop(rnode);
            let mut mnode = node.borrow_mut();
            mnode.metadata.parents.retain(|i| self.nodes[*i].is_some());
            mnode.metadata.children.retain(|i| self.nodes[*i].is_some());
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

            let parents = rnode.metadata.parents.len();
            if parents > 0 {
                continue;
            }
            let index = rnode.metadata.index;
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

    pub fn _traverse_recurse(
        &self,
        indices: &[usize],
        skip_archived: bool,
        max_depth: u32,
        depth: u32,
        start: Option<usize>,
        child_of_last: bool,
        skipped_depths: &mut Vec<u32>,
        last_depth: &mut u32,
        f: &mut impl FnMut(&Node, u32, bool, &[u32]),
    ) -> GraphResult<()> {
        // A sentinel value of 0 means infinite depth
        if max_depth != 0 && depth > max_depth {
            return Ok(());
        }

        // if we do not filter early, some arm renderings might look off.
        // for example, an entry is not actually the last entry, but it is rendered as last because
        // the actual last entry is archived. this will make the arm look wrong (not using the last
        // arm icon).
        let indices: Vec<usize> = indices
            .iter()
            .filter(|i| {
                !self.nodes[**i].as_ref().unwrap().borrow().metadata.archived || !skip_archived
            })
            .copied()
            .collect();

        for (i, idx) in indices.iter().enumerate() {
            if let Some(start) = start {
                if *idx == start {
                    return Err(ErrorType::GraphLooped(start, *idx));
                }
            }

            let last = i == indices.len() - 1;

            let child_of_last = if last { true } else { child_of_last };

            if let Some(node) = &self.nodes[*idx] {
                f(&node.borrow(), depth, last, skipped_depths);
            }

            if last {
                skipped_depths.push(depth - 1);
            }

            *last_depth = depth + 1;

            self._traverse_recurse(
                self.nodes[*idx]
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .metadata
                    .children
                    .as_slice(),
                false,
                max_depth,
                depth + 1,
                start,
                child_of_last,
                skipped_depths,
                last_depth,
                f,
            )?;
        }

        if depth < *last_depth {
            skipped_depths.pop();
        }

        Ok(())
    }

    // TODO: document this better
    /// Traverse nodes recusively. Calls a closure on each node traversal that takes a reference to
    /// the current node (`&Node`), its depth (`usize`), whether it's the last entry or not of
    /// the parent (`bool`), and arms of which depths to ignore when rendering (`&[u32]).
    pub fn traverse_recurse(
        &self,
        indices: &[usize],
        skip_archived: bool,
        max_depth: u32,
        f: &mut impl FnMut(&Node, u32, bool, &[u32]),
    ) -> GraphResult<()> {
        self._traverse_recurse(
            indices,
            skip_archived,
            max_depth,
            1,
            None,
            false,
            &mut Vec::new(),
            &mut 0,
            f,
        )
    }

    // TODO: returning an Option may make more sense?
    /// Returns the node index based on a identifier string.
    /// The identifier string may be an alias, or an index. For dates node accessing, use the
    /// `get_date_index` method.
    pub fn get_index(&self, id: &str) -> GraphResult<usize> {
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

    /// Retrieves date in [Utc](chrono::Utc) timezone
    pub fn get_date_index(&self, date: &NaiveDate) -> GraphResult<usize> {
        let key = date.hashmap_format();
        self.dates
            .get(&key)
            .ok_or(ErrorType::DateNodeIndexRetrievalError(key))
            .copied()
    }

    /// Sets an alias for node at `index`
    pub fn set_alias(&mut self, index: usize, alias: String) -> GraphResult<()> {
        self.aliases.insert(alias.clone(), index);
        self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .metadata
            .alias = Some(alias);
        Ok(())
    }

    /// Unsets a node at `index`'s alias
    pub fn unset_alias(&mut self, index: usize) -> GraphResult<()> {
        let alias = self.nodes[index]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .metadata
            .alias
            .take()
            .unwrap();
        self.aliases.remove(alias.as_str());
        Ok(())
    }

    /// Reorders node.
    ///
    /// # Arguments:
    /// - `node_idx`: node to rearrange
    /// - `parent_idx`: which node's parent to rearrange
    pub fn reorder_node_delta(
        &mut self,
        node_idx: usize,
        parent_idx: usize,
        delta_target_location: i32,
    ) -> GraphResult<()> {
        if delta_target_location == 0 {
            return Ok(());
        }
        let parents_vec = &mut self.nodes[parent_idx]
            .as_ref()
            .unwrap()
            .borrow_mut()
            .metadata
            .children;
        if let Some(pos) = parents_vec.iter().position(|&x| x == node_idx) {
            let pos_fix;
            // FIXME: uhm..

            if delta_target_location < 0 && pos as i32 >= delta_target_location {
                if pos as i32 + delta_target_location < 0 {
                    return Err(ErrorType::IndexOutOfRange(format!(
                        "Index is out of range from parents when reordering. Max move count: {pos}"
                    )));
                }
                pos_fix = pos - delta_target_location.unsigned_abs() as usize;
            } else {
                if pos + delta_target_location as usize > parents_vec.len() - 1 {
                    return Err(ErrorType::IndexOutOfRange(format!(
                        "Index is out of range from parents when reordering. Max move count: {}",
                        parents_vec.len() - 1 - pos
                    )));
                }
                pos_fix = pos + delta_target_location as usize;
            }
            parents_vec.remove(pos);
            parents_vec.insert(pos_fix, node_idx);
        } else {
            return Err(ErrorType::MalformedIndex(format!(
                "Index {node_idx} not found in {parent_idx} when reordering"
            )));
        }
        Ok(())
    }
}

/// Getters for external crates to obtain indices from private fields under `Graph`.
pub trait GraphGetters {
    fn get_node(&self, index: usize) -> Node;
    fn get_node_checked(&self, index: usize) -> Option<Node>;
    fn node_at_exists(&self, index: usize) -> bool;
    fn get_node_mut(&self, index: usize) -> RefMut<'_, Node>;
    fn get_root_nodes_indices(&self) -> &[usize];
    fn get_archived_node_indices(&self) -> &[usize];

    // TODO: both these uses vectors, is it worth the performance cost?
    fn get_date_nodes_indices(&self) -> Vec<usize>;
    fn get_node_children(&self, index: usize) -> Vec<usize>;
}

impl GraphGetters for Graph {
    // TODO: is panicking too much?
    /// Get a node of an index from graph. Note that the returned node is cloned from the original.
    /// *Warning: panics if index is invalid.*
    ///
    /// # Arguments
    /// - `index`: index of node
    ///
    /// # Returns
    /// A `Node`.
    fn get_node(&self, index: usize) -> Node {
        self.nodes[index].as_ref().unwrap().borrow().clone()
    }

    /// Get a node of an index from graph. Note that the returned node is cloned from the original.
    ///
    /// # Arguments
    /// - `index`: index of node
    ///
    /// # Returns
    /// An `Option` containing `Node` when node is found.
    fn get_node_checked(&self, index: usize) -> Option<Node> {
        if let Some(node) = self.nodes.get(index) {
            if let Some(node) = node {
                return Some(node.borrow().clone());
            }
        }
        None
    }

    fn node_at_exists(&self, index: usize) -> bool {
        self.nodes.get(index).is_some()
    }

    /// Get a node of an index from graph.
    fn get_node_mut(&self, index: usize) -> RefMut<'_, Node> {
        self.nodes[index].as_ref().unwrap().borrow_mut()
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
            .metadata
            .children
            .to_vec()
    }
}
