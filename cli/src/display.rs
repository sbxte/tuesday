use chrono::{Datelike, NaiveDate};
use colored::Colorize;
use tuecore::graph::node::task::{TaskData, TaskState};
use tuecore::graph::node::{Node, NodeType};
use tuecore::graph::{Graph, GraphGetters};

use crate::config::{CliConfig, DEFAULT_CONFIG};
use crate::{AppError, AppResult};

/// A struct representing 24-bit color.
#[derive(Copy, Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8
}

impl Color {
    /// Creates a new color.
    pub(crate) const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert from hex code formatted color to `Color`.
    pub(crate) fn from_str(input: &str) -> AppResult<Self> {
        if let Some(col) = str_to_color_enum(input) {
            return Ok(col.into())
        }
        if input.len() != 6 {
            return Err(crate::AppError::ParseError("Error parsing color: hex color must be 6 digits".into()));
        }

        let r = u8::from_str_radix(&input[0..2], 16).map_err(|_| AppError::ParseError("Invalid red component from color".to_string()))?;
        let g = u8::from_str_radix(&input[2..4], 16).map_err(|_| AppError::ParseError("Invalid green component from color".to_string()))?;
        let b = u8::from_str_radix(&input[4..6], 16).map_err(|_| AppError::ParseError("Invalid blue component from color".to_string()))?;

        Ok(Color { r, g, b })
    }

    /// Turn color into a tuple of r, g, and b values.
    fn tup(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

impl From<Color> for (u8, u8, u8) {
    fn from(value: Color) -> Self {
        (value.r, value.g, value.b)
    }

}

#[derive(Copy, Clone)]
pub enum ColorEnum {
    Red,
    Cyan,
    Blue,
    Green,
    Orange,
    Yellow,
    Purple,
    Magenta,
    Grey,
    DarkGrey,
    White
}

fn str_to_color_enum(input: &str) -> Option<ColorEnum> {
    match input {
        "red" => Some(ColorEnum::Red),
        "cyan" => Some(ColorEnum::Cyan),
        "blue" => Some(ColorEnum::Blue),
        "green" => Some(ColorEnum::Green),
        "orange" => Some(ColorEnum::Orange),
        "yellow" => Some(ColorEnum::Yellow),
        "purple" => Some(ColorEnum::Purple),
        "magenta" => Some(ColorEnum::Magenta),
        "grey" => Some(ColorEnum::Grey),
        "darkgrey" => Some(ColorEnum::DarkGrey),
        "white" => Some(ColorEnum::White),
        _ => None
    }
}

impl From<ColorEnum> for Color {
    fn from(value: ColorEnum) -> Self {
        match value {
            ColorEnum::Red => Color::new(122, 20, 20),
            ColorEnum::Cyan => Color::new(42, 112, 135),
            ColorEnum::Blue => Color::new(52, 57, 158),
            ColorEnum::Green => Color::new(60, 135, 45),
            ColorEnum::Orange => Color::new(156, 80, 42),
            ColorEnum::Yellow => Color::new(156, 132, 17),
            ColorEnum::Purple => Color::new(107, 57, 145),
            ColorEnum::Magenta => Color::new(117, 41, 113),
            ColorEnum::Grey => Color::new(117, 117, 117),
            ColorEnum::DarkGrey => Color::new(64, 64, 64),
            ColorEnum::White => Color::new(255, 255, 255),
        }
    }

}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255
        }
    }
}

pub struct Displayer<'a> {
    config: &'a CliConfig,
}

impl<'a> Displayer<'a> {
    pub fn new(config: &'a CliConfig) -> Self {
        Self {
            config,
        }
    }

    pub fn aliases_title(&self) -> String {
        format!("{}", "Aliases:".bold())
    }


    pub fn parents_title(&self) -> String {
        format!("Node has more than one parents, please specify the parent!\n{}", "List of parents:".bold())
    }

    pub fn display_id(&self, idx: usize, alias: Option<&str>) -> String {
        if let Some(alias) = alias {
            format!("{}:{}", idx.to_string().bright_green(), alias.bright_blue())
        } else {
            format!("{}", idx.to_string().bright_green())
        }
    }

    fn display_task_data(&self, task_data: &TaskData) -> String {
        match task_data.state {
            TaskState::None => return self.config.display.icons.node_none.to_string()
                .custom_color(self.config.display.icons.node_none.color.tup()).to_string(),
            TaskState::Partial => return self.config.display.icons.node_partial.to_string()
                .custom_color(self.config.display.icons.node_partial.color.tup()).to_string(),
            TaskState::Done => return self.config.display.icons.node_checked.to_string()
                .custom_color(self.config.display.icons.node_checked.color.tup()).to_string()
        }
    }

