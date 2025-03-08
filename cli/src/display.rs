use colored::Colorize;
use tuecore::graph::node::task::{TaskData, TaskState};
use tuecore::graph::node::{Node, NodeType};
use tuecore::graph::{Graph, GraphGetters};

use crate::AppResult;

pub fn aliases_title() -> String {
    format!("{}", "Aliases:".bold())
}

pub fn display_alias(idx: usize, alias: &str) -> String {
    format!("{}:{}", idx.to_string().bright_green(), alias.bright_blue())
}

fn display_task_data(task_data: &TaskData) -> String {
        match task_data.state {
            TaskState::None => return " ".to_string(),
            TaskState::Partial => return "~".to_string(),
            TaskState::Done => return "x".to_string()
        }
}

fn display_nodetype(node_type: &NodeType) -> String {
    match node_type {
        NodeType::Task(data) => display_task_data(data),
        NodeType::Date(_) => return "#".to_string(),
        NodeType::Pseudo => return "*".to_string(),
    }
}

fn display_node(node: &Node) -> String {
    let index = if let Some(ref alias) = node.metadata.alias {
        format!("({}:{})", node.metadata.index, alias)
    } else {
            format!("({})", node.metadata.index)
        }
        .bright_blue();
    let state = format!("{}{}{}", "[".bright_blue(), display_nodetype(&node.data), "]".bright_blue());
    format!("{} {} {}", state, node.title, index)
}

pub fn print_removal(idx: usize, recursive: bool) {
    if recursive {
        println!("Removed node {} and its children.", idx.to_string().bright_blue());

        } else {
        println!("Removed node {}.", idx.to_string().bright_blue());
    }
}

pub fn print_link_dates(from: usize, connect: bool) {
    let from = format!("({})", from).bright_blue();
    if connect {
        println!("{} -> {}", from, "(dates)".bright_blue());
    } else {
        println!("{} -x- {}", from, "(dates)".bright_blue());
    }
}

pub fn print_link_root(from: usize, connect: bool) {
    let from = format!("({})", from).bright_blue();
    if connect {
        println!("{} -> {}", from, "(root)".bright_blue());
    } else {
        println!("{} -x- {}", from, "(root)".bright_blue());
    }
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

/// CLI display methods.
pub trait CLIDisplay {
    fn display_node(node: &Node, depth: u32);

    fn print_tree_indent(depth: u32, dots: bool);

    fn list_roots(&self, max_depth: u32, show_archived: bool) -> AppResult<()>;

    fn list_archived(&self) -> AppResult<()>;

    fn list_dates(&self) -> AppResult<()>;

    fn list_children(&self, target: String, max_depth: u32, show_archived: bool) -> AppResult<()>;

    fn print_stats(&self, target: Option<String>) -> AppResult<()>;
}

impl CLIDisplay for Graph {
    fn display_node(node: &Node, depth: u32) {
        Graph::print_tree_indent(depth, node.metadata.parents.len() > 1);
        println!("{}", display_node(node));
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

    fn list_roots(&self, max_depth: u32, show_archived: bool) -> AppResult<()> {
        self.traverse_recurse(
            self.get_root_nodes_indices(),
            show_archived,
            max_depth,
            1,
            None,
            &mut |node, depth| Self::display_node(node, depth-1),
        )?;
        Ok(())
    }

    fn list_archived(&self) -> AppResult<()> {
        self.traverse_recurse(
            self.get_archived_node_indices(),
            true,
            1,
            1,
            None,
            &mut |node, depth| Self::display_node(node, depth-1),
        )?;
        Ok(())
    }

    fn list_dates(&self) -> AppResult<()> {
        let dates = self.get_date_nodes_indices();
        self.traverse_recurse(dates.as_slice(), false, 1, 1, None, &mut |node, depth| {
            Self::display_node(node, depth-1)
        })?;
        Ok(())
    }

    fn list_children(&self, target: String, max_depth: u32, show_archived: bool) -> AppResult<()> {
        let index = self.get_index(&target)?;

        // Display self as well
        self.with_node(index, &mut |node| Self::display_node(node, 0));

        self.traverse_recurse(
            self.get_node_children(index).as_slice(),
            show_archived,
            max_depth,
            1,
            Some(index),
            &mut |node, depth| Self::display_node(node, depth),
        )?;
        Ok(())
    }

    fn print_stats(&self, target: Option<String>) -> AppResult<()> {
        // If a specific node is specified
        if let Some(target) = target {
            let index = self.get_index(&target)?;
            let node = self.get_nodes()[index].as_ref().unwrap().borrow();
            println!("ID      : {}", index);
            println!("Message : {}", &node.title);
            println!("Parents :");
            for i in &node.metadata.parents {
                let parent = self.get_nodes()[*i].as_ref().unwrap().borrow();
                println!(
                    "({}) {} [{}]",
                    parent.metadata.index, parent.title, display_nodetype(&parent.data)
                );
            }
            println!("Children:");
            for i in &node.metadata.children {
                let child = self.get_nodes()[*i].as_ref().unwrap().borrow();
                println!(
                    "({}) {} [{}]",
                    child.metadata.index, child.title, display_nodetype(&child.data)
                );
            }
            if let Some(ref alias) = node.metadata.alias {
                println!("Alias   : {}", alias);
            }
            println!("Archived: {}", node.metadata.archived);
            println!("Status  : [{}]", display_nodetype(&node.data));

        // Else, list out stats for the whole graph
        } else {
            println!(
                "Nodes   : {} (Empty: {})",
                self.get_nodes().len(),
                self.get_nodes()
                    .iter()
                    .fold(0, |acc, x| if x.is_none() { acc + 1 } else { acc })
            );
            println!(
                "Edges   : {}",
                self.get_nodes()
                    .iter()
                    .fold(0, |acc, x| if let Some(x) = x {
                        acc + x.borrow().metadata.parents.len()
                    } else {
                        acc
                    })
                    + self.get_roots().len()
            );
            println!("Roots   : {}", self.get_roots().len());
            println!("Dates   : {}", self.get_dates().len());
            println!("Aliases : {}", self.get_aliases().len());
            println!("Archived: {}", self.get_archived().len());
        }
        Ok(())
    }
}
