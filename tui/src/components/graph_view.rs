use std::process::exit;

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

const INVALID_NODE_SELECTION_MSG: &str = "Invalid selected node index found";

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
    let indent = Node::print_tree_indent(depth, value.parents.len() > 1);
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

#[derive(Debug)]
enum NodeLoc {
    Idx(usize),
    Roots,
}

pub struct GraphViewComponent {
    current_node: NodeLoc,
    filter: String,
    graph: Option<Graph>,
    rendered_nodes_len: usize,
    list_state: ListState,
    max_depth: u32,
    selected_indices: Vec<usize>,
    show_date_graphs: bool,
    path: Vec<usize>,
    selection_idx_path: Vec<usize>,
    show_archived: bool,
}

impl GraphViewComponent {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select_first();
        Self {
            current_node: NodeLoc::Roots,
            filter: String::new(),
            graph: None,
            list_state,
            max_depth: 4,
            selected_indices: Vec::new(),
            show_archived: false,
            rendered_nodes_len: 0,
            path: Vec::new(),
            selection_idx_path: Vec::new(),
            show_date_graphs: false,
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

    pub fn curr_idx(&self) -> Option<usize> {
        match self.current_node {
            NodeLoc::Idx(idx) => Some(idx),
            _ => None,
        }
    }
    pub fn step_into(&mut self) {
        if let Some(graph) = &self.graph {
            self.selection_idx_path.push(
                self.list_state
                    .selected()
                    .expect(INVALID_NODE_SELECTION_MSG),
            );
            match self.current_node {
                NodeLoc::Roots => {
                    if self.show_date_graphs {
                        let indices = graph.get_date_nodes_indices();
                        let node_idx = indices[self
                            .list_state
                            .selected()
                            .expect(INVALID_NODE_SELECTION_MSG)];

                        self.current_node = NodeLoc::Idx(node_idx);
                        self.path.push(indices[self.list_state.selected().unwrap()]);
                    } else {
                        let indices = graph.get_root_nodes_indices();
                        let node_idx = indices[self
                            .list_state
                            .selected()
                            .expect(INVALID_NODE_SELECTION_MSG)];

                        self.current_node = NodeLoc::Idx(node_idx);
                        self.path.push(indices[self.list_state.selected().unwrap()]);
                    }
                }
                NodeLoc::Idx(idx) => {
                    if self.list_state.selected() == Some(0) {
                        return;
                    }
                    self.path.push(idx);
                    let node_idx = Self::get_node_idx(
                        graph,
                        idx,
                        self.max_depth,
                        self.list_state.selected().expect("Invalid node selection") - 1,
                        !self.show_archived,
                    );
                    self.current_node = NodeLoc::Idx(node_idx);
                }
            }

            if self.rendered_nodes_len > 1 {
                self.list_state.select(Some(1));
            } else {
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn step_out(&mut self) {
        // not on root
        if self.path.len() > 1 {
            self.list_state.select(self.selection_idx_path.pop());
            self.current_node = NodeLoc::Idx(self.path.pop().expect(INVALID_NODE_SELECTION_MSG));
        } else if self.path.len() == 1 {
            self.list_state.select(self.selection_idx_path.pop());
            self.path.pop();
            self.current_node = NodeLoc::Roots;
        }
    }

    /// Get the real node index from selection index.
    fn get_node_idx(
        graph: &Graph,
        parent: usize,
        depth: u32,
        selected_index: usize,
        show_archived: bool,
    ) -> usize {
        let children_indices = graph.get_node_children(parent);
        let mut node_traversal_count = 0;
        let mut node_idx = 0;
        graph
            .traverse_recurse(
                &children_indices,
                show_archived,
                depth,
                1,
                None,
                &mut |node, _depth| {
                    if node_traversal_count == selected_index {
                        node_idx = node.index;
                        // FIXME: do not keep looping when we've found the correct index
                    }
                    node_traversal_count += 1;
                },
            )
            .expect("Failed to get node index");
        node_idx
    }

    pub fn select_first(&mut self) {
        self.list_state.select_first()
    }

    pub fn select_last(&mut self) {
        self.list_state.select_last()
    }

    pub fn select_next(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if idx + 1 < self.rendered_nodes_len {
                self.list_state.select_next()
            } else {
                self.list_state.select_first()
            }
        }
    }

    pub fn select_previous(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if idx > 0 {
                self.list_state.select_previous()
            } else {
                self.list_state.select_last()
            }
        }
    }

    fn modify_task_status(graph: &mut Graph, node_idx: usize, curr_state: NodeState) {
        match curr_state {
            NodeState::Done => {
                // TODO: error handling?
                // TODO: this gets the node_idx converted to string, then the internal function
                // converts it back into an index. nahh.
                let _ = graph.set_state(node_idx.to_string(), NodeState::None, true);
            }
            NodeState::None => {
                let _ = graph.set_state(node_idx.to_string(), NodeState::Done, true);
            }
            NodeState::Pseudo => (),
            NodeState::Partial => {
                let _ = graph.set_state(node_idx.to_string(), NodeState::Done, true);
            }
        };
    }
    pub fn check_active(&mut self) {
        if let Some(ref mut graph) = self.graph {
            match self.current_node {
                NodeLoc::Roots => {
                    let indices = graph.get_root_nodes_indices();
                    let node_idx = indices[self.list_state.selected().unwrap()];
                    let state = graph.get_node(node_idx).state;
                    Self::modify_task_status(graph, node_idx, state);
                }
                NodeLoc::Idx(idx) => {
                    let node_idx = {
                        if self
                            .list_state
                            .selected()
                            .expect(INVALID_NODE_SELECTION_MSG)
                            == 0
                        {
                            idx
                        } else {
                            Self::get_node_idx(
                                graph,
                                idx,
                                self.max_depth,
                                self.list_state.selected().expect("Invalid node") - 1,
                                !self.show_archived,
                            )
                        }
                    };

                    let state = graph.get_node(node_idx).state;
                    Self::modify_task_status(graph, node_idx, state);
                }
            }
        }
    }

    // TODO: maybe use different path stack for the date graphs view
    pub fn switch_date_graph() {}
}

impl Widget for &mut GraphViewComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut list_items = Vec::<ListItem>::new();
        if let Some(graph) = &self.graph {
            match self.current_node {
                NodeLoc::Roots => {
                    self.rendered_nodes_len = 0;
                    if self.show_date_graphs {
                        let indices = graph.get_date_nodes_indices();
                        graph
                            .traverse_recurse(
                                indices.as_slice(),
                                !self.show_archived,
                                1,
                                1,
                                None,
                                &mut |node, depth| {
                                    self.rendered_nodes_len += 1;
                                    list_items.push(list_item_from_node(node.clone(), depth))
                                },
                            )
                            .expect("Failed to traverse nodes");
                    } else {
                        self.rendered_nodes_len = 0;
                        let indices = graph.get_root_nodes_indices();
                        graph
                            .traverse_recurse(
                                indices,
                                !self.show_archived,
                                1,
                                1,
                                None,
                                &mut |node, depth| {
                                    self.rendered_nodes_len += 1;
                                    list_items.push(list_item_from_node(node.clone(), depth))
                                },
                            )
                            .expect("Failed to traverse nodes");
                    }
                }

                NodeLoc::Idx(idx) => {
                    self.rendered_nodes_len = 1; // there will always be the parent
                    let indices = graph.get_node_children(idx);
                    graph.with_node(idx, &mut |node| {
                        list_items.push(list_item_from_node(node.clone(), 0))
                    });
                    graph
                        .traverse_recurse(
                            indices.as_slice(),
                            !self.show_archived,
                            self.max_depth,
                            1,
                            None,
                            &mut |node, depth| {
                                self.rendered_nodes_len += 1;
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