    fn display_nodetype(&self, node_type: &NodeType) -> String {
        match node_type {
            NodeType::Task(data) => self.display_task_data(data),
            NodeType::Date(_) => return self.config.display.icons.node_date.to_string()
            .custom_color(self.config.display.icons.node_date.color.tup()).to_string(),
            NodeType::Pseudo => return self.config.display.icons.node_pseudo.to_string()
            .custom_color(self.config.display.icons.node_pseudo.color.tup()).to_string(),
        }
    }

    fn fmt_node(&self, node: &Node) -> String {
        let index = if let Some(ref alias) = node.metadata.alias {
            format!("({}:{})", node.metadata.index, alias)
        } else {
                format!("({})", node.metadata.index)
            }
            .bright_blue();
        let state = self.display_nodetype(&node.data);

        let dim = node.metadata.archived;

        if let NodeType::Date(data) = &node.data {
            let title = if node.title.is_empty() {
                " ".to_string()
            } else {
                format!(" {} ", node.title.clone())
            };
            if dim {
                format!("{} {}{}{}", state, format!("[{}]", data.date.format(&self.config.display.date_fmt)).dimmed(), title.dimmed(), index)
            } else {
                format!("{} {}{}{}", state, format!("[{}]", data.date.format(&self.config.display.date_fmt)).dimmed(), title, index)
            }
        } else {
            if dim {
                format!("{} {} {}", state, node.title.dimmed(), index)
            } else {
                format!("{} {} {}", state, node.title, index)
            }
        }
    }

    pub fn display_node(&self, node: &Node, depth: u32, last: bool, skipped_depths: &[u32]) {
        self.print_tree_indent(depth, node.metadata.parents.len() > 1, last, skipped_depths);
        println!("{}", self.fmt_node(node));
    }

    pub fn print_tree_indent(&self, depth: u32, dots: bool, last: bool, skipped_depths: &[u32]) {
        if depth == 0 {
            return;
        }

        if !self.config.display.bar_indent {
            for i in 0..depth - 1 {
                if skipped_depths.iter().find(|j| **j == i).is_some() {
                    print!("    ");
                } else {
                    print!(" {}  ", self.config.display.icons.arm_bar.to_string()
                        .custom_color(self.config.display.icons.arm_bar.color.tup())
                    );
                }
            }
        } else {
            for _ in 0..(depth - 1) {
                print!(" {}  ", self.config.display.icons.arm_bar.to_string()
                    .custom_color(self.config.display.icons.arm_bar.color.tup())
                );
            }

        }

        if dots {
            if last {
                print!(" {}", self.config.display.icons.arm_multiparent_last.to_string()
                .custom_color(self.config.display.icons.arm_multiparent_last.color.tup())
                );
            } else {
                print!(" {}", self.config.display.icons.arm_multiparent.to_string()
                .custom_color(self.config.display.icons.arm_multiparent.color.tup())
                );
            }
        } else {
            if last {
                print!(" {}", self.config.display.icons.arm_last.to_string()
                .custom_color(self.config.display.icons.arm_last.color.tup())
                );
            } else {
                print!(" {}", self.config.display.icons.arm.to_string()
                .custom_color(self.config.display.icons.arm.color.tup())
                );

            }
        }
    }

    pub fn list_roots(&self, graph: &Graph, max_depth: u32, show_archived: bool) -> AppResult<()> {
        // TODO: wonky
        let indices = graph.get_root_nodes_indices();
        if max_depth == 1 {
            for i in indices {
                graph.with_node(*i, &mut |node| self.display_node(node, 0, false, &[]));
            }
        } else {
            for i in indices {
                self.list_children(graph, *i, max_depth.checked_sub(1).unwrap_or(0), show_archived)?
            }
        }
        Ok(())
    }

    pub fn list_archived(&self, graph: &Graph) -> AppResult<()> {
        // we probably don't want to recurse.
        // TODO: or does it make more sense to recurse?
        let indices = graph.get_archived_node_indices();

        for i in indices {
            graph.with_node(*i, &mut |node| self.display_node(node, 0, false, &[]));
        }
        Ok(())
    }

    pub fn list_dates(&self, graph: &Graph, skip_archived: bool) -> AppResult<()> {
        let dates: Vec<usize> = graph.get_date_nodes_indices().iter()
            .filter(|idx| !graph.get_node(**idx).metadata.archived || !skip_archived)
            .map(|x| *x).collect();
        graph.traverse_recurse(dates.as_slice(), false, 1, &mut |node, depth, last, depth_of_last| { self.display_node(node, depth-1, last,  depth_of_last) })?;
        Ok(())
    }

