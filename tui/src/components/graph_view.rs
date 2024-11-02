use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{palette::tailwind::SLATE, Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, StatefulWidget, Widget},
};
use tuecore::graph::{Graph, GraphGetters, Node, NodeState};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const PATTERN_MATCH_STYLE: Style = Style::new()
    .bg(Color::LightYellow)
    .fg(Color::Black)
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::UNDERLINED);
const PATTERN_MATCH_SELECTED_STYLE: Style = Style::new()
    .bg(Color::Yellow)
    .add_modifier(Modifier::UNDERLINED);
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
        filter: &str,
        storage: &mut Vec<NodeInfo>,
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
        filter: &str,
        storage: &mut Vec<NodeInfo>,
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
            let msg_match = node.message.clone().to_lowercase();
            let pattern_loc;
            if filter.is_empty() {
                pattern_loc = None;
            } else {
                pattern_loc = msg_match.find(&filter.to_lowercase());
            }
            let node_info = NodeInfo::new(node.index, depth, pattern_loc);
            storage.push(node_info);

            // If there's no need to show archived nodes then ignore it and its children
            if !skip_archived && node.archived {
                continue;
            }

            GraphTUI::get_nodes(
                self,
                &self.get_node_children(node.index),
                skip_archived,
                max_depth,
                depth + 1,
                start,
                filter,
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

/// Get a `Line` from a node message, with its pattern highlighted.
fn highlight_node_message(
    message: &str,
    pos: usize,
    pattern_len: usize,
    is_selected: bool,
) -> (Span<'static>, Span<'static>, Span<'static>) {
    // FIXME: let me out of this ownership hell
    // it would be useful to use slices but that'd mean that message has to be a reference as well
    // but then it would be reference of.. what..? nodes are dropped after they're converted to a
    // ListItem
    let left_msg = (&message[0..pos]).to_string();
    let left = Span::raw(left_msg);
    let mid_msg: String = (&message[pos..pos + pattern_len]).to_string();
    let mid;
    if is_selected {
        mid = Span::styled(mid_msg, PATTERN_MATCH_SELECTED_STYLE);
    } else {
        mid = Span::styled(mid_msg, PATTERN_MATCH_STYLE);
    }
    let right_msg: String = (&message[pos + pattern_len..message.len()]).to_string();
    let right = Span::raw(right_msg);
    (left, mid, right)
}

fn list_item_from_node<'a>(
    value: Node,
    depth: u32,
    filter_pos: Option<usize>,
    pattern_len: usize,
    is_selected: bool,
) -> ListItem<'a> {
    let indent = Node::print_tree_indent(depth, value.parents.len() > 1);
    let status = value.get_status();
    let statusbox_left = Span::styled("[", GRAPH_STATUSBOX_STYLE);
    let statusbox_right = Span::styled("] ", GRAPH_STATUSBOX_STYLE);

    // TODO: refactor lol what is this
    if let Some(indent) = indent {
        if let Some(pos) = filter_pos {
            let (left, mid, right) =
                highlight_node_message(&value.message, pos, pattern_len, is_selected);
            return ListItem::new(Line::from(vec![
                indent,
                statusbox_left,
                status,
                statusbox_right,
                left,
                mid,
                right,
            ]));
        } else {
            let message = Span::raw(value.message.to_owned());
            return ListItem::new(Line::from(vec![
                indent,
                statusbox_left,
                status,
                statusbox_right,
                message,
            ]));
        }
    } else {
        if let Some(pos) = filter_pos {
            let (left, mid, right) =
                highlight_node_message(&value.message, pos, pattern_len, is_selected);
            return ListItem::new(Line::from(vec![
                statusbox_left,
                status,
                statusbox_right,
                left,
                mid,
                right,
            ]));
        } else {
            let message = Span::raw(value.message.to_owned());
            return ListItem::new(Line::from(vec![
                statusbox_left,
                status,
                statusbox_right,
                message,
            ]));
        }
    }
}

#[derive(PartialEq)]
enum NodeLoc {
    Idx(usize),
    Roots,
}

pub struct GraphViewComponent {
    current_node: NodeLoc,
    graph: Option<Graph>,

    /// Nodes that should be rendered in current view. Stores the node index and its depth.
    /// Influenced by the depth and whether or not we should be rendering archived nodes.
    nodes: Vec<NodeInfo>,

    list_state: ListState,
    max_depth: u32,
    selected_indices: Vec<usize>,
    show_date_graphs: bool,
    path: Vec<usize>,
    selection_idx_path: Vec<usize>,
    show_archived: bool,
    filter: String,

    /// Vector of nodes that match current filter pattern. Consists of node indices (`list_state`'s
    /// index, not the real node index)
    filtered_nodes: Vec<usize>,
}

/// The minimum amount of information needed to be stored locally to enhance the efficiency of nodes rendering.
struct NodeInfo {
    /// Real index of node within graph.
    node_idx: usize,

    /// Depth of node relative to the parent node on current view.
    depth: u32,

    /// Where the pattern match (if node matches the set filter)
    pattern_loc: Option<usize>,
}

