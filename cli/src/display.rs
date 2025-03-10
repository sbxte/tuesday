use chrono::{Datelike, NaiveDate};
use colored::Colorize;
use tuecore::graph::node::task::{TaskData, TaskState};
use tuecore::graph::node::{Node, NodeType};
use tuecore::graph::{Graph, GraphGetters};

use crate::AppResult;

pub fn aliases_title() -> String {
    format!("{}", "Aliases:".bold())
}

pub fn parents_title() -> String {
    format!("Node has more than one parents, please specify the parent!\n{}", "List of parents:".bold())
}
pub fn display_id(idx: usize, alias: Option<&str>) -> String {
    if let Some(alias) = alias {
        format!("{}:{}", idx.to_string().bright_green(), alias.bright_blue())
    } else {
        format!("{}", idx.to_string().bright_green())
    }
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

    let dim = node.metadata.archived;

    if let NodeType::Date(data) = &node.data {
        let title = if node.title.is_empty() {
        " ".to_string()
        } else {
            format!(" {} ", node.title.clone())
        };
        if dim {
            format!("{} {}{}{}", state, format!("[{}]", data.date.format("%Y-%m-%d")).dimmed(), title.dimmed(), index)
        } else {
            format!("{} {}{}{}", state, format!("[{}]", data.date.format("%Y-%m-%d")).dimmed(), title, index)
        }
    } else {
        if dim {
            format!("{} {} {}", state, node.title.dimmed(), index)
        } else {
            format!("{} {} {}", state, node.title, index)
        }
    }
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

pub fn days_in_month(year: i32, month: u32) -> i64 {
    NaiveDate::from_ymd_opt(
        match month {
            12 => year + 1,
            _ => year

        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    ).unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days()
}

// TODO: make this configurable
const HEATMAP_PALLETE: [(u8, u8, u8); 5] = [(58, 80, 162), (120, 94, 240), (220, 38, 127), (254, 97, 0), (255, 176, 0)];

fn print_heatmap() {
    print!("\x1B[6A"); // up
    print!("\x1B[24C"); // right
    print!("finished");
    print!("\x1B[9D"); // left
    print!("\x1B[1B"); // down
    for i in 0..5 {
        print!("{}", "  ".on_custom_color(HEATMAP_PALLETE[i]));
    }
    print!("\x1B[11D"); // left
    print!("\x1B[1B"); // down
    print!("less    more");
    print!("\x1B[4B\r"); // down

}

pub fn print_calendar(graph: &Graph, date: &NaiveDate) -> AppResult<()> {
    println!("Calendar: {} {}", date.format("%B").to_string().bold(), date.format("%Y").to_string().green());

    for i in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
        print!("{}", i.yellow());
        print!(" ");
    }

    let days_in_month = days_in_month(date.year(), date.month()) as u32;

    // HACK: i don't know why datetime is so unnecessarily hard here?!??!! please fix below
    let nd = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
    let first_day = nd.format("%u").to_string().parse::<u32>().unwrap() % 7;

    print!("\n");

    let days: Vec<u32> = (1..=days_in_month+first_day).into_iter().collect();
    for i in days {
        if i > first_day {
            let curr_date = i - first_day;
            if let Ok(idx) = graph.get_date_index(&NaiveDate::from_ymd_opt(date.year(), date.month(), curr_date).unwrap()) {
                let node = graph.get_node(idx);
                let finished = node.metadata.children.iter().filter(|idx| {
                    let node = graph.get_node(**idx);
                    if let NodeType::Task(data) = node.data {
                        data.state == TaskState::Done
                    } else {
                        false
                    }
                }).count();
                let total_nodes = node.metadata.children.iter().filter(|idx| {
                    let node = graph.get_node(**idx);
                    if let NodeType::Task(_) = node.data {
                        true
                    } else {
                        false
                    }
                }).count();

                let range_finished = if total_nodes == 0 {
                    0
                } else {
                    finished * 4 / total_nodes
                };

                let color = HEATMAP_PALLETE[range_finished];

                if curr_date == date.day() {
                    print!("{} ", format!("{:02}", curr_date).bold().underline().on_custom_color(color));
                } else {
                    print!("{} ", format!("{:02}", curr_date).on_custom_color(color));

                }
            } else {
                    print!("{} ", format!("{:02}", curr_date).dimmed());
            }
        } else {
            print!("   ");
        }
        if i % 7 == 0 {
            print!("\n");
        }
    }

    print!("\n");

    print_heatmap();
    Ok(())

}

/// CLI display methods.
pub trait CLIDisplay {
    fn display_node(node: &Node, depth: u32, last: bool);

    fn print_tree_indent(depth: u32, dots: bool);

    fn list_roots(&self, max_depth: u32, show_archived: bool) -> AppResult<()>;

    fn list_archived(&self) -> AppResult<()>;

    fn list_dates(&self, skip_archived: bool) -> AppResult<()>;

    fn list_children(&self, target: usize, max_depth: u32, show_archived: bool) -> AppResult<()>;

    fn print_stats(&self, target: Option<usize>) -> AppResult<()>;
}

impl CLIDisplay for Graph {
    fn display_node(node: &Node, depth: u32, last: bool) {
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
            &mut |node, depth, last| Self::display_node(node, depth-1, last),
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
            &mut |node, depth, last| Self::display_node(node, depth-1, last),
        )?;
        Ok(())
    }

    fn list_dates(&self, skip_archived: bool) -> AppResult<()> {
        let dates: Vec<usize> = self.get_date_nodes_indices().iter()
            .filter(|idx| !self.get_node(**idx).metadata.archived || skip_archived)
            .map(|x| *x).collect();
        self.traverse_recurse(dates.as_slice(), false, 1, 1, None,
            &mut |node, depth, last| { Self::display_node(node, depth-1, last) })?;
        Ok(())
    }

    fn list_children(&self, target: usize, max_depth: u32, show_archived: bool) -> AppResult<()> {
        // Display self as well
        self.with_node(target, &mut |node| Self::display_node(node, 0, false));

        self.traverse_recurse(
            self.get_node_children(target).as_slice(),
            show_archived,
            max_depth,
            1,
            Some(target),
            &mut |node, depth, last| Self::display_node(node, depth, last),
        )?;
        Ok(())
    }

    fn print_stats(&self, target: Option<usize>) -> AppResult<()> {
        // If a specific node is specified
        if let Some(target) = target {
            let node = self.get_nodes()[target].as_ref().unwrap().borrow();
            println!("ID      : {}", target);
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
