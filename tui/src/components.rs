pub mod cmdline;
pub mod graph_view;
pub mod statusbar;
pub mod tabs;


use cmdline::CmdlineComponent;
use graph_view::GraphViewComponent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};
use statusbar::StatusBarComponent;
use tabs::TabComponent;

/// Render areas for the app
pub struct AppLayout {
    pub tabs_view: Rect,
    pub graph_view: Rect,
    pub status_bar: Rect,
    pub cmdline: Rect,
}
pub fn new_layout() -> Layout {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
}

impl AppLayout {
    pub fn split(layout: Layout, area: Rect) -> AppLayout {
        let rects = layout.split(area);
        Self {
            tabs_view: rects[0],
            graph_view: rects[1],
            status_bar: rects[2],
            cmdline: rects[3],
        }
    }
}

/// UI Components for the app
pub struct AppUIComponent {
    pub(crate) tabs: TabComponent,
    pub(crate) graph_view: GraphViewComponent,
    pub(crate) status_bar: StatusBarComponent,
    pub(crate) cmdline: CmdlineComponent,
}

impl Default for AppUIComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl AppUIComponent {
    pub fn new() -> Self {
        Self {
            tabs: TabComponent::new(),
            graph_view: GraphViewComponent::new(),
            status_bar: StatusBarComponent::new(),
            cmdline: CmdlineComponent::new(),
        }
    }
}

impl Widget for &mut AppUIComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let layout = AppLayout::split(new_layout(), area);
        self.tabs.render(layout.tabs_view, buf);
        self.cmdline.render(layout.cmdline, buf);

        match self.tabs.curr_view() {
            tabs::TabView::Tasks => self.graph_view.render(layout.graph_view, buf),
            tabs::TabView::Calendar => (),
            tabs::TabView::DateGraph => (),
        }
        self.status_bar.render(layout.status_bar);
    }
}
