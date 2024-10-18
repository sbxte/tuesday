pub mod cmdline;
pub mod graph_view;
pub mod statusbar;
pub mod tabs;

use cmdline::CmdlineComponent;
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
    cmdline: Rect,
}

impl AppLayout {
    pub fn new(area: Rect) -> Self {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(2),
                Constraint::Length(1),
            ])
            .split(area);

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
        let layout = AppLayout::new(area);
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
