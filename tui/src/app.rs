use crate::{events::AppEvent, ui};
use crossterm::event::Event;
use ratatui::{
    layout::{Direction, Layout, Rect},
    prelude::Constraint,
    widgets::Widget,
    Frame,
};
use tuecore::graph::Graph;

#[derive(Default, PartialEq)]
pub enum AppView {
    #[default]
    Tasks,
    DateGraph,
    Calendar,
}

#[derive(Default)]
struct AppState {
    pub(crate) current_view: AppView,
    pub(crate) should_exit: bool,
}

/// App state
pub struct App {
    pub(crate) components: ui::AppUIComponent,
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

    pub fn current_view(&self) -> &AppView {
        &self.state.current_view
    }

    pub fn multiple_nodes_selected(&self) -> bool {
        self.components.graph_view.graph_multiple_selected()
    }

    pub fn new() -> Self {
        App {
            components: ui::AppUIComponent::new(),
            state: AppState::default(),
        }
    }

    /// Process an event, which in turn may emit another event.
    pub fn process_event(&mut self, event: &AppEvent) -> Option<AppEvent> {
        match event {
            AppEvent::Quit => self.state.should_exit = true,
            _ => (),
        }
        None
    }
}
