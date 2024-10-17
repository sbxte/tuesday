use crate::{
    components::{self, tabs::TabView},
    events::{self, ActiveNodeOperation, AppEvent, InternalEvent, NavDirection},
};
use tuecore::graph::Graph;

#[derive(Default)]
pub struct AppState {
    pub(crate) current_view: TabView,
    pub(crate) should_exit: bool,
    pub(crate) cmdline_focused: bool,
}

/// App state
pub struct App {
    pub(crate) components: components::AppUIComponent,
    pub(crate) state: AppState,
}

impl App {
    pub fn should_exit(&self) -> bool {
        self.state.should_exit
    }
    pub fn load_graph(&mut self, graph: Graph) {
        self.components.graph_view.load_graph(graph);
    }

    pub fn graph_is_loaded(&self) -> bool {
        self.components.graph_view.graph_is_loaded()
    }

    pub fn current_view(&self) -> &TabView {
        &self.components.tabs.curr_view()
    }

    pub fn multiple_nodes_selected(&self) -> bool {
        self.components.graph_view.graph_multiple_selected()
    }

    pub fn new() -> Self {
        App {
            components: components::AppUIComponent::new(),
            state: AppState::default(),
        }
    }

    /// Process an event, which in turn may emit another event.
    pub fn process_event(&mut self, event: &AppEvent) -> Option<AppEvent> {
        match event {
            AppEvent::Internal(event) => match event {
                InternalEvent::AskPrompt(message, f) => {
                    self.components.cmdline.ask_prompt(message, f)
                }
            },
            AppEvent::Quit => self.state.should_exit = true,
            AppEvent::TabChange(direction) => self.components.tabs.switch_view(direction),
            AppEvent::Navigate(navigation) => match navigation {
                NavDirection::Next => self.components.graph_view.select_next(),
                NavDirection::Previous => self.components.graph_view.select_previous(),
                NavDirection::StepIn => self.components.graph_view.step_into(),
                NavDirection::StepOut => self.components.graph_view.step_out(),
                NavDirection::First => self.components.graph_view.select_first(),
                NavDirection::Last => self.components.graph_view.select_last(),
                NavDirection::ToggleRootView => {
                    self.components.graph_view.toggle_switch_roots_view()
                }
                NavDirection::ToRoot => self.components.graph_view.switch_view_to_roots(),
                _ => (),
            },
            AppEvent::OperateActiveNode(operation) => match operation {
                ActiveNodeOperation::Check => self.components.graph_view.check_active(),
                _ => (),
            },
            _ => (),
        }
        None
    }
}