impl NodeInfo {
    fn new(node_idx: usize, depth: u32, pattern_loc: Option<usize>) -> Self {
        Self {
            node_idx,
            depth,
            pattern_loc,
        }
    }
}

impl Default for GraphViewComponent {
    fn default() -> Self {
        Self::new()
    }
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
            filter: String::new(),
            filtered_nodes: Vec::new(),
        }
    }

    // Refresh list of nodes to render. Always call after graph is manipulated.
    pub fn update_nodes(&mut self) {
        self.nodes.clear();
        if let Some(graph) = &self.graph {
            match self.current_node {
                NodeLoc::Roots => {
                    if self.show_date_graphs {
                        GraphTUI::get_nodes(
                            graph,
                            &graph.get_date_nodes_indices(),
                            self.show_archived,
                            self.max_depth,
                            1,
                            None,
                            &self.filter,
                            &mut self.nodes,
                        )
                    } else {
                        GraphTUI::get_nodes(
                            graph,
                            graph.get_root_nodes_indices(),
                            self.show_archived,
                            self.max_depth,
                            1,
                            None,
                            &self.filter,
                            &mut self.nodes,
                        );
                    }
                }
                NodeLoc::Idx(idx) => {
                    self.nodes.push(NodeInfo::new(idx, 0, None)); // the parent node
                    GraphTUI::get_nodes(
                        graph,
                        &graph.get_node_children(idx),
                        self.show_archived,
                        self.max_depth,
                        1,
                        None,
                        &self.filter,
                        &mut self.nodes,
                    );
                }
            }

            // TODO: maybe build the list below when we build the nodes?
            self.filtered_nodes = self
                .nodes
                .iter()
                .filter_map(|x| {
                    if x.pattern_loc.is_some() {
                        return Some(x.node_idx);
                    }
                    None
                })
                .collect();
        }
    }

    /// Get the currently selected index. This is different from the actual node index.
    fn get_selected_idx(&self) -> usize {
        self.list_state
            .selected()
            .expect(INVALID_NODE_SELECTION_MSG)
    }

    pub fn get_current_node(&self) -> Option<Node> {
        if let Some(graph) = &self.graph {
            let idx = self.get_selected_idx();
            if self.nodes.len() > idx {
                return Some(graph.get_node(self.nodes[idx].node_idx));
            }
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
        !self.selected_indices.is_empty()
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

    pub fn set_filter(&mut self, string: String) {
        self.filter = string;
        self.update_nodes();
    }

    /// Go to next node that matches filter
    // TODO: why not just store everything beforehand?
    pub fn jump_next_filter(&mut self) {
        let mut idx = self.get_selected_idx();
        while self.nodes[idx + 1].pattern_loc.is_none() {
            self.list_state.select(Some(idx));
            idx += 1;
        }
    }

    pub fn jump_prev_filter(&mut self) {
        let mut idx = self.get_selected_idx();
        while self.nodes[idx - 1].pattern_loc.is_none() {
            self.list_state.select(Some(idx));
            idx -= 1;
        }
    }

    pub fn delete_active_node(&mut self) {
        if let Some(graph) = &mut self.graph {
            let idx = self
                .list_state
                .selected()
                .expect(INVALID_NODE_SELECTION_MSG);
            let _ = graph.remove(self.nodes[idx].node_idx.to_string());
            self.update_nodes();
        }
    }

    pub fn step_into(&mut self) {
        if let Some(graph) = &mut self.graph {
            let idx = self
                .list_state
                .selected()
                .expect(INVALID_NODE_SELECTION_MSG);
            let node = graph.get_node(self.nodes[idx].node_idx);
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
        match self.path.len() {
            2.. => {
                self.path.pop();
                self.current_node =
                    NodeLoc::Idx(*self.path.last().expect(INVALID_NODE_SELECTION_MSG));
                self.list_state.select(self.selection_idx_path.pop());
            }
            1 => {
                self.list_state.select(self.selection_idx_path.pop());
                self.path.pop();
                self.current_node = NodeLoc::Roots;
            }
            _ => {}
        }
        self.update_nodes();
    }

    pub fn select_first(&mut self) {
        self.list_state.select_first()
    }

    pub fn select_last(&mut self) {
        self.update_nodes();
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
            let node = graph.get_node(self.nodes[idx].node_idx);
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

            let node_idx = self.nodes[idx].node_idx;
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

            let node = graph.get_node(self.nodes[idx].node_idx);
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
        let selected_idx = self.get_current_node();
        let active_node_idx;
        // TODO: refactor (maybe)
        if let Some(node) = &selected_idx {
            active_node_idx = node.index;
        } else {
            active_node_idx = 0;
        }
        if let Some(graph) = &self.graph {
            let list_items: Vec<ListItem> = self
                .nodes
                .iter()
                .map(|node_info| {
                    list_item_from_node(
                        graph.get_node(node_info.node_idx),
                        node_info.depth,
                        node_info.pattern_loc,
                        self.filter.len(),
                        selected_idx.is_some() && node_info.node_idx == active_node_idx,
                    )
                })
                .collect();

            let list = List::new(list_items)
                .highlight_style(SELECTED_STYLE)
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

            StatefulWidget::render(list, area, buf, &mut self.list_state);
        }
    }
}
