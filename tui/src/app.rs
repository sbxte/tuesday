use crate::{
    components::{self, tabs::TabView},
    events::{
        self, ActiveNodeOperation, AppEvent, AskPromptType, InternalEvent, NavDirection,
        OperationalEvent,
    },
};
use tuecore::graph::Graph;

const STOP_CAPTURING_KEY: Option<AppEvent> = Some(AppEvent::Internal(InternalEvent::StopPrompt));

#[derive(Default)]
pub struct AppState {
    pub(crate) current_view: TabView,
    pub(crate) should_exit: bool,
    pub(crate) cmdline_focused: bool,
    pub(crate) is_capturing_key: Option<AskPromptType>,
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

    pub fn is_capturing_keys(&self) -> bool {
        self.state.is_capturing_key.is_some()
    }

    /// Process an `AppEvent`, which in turn may emit another event.
    ///
    /// There are two modes for event processing. The first is "normal" mode
    /// where events are processed as-is.
    ///
    /// The second is "capture" mode where this method would use the `Cmdline` component to process keys
    /// forwarded by event dispatcher, and act upon all operational events received back.
    pub fn process_event(&mut self, event: AppEvent) -> Option<AppEvent> {
        // Handle events during capturing mode
        if let Some(prompt_type) = &self.state.is_capturing_key {
            match event {
                AppEvent::Internal(ev) => match ev {
                    InternalEvent::ForwardKey(code) => {
                        return self.components.cmdline.process_input(&prompt_type, &code)
                    }
                    InternalEvent::StopPrompt => {
                        self.components.cmdline.hide_prompt();
                        self.state.is_capturing_key = None;
                        return None;
                    }
                    _ => (),
                },
                AppEvent::Operational(ev) => match ev {
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::Delete) => {
                        self.components.graph_view.delete_active_node();
                        // TODO: consider automatically returning this after matching the
                        // OperationalEvent
                        return STOP_CAPTURING_KEY;
                    }
                    _ => (),
                },
            }
            return None;
        }

        // Events processed during normal mode
        match event {
            AppEvent::Internal(InternalEvent::AskPrompt(prompt_type, msg)) => {
                self.components.cmdline.set_prompt(&msg);
                self.components.cmdline.show_prompt();
                self.state.is_capturing_key = Some(prompt_type);
            }

            AppEvent::Operational(ev) => match ev {
                OperationalEvent::Quit => self.state.should_exit = true,
                OperationalEvent::TabChange(direction) => {
                    self.components.tabs.switch_view(&direction)
                }

                OperationalEvent::Navigate(navigation) => match navigation {
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
                OperationalEvent::OperateActiveNode(ref operation) => match operation {
                    ActiveNodeOperation::Check => self.components.graph_view.check_active(),
                    ActiveNodeOperation::Delete => {
                        return Some(AppEvent::Internal(InternalEvent::AskPrompt(
                            AskPromptType::Confirmation(ev),
                            "Delete active node? (y/n)".to_string(),
                        )));
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
        None
    }
}
