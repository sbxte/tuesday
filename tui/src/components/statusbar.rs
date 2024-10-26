use ratatui::layout::Rect;

use crate::events::AppEvent;

enum TabView {
    Tasks,
    DateGraph,
    Calendar,
}

#[derive(PartialEq)]
enum RefreshInterval {
    /// When graph is updated, e.g by switching parent, changing nesting depth, or deleting a node.
    TreeUpdate,
}

pub struct StatusBarItem {
    refresh_interval: RefreshInterval,
    component: Box<dyn StatusBarItemUI>,
}

impl StatusBarItem {
    fn new(component: Box<dyn StatusBarItemUI>) -> Self {
        Self {
            refresh_interval: component.get_refresh_interval(),
            component,
        }
    }
}

trait StatusBarItemUI {
    fn init(&self);
    fn render(&self);
    fn get_refresh_interval(&self) -> RefreshInterval;
    fn update(&self);
}

pub struct StatusBarComponent {
    items: Vec<StatusBarItem>,
}

impl StatusBarComponent {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn insert_bar_item(&mut self, item: StatusBarItem) {
        self.items.push(item);
    }

    pub fn process_event(&self, event: AppEvent) -> Option<AppEvent> {
        for ui_item in &self.items {
            match ui_item.refresh_interval {
                RefreshInterval::TreeUpdate => match event {
                    // Do not do anything when these events are received
                    AppEvent::Operational(crate::events::OperationalEvent::Help) => return None,
                    AppEvent::Operational(crate::events::OperationalEvent::Quit) => return None,
                    AppEvent::Operational(crate::events::OperationalEvent::Selection(_)) => {
                        return None
                    }
                    _ => ui_item.component.update(),
                },
            }
        }
        None
    }

    pub fn render(&self, rect: Rect) {
        ()
    }
}
