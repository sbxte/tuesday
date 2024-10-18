use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::events::{AppEvent, AskPromptType, OperationalEvent};

pub struct CmdlineComponent {
    prompt: String,
    input_string: String,
    shown: bool,
}

impl CmdlineComponent {
    pub fn new() -> Self {
        Self {
            input_string: String::new(),
            prompt: String::new(),
            shown: false,
        }
    }

    pub fn process_input(
        &mut self,
        key_capture_type: &AskPromptType,
        code: &KeyCode,
    ) -> Option<AppEvent> {
        match key_capture_type {
            AskPromptType::Confirmation(ev) => {
                if code == &KeyCode::Char('y') {
                    return Some(AppEvent::Operational(*ev));
                }
                Some(AppEvent::Internal(crate::events::InternalEvent::StopPrompt))
            }
            AskPromptType::Continual(ev) => {
                if code == &KeyCode::Esc {
                    return Some(AppEvent::Operational(*ev));
                }
                self.operate_string(code);
                None
            }
            AskPromptType::Input(ev) => {
                if code == &KeyCode::Enter {
                    return Some(AppEvent::Operational(*ev));
                }
                self.operate_string(code);
                None
            }
        }
    }

    pub fn get_input(&self) -> &String {
        &self.input_string
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_owned()
    }

    pub fn show_prompt(&mut self) {
        self.shown = true;
    }
    pub fn hide_prompt(&mut self) {
        self.input_string.clear();
        self.shown = false;
    }
    /// Operate on string based on key code.
    fn operate_string(&mut self, code: &KeyCode) {
        match code {
            KeyCode::Char(c) => self.input_string.push(*c),
            KeyCode::Backspace => {
                self.input_string.pop();
            }
            _ => (),
        }
    }
}

impl Widget for &mut CmdlineComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if self.shown {
            Line::from(vec![
                Span::from(self.prompt.clone()),
                Span::from(" "),
                Span::from(self.input_string.clone()),
            ])
            .render(area, buf);
        }
    }
}
