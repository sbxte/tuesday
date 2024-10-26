use std::fmt::Display;

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Tabs, Widget},
};

const HIGLIGHTED_COLOR: Style = Style::new().bg(Color::DarkGray).fg(Color::White);

use crate::events::TabDirection;

#[derive(Default)]
pub enum TabView {
    #[default]
    Tasks,
    DateGraph,
    Calendar,
}

impl Display for TabView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Tasks => "Tasks".to_owned(),
                Self::Calendar => "Calendar".to_owned(),
                Self::DateGraph => "Date Graph".to_owned(),
            }
        )
    }
}

impl TabView {
    fn idx(&self) -> usize {
        match self {
            Self::Tasks => 0,
            Self::Calendar => 1,
            Self::DateGraph => 2,
        }
    }
}

pub struct TabComponent {
    current_view: TabView,
}

impl Default for TabComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl TabComponent {
    pub fn curr_view(&self) -> &TabView {
        &self.current_view
    }

    pub fn new() -> Self {
        Self {
            current_view: TabView::Tasks,
        }
    }
    pub fn switch_view(&mut self, direction: &TabDirection) {
        match self.current_view {
            TabView::Tasks if *direction == TabDirection::Previous => {
                self.current_view = TabView::DateGraph;
            }
            TabView::Tasks if *direction == TabDirection::Next => {
                self.current_view = TabView::Calendar;
            }
            TabView::DateGraph if *direction == TabDirection::Previous => {
                self.current_view = TabView::Calendar;
            }

            TabView::DateGraph if *direction == TabDirection::Next => {
                self.current_view = TabView::Tasks;
            }

            TabView::Calendar if *direction == TabDirection::Next => {
                self.current_view = TabView::DateGraph;
            }

            TabView::Calendar if *direction == TabDirection::Previous => {
                self.current_view = TabView::Tasks;
            }
            _ => (),
        }
    }
}

impl Widget for &mut TabComponent {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let tabs = [TabView::Tasks, TabView::DateGraph, TabView::Calendar];
        Tabs::new(tabs.iter().map(|tab| Line::from(tab.to_string())))
            .highlight_style(HIGLIGHTED_COLOR)
            .select(self.current_view.idx())
            .render(area, buf);
    }
}
