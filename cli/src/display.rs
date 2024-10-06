use anyhow::Result;
use tuecore::graph::{Graph, GraphGetters, Node};

/// CLI display methods.
pub trait CLIDisplay {
    fn display_node(node: &Node, depth: u32);

    fn print_tree_indent(depth: u32, dots: bool);

    fn list_roots(&self, max_depth: u32, show_archived: bool) -> Result<()>;

    fn list_archived(&self) -> Result<()>;

    fn list_dates(&self) -> Result<()>;

    fn list_children(&self, target: String, max_depth: u32, show_archived: bool) -> Result<()>;
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

    fn list_roots(&self, max_depth: u32, show_archived: bool) -> Result<()> {
        self.traverse_recurse(
            self.get_root_nodes_indices(),
            show_archived,
            max_depth,
            1,
            None,
            &|node, depth| Self::display_node(node, depth),
        )?;
        Ok(())
    }

    fn list_archived(&self) -> Result<()> {
        self.traverse_recurse(
            self.get_archived_node_indices(),
            true,
            1,
            1,
            None,
            &|node, depth| Self::display_node(node, depth),
        )?;
        Ok(())
    }

    fn list_dates(&self) -> Result<()> {
        let dates = self.get_date_nodes_indices();
        self.traverse_recurse(dates.as_slice(), false, 1, 1, None, &|node, depth| {
            Self::display_node(node, depth)
        })?;
        Ok(())
    }

    fn list_children(&self, target: String, max_depth: u32, show_archived: bool) -> Result<()> {
        let index = self.get_index(&target)?;

        // Display self as well
        self.with_node(index, |node| Self::display_node(&node, 0));

        self.traverse_recurse(
            self.get_node_children(index).as_slice(),
            show_archived,
            max_depth,
            1,
            Some(index),
            &|node, depth| Self::display_node(&node, depth),
        )?;
        Ok(())
    }
}
