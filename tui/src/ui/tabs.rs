use ratatui::layout::Rect;

use crate::events::TabDirection;

enum TabView {
    Tasks,
    DateGraph,
    Calendar,
}

pub struct TabComponent {
    current_view: TabView,
}

impl TabComponent {
    pub fn new() -> Self {
        Self {
            current_view: TabView::Tasks,
        }
    }
    pub fn switch_view(&mut self, direction: TabDirection) {
        match self.current_view {
            TabView::Tasks if direction == TabDirection::Next => {
                self.current_view = TabView::DateGraph;
            }
            TabView::Tasks if direction == TabDirection::Previous => {
                self.current_view = TabView::Calendar;
            }
            TabView::DateGraph if direction == TabDirection::Next => {
                self.current_view = TabView::Calendar;
            }

            TabView::DateGraph if direction == TabDirection::Previous => {
                self.current_view = TabView::Tasks;
            }

            TabView::Calendar if direction == TabDirection::Previous => {
                self.current_view = TabView::DateGraph;
            }

            TabView::Calendar if direction == TabDirection::Previous => {
                self.current_view = TabView::Tasks;
            }
            _ => (),
        }
    }

    pub fn render(&self, rect: Rect) {
        ()
    }
}
