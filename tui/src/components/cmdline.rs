use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

use crate::events::{AppEvent, AskPromptType};

const PROMPT_STYLE: Style = Style::new().fg(ratatui::style::Color::Yellow);

pub struct CmdlineComponent {
    prompt: String,
    input_string: String,
    input_pos: usize,
    shown: bool,
}

impl Default for CmdlineComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl CmdlineComponent {
    pub fn new() -> Self {
        Self {
            input_string: String::new(),
            prompt: String::new(),
            input_pos: 0,
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
                if code == &KeyCode::Char('y') || code == &KeyCode::Char('Y') {
                    // TODO: we only received a reference, so we're copying the event here. can we
                    // do better?
                    return Some(AppEvent::Operational(*ev));
                }
                Some(AppEvent::Internal(crate::events::InternalEvent::StopPrompt))
            }
            AskPromptType::Continual(ev) => {
                if code == &KeyCode::Esc || code == &KeyCode::Enter {
                    return Some(AppEvent::Internal(crate::events::InternalEvent::StopPrompt));
                }
                self.operate_string(code);
                Some(AppEvent::Operational(*ev))
            }
            AskPromptType::Input(ev) => {
                if code == &KeyCode::Enter {
                    return Some(AppEvent::Operational(*ev));
                } else if code == &KeyCode::Esc {
                    return Some(AppEvent::Internal(crate::events::InternalEvent::StopPrompt));
                }
                self.operate_string(code);
                None
            }
        }
    }

    pub fn get_curr_input(&self) -> &str {
        &self.input_string
    }

    pub fn set_curr_input(&mut self, input: &str) {
        self.input_string = input.to_string();
        self.input_pos = self.input_string.len();
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_owned()
    }

    pub fn show_prompt(&mut self) {
        self.shown = true;
    }
    pub fn hide_prompt(&mut self) {
        self.input_string.clear();
        self.input_pos = 0;
        self.shown = false;
    }

    pub fn get_cursor_pos(&self, area: Rect) -> (u16, u16) {
        (
            area.x + (self.prompt.len() as u16 + self.input_string.len() as u16)
                - (self.input_string.len() as u16 - self.input_pos as u16),
            area.y + 1,
        )
    }
    /// Operate on string based on key code.
    fn operate_string(&mut self, code: &KeyCode) {
        match code {
            KeyCode::Right => {
                if self.input_pos < self.input_string.len() {
                    self.input_pos += 1;
                }
            }
            KeyCode::Left => {
                if self.input_pos > 0 {
                    self.input_pos -= 1;
                }
            }
            KeyCode::Char(c) => {
                self.input_string.insert(self.input_pos, *c);
                self.input_pos += 1;
            }
            KeyCode::Backspace => {
                if !self.input_string.is_empty() && self.input_pos > 0 {
                    self.input_string.remove(self.input_pos - 1);
                    self.input_pos -= 1;
                }
            }
            KeyCode::Delete => {
                if !self.input_string.is_empty() && self.input_pos < self.input_string.len() {
                    self.input_string.remove(self.input_pos);
                }
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
                Span::from(self.prompt.clone()).style(PROMPT_STYLE),
                Span::from(self.input_string.clone()),
            ])
            .render(area, buf);
        }
    }
}
