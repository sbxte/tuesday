pub mod graph_view;
pub mod statusbar;
pub mod tabs;

use graph_view::GraphViewComponent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
    Frame,
};
use statusbar::StatusBarComponent;
use tabs::TabComponent;

/// Render areas for the app
pub struct AppLayout {
    tabs_view: Rect,
    graph_view: Rect,
    status_bar: Rect,
}

impl AppLayout {
    pub fn new(frame: Rect) -> Self {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Percentage(2),
            ])
            .split(frame);

        Self {
            tabs_view: rects[0],
            graph_view: rects[1],
            status_bar: rects[2],
        }
    }
}

/// UI Components for the app
pub struct AppUIComponent {
    pub(crate) tabs: TabComponent,
    pub(crate) graph_view: GraphViewComponent,
    pub(crate) status_bar: StatusBarComponent,
}

impl AppUIComponent {
    pub fn new() -> Self {
        Self {
            tabs: TabComponent::new(),
            graph_view: GraphViewComponent::new(),
            status_bar: StatusBarComponent::new(),
        }
    }
}

impl Widget for &mut AppUIComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let layout = AppLayout::new(area);
        self.tabs.render(layout.tabs_view);
        self.graph_view.render(layout.graph_view, buf);
        self.status_bar.render(layout.status_bar);
    }
}
