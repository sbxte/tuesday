use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{palette::tailwind::SLATE, Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, StatefulWidget, Widget},
};
use tuecore::graph::{Graph, GraphGetters, Node, NodeState};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const GRAPH_STATUSBOX_STYLE: Style = Style::new().fg(Color::Blue);

const INVALID_NODE_SELECTION_MSG: &str = "Invalid selected node index found";

trait GraphTUI {
    fn get_nodes(
        &self,
        indices: &[usize],
        skip_archived: bool,
        max_depth: u32,
        depth: u32,
        start: Option<usize>,
        storage: &mut Vec<(usize, u32)>,
    );
}

impl GraphTUI for Graph {
    fn get_nodes(
        &self,
        indices: &[usize],
        skip_archived: bool,
        max_depth: u32,
        depth: u32,
        start: Option<usize>,
        storage: &mut Vec<(usize, u32)>,
    ) {
        // A sentinel value of 0 means infinite depth
        if max_depth != 0 && depth > max_depth {
            return;
        }

        for i in indices {
            if let Some(start) = start {
                if *i == start {
                    panic!("Graph looped");
                }
            }

            let node = self.get_node(*i);
            storage.push((node.index, depth));

            // If there's no need to show archived nodes then ignore it and its children
            if !skip_archived && node.archived {
                continue;
            }

            self.get_nodes(
                &self.get_node_children(node.index),
                skip_archived,
                max_depth,
                depth + 1,
                start,
                storage,
            )
        }
    }
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

#[derive(PartialEq)]
enum NodeLoc {
    Idx(usize),
    // TODO: side effect may arise as this field is not tied to show_archived under the
    // GraphViewComponent struct below
    Roots,
}

pub struct GraphViewComponent {
    current_node: NodeLoc,
    graph: Option<Graph>,