    pub fn list_children(&self, graph: &Graph, target: usize, max_depth: u32, show_archived: bool) -> AppResult<()> {
        // Display self as well
        graph.with_node(target, &mut |node| self.display_node(node, 0, false, &[]));

        graph.traverse_recurse(
            graph.get_node_children(target).as_slice(),
            show_archived,
            max_depth,
            &mut |node, depth, last, depth_of_last| self.display_node(node, depth, last, depth_of_last),
        )?;
        Ok(())
    }

    pub fn print_stats(&self, graph: &Graph, target: Option<usize>) -> AppResult<()> {
        // If a specific node is specified
        if let Some(target) = target {
            let node = graph.get_nodes()[target].as_ref().unwrap().borrow();
            println!("ID      : {}", target);
            println!("Message : {}", &node.title);
            println!("Parents :");
            for i in &node.metadata.parents {
                let parent = graph.get_nodes()[*i].as_ref().unwrap().borrow();
                println!(
                    "({}) {} [{}]",
                    parent.metadata.index, parent.title, self.display_nodetype(&parent.data)
                );
            }
            println!("Children:");
            for i in &node.metadata.children {
                let child = graph.get_nodes()[*i].as_ref().unwrap().borrow();
                println!(
                    "({}) {} [{}]",
                    child.metadata.index, child.title, self.display_nodetype(&child.data)
                );
            }
            if let Some(ref alias) = node.metadata.alias {
                println!("Alias   : {}", alias);
            }
            println!("Archived: {}", node.metadata.archived);
            println!("Status  : [{}]", self.display_nodetype(&node.data));

        // Else, list out stats for the whole graph
        } else {
            println!(
                "Nodes   : {} (Empty: {})",
                graph.get_nodes().len(),
                graph.get_nodes()
                    .iter()
                    .fold(0, |acc, x| if x.is_none() { acc + 1 } else { acc })
            );
            println!(
                "Edges   : {}",
                graph.get_nodes()
                    .iter()
                    .fold(0, |acc, x| if let Some(x) = x {
                        acc + x.borrow().metadata.parents.len()
                    } else {
                        acc
                    })
                    + graph.get_roots().len()
            );
            println!("Roots   : {}", graph.get_roots().len());
            println!("Dates   : {}", graph.get_dates().len());
            println!("Aliases : {}", graph.get_aliases().len());
            println!("Archived: {}", graph.get_archived().len());
        }
        Ok(())
    }

    fn print_heatmap(&self) {
        print!("\x1B[6A"); // up
        print!("\x1B[24C"); // right
        print!("finished");
        print!("\x1B[9D"); // left
        print!("\x1B[1B"); // down
        for i in 0..5 {
            print!("{}", "  ".on_custom_color(self.config.display.calendar_config.heatmap_palette[i].tup()));
        }
        print!("\x1B[11D"); // left
        print!("\x1B[1B"); // down
        print!("less    more");
        print!("\x1B[4B\r"); // down

    }

    pub fn print_calendar(&self, graph: &Graph, date: &NaiveDate) -> AppResult<()> {
        println!("Calendar: {} {}", date.format("%B").to_string().bold(), date.format("%Y").to_string().green());

        for i in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
            print!("{}", i.yellow());
            print!(" ");
        }

        let days_in_month = self.days_in_month(date.year(), date.month()) as u32;

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

                    let color = self.config.display.calendar_config.heatmap_palette[range_finished].tup();

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

        self.print_heatmap();
        Ok(())

    }


    pub fn print_removal(&self, idx: usize, recursive: bool) {
        if recursive {
            println!("Removed node {} and its children.", idx.to_string().bright_blue());

        } else {
            println!("Removed node {}.", idx.to_string().bright_blue());
        }
    }

    pub fn print_link_dates(&self, from: usize, connect: bool) {
        let from = format!("({})", from).bright_blue();
        if connect {
            println!("{} -> {}", from, "(dates)".bright_blue());
        } else {
            println!("{} -x- {}", from, "(dates)".bright_blue());
        }
    }

    pub fn print_link_root(&self, from: usize, connect: bool) {
        let from = format!("({})", from).bright_blue();
        if connect {
            println!("{} -> {}", from, "(root)".bright_blue());
        } else {
            println!("{} -x- {}", from, "(root)".bright_blue());
        }
    }

    pub fn print_link(&self, from: usize, to: usize, connect: bool) {
        let from = format!("({})", from).bright_blue();
        let to = format!("({})", to).bright_blue();
        if connect {
            println!("{} -> {}", from, to);
        } else {
            println!("{} -x- {}", from, to);
        }
    }

    pub fn days_in_month(&self, year: i32, month: u32) -> i64 {
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

    pub fn template_cfg(&self) -> &'static str {
        DEFAULT_CONFIG
    }

}
