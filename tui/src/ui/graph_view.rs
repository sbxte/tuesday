use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{palette::tailwind::SLATE, Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, StatefulWidget, Widget},
};
use tuecore::graph::{Graph, GraphGetters, Node, NodeState, NodeType};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const GRAPH_STATUSBOX_STYLE: Style = Style::new().fg(Color::Blue);

enum TabView {
    Tasks,
    DateGraph,
    Calendar,
}

#[derive(PartialEq)]
enum NodeViewPosition {
    Root,
    Node(usize),
}

trait NodeTUIDisplay {
    fn print_tree_indent(depth: u32, multi_parents: bool) -> Option<Span<'static>>;
    fn get_status(&self) -> Span<'static>;
}
impl NodeTUIDisplay for Node {
    fn print_tree_indent(depth: u32, multi_parents: bool) -> Option<Span<'static>> {
        if depth == 0 {
            return None;
        }

        let mut content =
            String::from(" |  ").repeat((depth - 1).try_into().expect("Invalid node depth"));
        if multi_parents {
            content.push_str(" +..");
        } else {
            content.push_str(" +--");
        }
        Some(Span::raw(content))
    }

    fn get_status(&self) -> Span<'static> {
        match self.state {
            NodeState::None => Span::raw(" "),
            NodeState::Partial => Span::raw("~"),
            NodeState::Done => Span::raw("x"),
            NodeState::Pseudo => Span::raw("+"),
        }
    }
}

fn list_item_from_node(value: Node, depth: u32) -> ListItem<'static> {
    let indent = Node::print_tree_indent(depth, value.children.len() > 1);
    let status = value.get_status();
    let statusbox_left = Span::styled("[", GRAPH_STATUSBOX_STYLE);
    let statusbox_right = Span::styled("] ", GRAPH_STATUSBOX_STYLE);
    let message = Span::raw(value.message.to_owned());
    if let Some(indent) = indent {
        return ListItem::new(Line::from(vec![
            indent,
            statusbox_left,
            status,
            statusbox_right,
            message,
        ]));
    } else {
        return ListItem::new(Line::from(vec![
            statusbox_left,
            status,
            statusbox_right,
            message,
        ]));
    }
}

enum NodeLoc {
    Idx(usize),
    Root,
    Dates,
}

pub struct GraphViewComponent {
    current_node: NodeLoc,
    filter: String,
    graph: Option<Graph>,
    list_state: ListState,
    max_depth: u32,
    selected_indices: Vec<usize>,
    show_archived: bool,
}

impl GraphViewComponent {
    pub fn new() -> Self {
        Self {
            current_node: NodeLoc::Root,
            filter: String::new(),
            graph: None,
            list_state: ListState::default(),
            max_depth: 1,
            selected_indices: Vec::new(),
            show_archived: false,
        }
    }

    pub fn load_graph(&mut self, graph: Graph) {
        self.graph = Some(graph);
    }

    pub fn graph_is_loaded(&self) -> bool {
        self.graph.is_some()
    }

    pub fn graph_multiple_selected(&self) -> bool {
        self.selected_indices.len() > 0
    }
}

impl Widget for &mut GraphViewComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if let Some(graph) = &self.graph {
            let mut list_items = Vec::<ListItem>::new();
            match self.current_node {
                NodeLoc::Root => {
                    let indices = graph.get_root_nodes_indices();
                    graph
                        .traverse_recurse(
                            indices,
                            !self.show_archived,
                            self.max_depth,
                            1,
                            None,
                            &mut |node, depth| {
                                list_items.push(list_item_from_node(node.clone(), depth))
                            },
                        )
                        .expect("Failed to traverse nodes");
                }
                NodeLoc::Idx(idx) => {
                    let indices = graph.get_node_children(idx);
                    graph
                        .traverse_recurse(
                            indices.as_slice(),
                            !self.show_archived,
                            self.max_depth,
                            1,
                            None,
                            &mut |node, depth| {
                                list_items.push(list_item_from_node(node.clone(), depth))
                            },
                        )
                        .expect("Failed to traverse nodes");
                }
                NodeLoc::Dates => {
                    let indices = graph.get_date_nodes_indices();
                    graph
                        .traverse_recurse(
                            indices.as_slice(),
                            !self.show_archived,
                            self.max_depth,
                            1,
                            None,
                            &mut |node, depth| {
                                list_items.push(list_item_from_node(node.clone(), depth))
                            },
                        )
                        .expect("Failed to traverse nodes");
                }
            };

            let list = List::new(list_items)
                .highlight_style(SELECTED_STYLE)
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

            StatefulWidget::render(list, area, buf, &mut self.list_state);
        }
    }
}