    /// Nodes that should be rendered in current view. Stores the node index and its depth.
    /// Influenced by the depth and whether or not we should be rendering archived nodes.
    nodes: Vec<(usize, u32)>,

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
            nodes: Vec::new(),
            graph: None,
            list_state,
            max_depth: 1,
            selected_indices: Vec::new(),
            show_archived: false,
            path: Vec::new(),
            selection_idx_path: Vec::new(),
            show_date_graphs: false,
        }
    }

    // Refresh list of nodes to render. Always call after graph is manipulated.
    pub fn update_nodes(&mut self) {
        self.nodes.clear();
        if let Some(graph) = &self.graph {
            match self.current_node {
                NodeLoc::Roots => {
                    if self.show_date_graphs {
                        graph.get_nodes(
                            &graph.get_date_nodes_indices(),
                            self.show_archived,
                            self.max_depth,
                            1,
                            None,
                            &mut self.nodes,
                        )
                    } else {
                        graph.get_nodes(
                            graph.get_root_nodes_indices(),
                            self.show_archived,
                            self.max_depth,
                            1,
                            None,
                            &mut self.nodes,
                        );
                    }
                }
                NodeLoc::Idx(idx) => {
                    self.nodes.push((idx, 0)); // the parent node
                    graph.get_nodes(
                        &graph.get_node_children(idx),
                        self.show_archived,
                        self.max_depth,
                        1,
                        None,
                        &mut self.nodes,
                    );
                }
            }
        }
    }

    fn get_selected_idx(&self) -> usize {
        self.list_state
            .selected()
            .expect(INVALID_NODE_SELECTION_MSG)
    }

    pub fn get_current_node(&self) -> Option<Node> {
        if let Some(graph) = &self.graph {
            let idx = self.get_selected_idx();
            return Some(graph.get_node(self.nodes[idx].0));
        }
        None
    }

    pub fn load_graph(&mut self, graph: Graph) {
        self.graph = Some(graph);
        self.update_nodes();
    }

    pub fn get_graph(&self) -> &Option<Graph> {
        &self.graph
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

    pub fn set_depth(&mut self, depth: u32) {
        self.max_depth = depth;
        self.update_nodes();
    }

    pub fn delete_active_node(&mut self) {
        if let Some(graph) = &mut self.graph {
            let idx = self
                .list_state
                .selected()
                .expect(INVALID_NODE_SELECTION_MSG);
            let _ = graph.remove(self.nodes[idx].0.to_string());
            self.update_nodes();
        }
    }

    pub fn step_into(&mut self) {
        if let Some(graph) = &mut self.graph {
            let idx = self
                .list_state
                .selected()
                .expect(INVALID_NODE_SELECTION_MSG);
            let node = graph.get_node(self.nodes[idx].0);
            self.current_node = NodeLoc::Idx(node.index);
            self.path.push(node.index);
            self.selection_idx_path.push(idx);

            self.update_nodes();

            if self.nodes.len() > 1 {
                self.list_state.select(Some(1));
            } else {
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn step_out(&mut self) {
        // not on root
        if self.path.len() > 1 {
            self.path.pop();
            self.current_node = NodeLoc::Idx(*self.path.last().expect(INVALID_NODE_SELECTION_MSG));
            self.list_state.select(self.selection_idx_path.pop());
        } else if self.path.len() == 1 {
            self.list_state.select(self.selection_idx_path.pop());
            self.path.pop();
            self.current_node = NodeLoc::Roots;
        }
        self.update_nodes();
    }

    pub fn select_first(&mut self) {
        self.list_state.select_first()
    }

    pub fn select_last(&mut self) {
        self.list_state.select_last()
    }

    pub fn select_next(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if idx + 1 < self.nodes.len() {
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

    /// Toggle between root date graphs and normal root graphs
    pub fn toggle_switch_roots_view(&mut self) {
        if self.current_node == NodeLoc::Roots {
            self.show_date_graphs = !self.show_date_graphs;
            self.select_first();
            self.update_nodes();
        }
    }

    /// Switch to view of root nodes
    pub fn switch_view_to_roots(&mut self) {
        self.switch_to_root();
    }

    fn switch_to_root(&mut self) {
        self.path.clear();
        self.current_node = NodeLoc::Roots;
    }

    // Get the currently selected index. This is different from the actual node index.
    fn get_current_idx(&self) -> usize {
        self.list_state
            .selected()
            .expect(INVALID_NODE_SELECTION_MSG)
    }

    fn modify_task_status(graph: &mut Graph, node_idx: usize, curr_state: NodeState) {
        match curr_state {
            NodeState::Done => {
                // TODO: error handling?
                // TODO: this gets the node_idx converted to string, then the internal function
                // converts it back into a usize. nahh.
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

    pub fn rename_active(&mut self, new_message: &str) {
        if let Some(graph) = &mut self.graph {
            let idx = self
                .list_state
                .selected()
                .expect(INVALID_NODE_SELECTION_MSG);
            let node = graph.get_node(self.nodes[idx].0);
            let _ = graph.rename_node(node.index.to_string(), new_message.to_owned());
            self.update_nodes();
        }
    }

    pub fn add_node_to_active(&mut self, message: &str, pseudo: bool) {
        if let Some(graph) = &mut self.graph {
            let idx = self
                .list_state
                .selected()
                .expect(INVALID_NODE_SELECTION_MSG);

            let node_idx = self.nodes[idx].0;
            let _ = graph.insert_child(message.to_string(), node_idx.to_string(), pseudo);
            self.update_nodes();
        }
    }

    pub fn add_node_to_parent(&mut self, message: &str, pseudo: bool) {
        if let Some(graph) = &mut self.graph {
            match self.current_node {
                NodeLoc::Roots => graph.insert_root(message.to_string(), false),
                NodeLoc::Idx(idx) => {
                    let _ = graph.insert_child(message.to_string(), idx.to_string(), pseudo);
                }
            }
        }
        self.update_nodes();
    }

    pub fn check_active(&mut self) {
        if let Some(graph) = &mut self.graph {
            let idx = self
                .list_state
                .selected()
                .expect(INVALID_NODE_SELECTION_MSG);

            let node = graph.get_node(self.nodes[idx].0);
            Self::modify_task_status(graph, node.index, node.state);
            self.update_nodes();
        }
    }
}

impl Widget for &mut GraphViewComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if let Some(graph) = &self.graph {
            let list_items: Vec<ListItem> = self
                .nodes
                .iter()
                .map(|(idx, depth)| list_item_from_node(graph.get_node(*idx), *depth))
                .collect();

            let list = List::new(list_items)
                .highlight_style(SELECTED_STYLE)
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

            StatefulWidget::render(list, area, buf, &mut self.list_state);
        }
    }
}
