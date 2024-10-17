use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct CmdlineComponent {
    hint_text: String,
    shown: bool,
}

impl CmdlineComponent {
    pub fn new() -> Self {
        Self {
            hint_text: String::new(),
            shown: false,
        }
    }

    pub fn ask_prompt(&mut self, message: &str, callback: &dyn Fn(bool) -> ()) {
        let mut value = false;
        self.hint_text = message.to_string();
        self.shown = true;
        if let Event::Key(key_event) = event::read().expect("Failed to read event") {
            match key_event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => callback(true),
                _ => callback(false),
            }
        };
        self.shown = false;
    }
}

impl Widget for &mut CmdlineComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if self.shown {
            let message = Line::from(self.hint_text.clone());
            message.render(area, buf)
        }
    }
}
