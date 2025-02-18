use tuecore::graph::node::Node;
use tuecore::graph::{Graph, GraphGetters};

use crate::AppResult;

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
        Graph::print_tree_indent(depth, node.parents.len() > 1);
        println!("{}", node);
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
            &mut |node, depth| Self::display_node(node, depth),
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
            &mut |node, depth| Self::display_node(node, depth),
        )?;
        Ok(())
    }

    fn list_dates(&self) -> AppResult<()> {
        let dates = self.get_date_nodes_indices();
        self.traverse_recurse(dates.as_slice(), false, 1, 1, None, &mut |node, depth| {
            Self::display_node(node, depth)
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
            println!("Message : {}", &node.message);
            println!("Parents :");
            for i in &node.parents {
                let parent = self.get_nodes()[*i].as_ref().unwrap().borrow();
                println!("({}) {} [{}]", parent.index, parent.message, parent.state);
            }
            println!("Children:");
            for i in &node.children {
                let child = self.get_nodes()[*i].as_ref().unwrap().borrow();
                println!("({}) {} [{}]", child.index, child.message, child.state);
            }
            if let Some(ref alias) = node.alias {
                println!("Alias   : {}", alias);
            }
            println!("Archived: {}", node.archived);
            println!("Status  : [{}]", node.state);

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
                        acc + x.borrow().parents.len()
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
