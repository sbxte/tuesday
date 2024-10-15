use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::{app::App, components::tabs::TabView};

/// Possible Tab Change Directions.
#[derive(PartialEq)]
pub enum TabDirection {
    Next,     // ]
    Previous, // [
}

/// Possible Navigation Actions.
pub enum NavDirection {
    JumpTo,         // J
    LastLocation,   // -
    Next,           // j
    Previous,       // k
    StepIn,         // l
    StepOut,        // h
    First,          // g
    Last,           // G
    ToggleRootView, // ~
    ToRoot,         // `
}

/// Operations done with active node.
pub enum ActiveNodeOperation {
    CopyTo,     // P
    LinkTo,     // S
    Modify,     // M
    Delete,     // X
    MoveTo,     // V
    Rename,     // r
    UnlinkFrom, // D
    Check,      // C
}

/// Operations done with selected nodes.
pub enum SelectedNodeOperation {
    CopyTo,     // p
    Delete,     // x
    LinkTo,     // s
    MoveTo,     // v
    UnlinkFrom, // d
    Check,      // c
}

/// Node filtering operations.
pub enum ViewFilterOperation {
    Filter,         // /
    SetDepth,       // *
    ToggleArchived, // .
}

/// Node selection operations
pub enum NodeSelectionOperation {
    RangeMark,    // m
    ToggleSelect, // space
}

/// App Events
pub enum AppEvent {
    Filter(ViewFilterOperation),
    Help, // H or F1
    Navigate(NavDirection),
    OperateActiveNode(ActiveNodeOperation),
    OperateSelected(SelectedNodeOperation),
    Quit, // q
    Selection(NodeSelectionOperation),
    TabChange(TabDirection),
}

/// Process key inputs based on context and emit the appropriate event.
pub fn process_key(app: &App, key_event: KeyEvent) -> Option<AppEvent> {
    if key_event.kind == KeyEventKind::Release {
        return None;
    }

    match key_event.code {
        KeyCode::Char(']') | KeyCode::Tab => return Some(AppEvent::TabChange(TabDirection::Next)),
        KeyCode::Char('[') | KeyCode::BackTab => {
            return Some(AppEvent::TabChange(TabDirection::Previous))
        }
        KeyCode::Char('q') => return Some(AppEvent::Quit),
        KeyCode::F(1) => return Some(AppEvent::Help),
        _ => (),
    }

    // Specific to current view
    match app.current_view() {
        TabView::Tasks | TabView::DateGraph => {
            if !app.graph_is_loaded() {
                return None;
            };
            return match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => Some(AppEvent::Navigate(NavDirection::Next)),
                KeyCode::Char('k') | KeyCode::Up => {
                    Some(AppEvent::Navigate(NavDirection::Previous))
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    Some(AppEvent::Navigate(NavDirection::StepIn))
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    Some(AppEvent::Navigate(NavDirection::StepOut))
                }
                KeyCode::Char('`') => Some(AppEvent::Navigate(NavDirection::ToRoot)),
                KeyCode::Char('~') => Some(AppEvent::Navigate(NavDirection::ToggleRootView)),
                KeyCode::Char('-') => Some(AppEvent::Navigate(NavDirection::LastLocation)),
                KeyCode::Char('J') => Some(AppEvent::Navigate(NavDirection::JumpTo)),
                KeyCode::Char('g') => Some(AppEvent::Navigate(NavDirection::First)),
                KeyCode::Char('G') => Some(AppEvent::Navigate(NavDirection::Last)),

                KeyCode::Char('P') => {
                    Some(AppEvent::OperateActiveNode(ActiveNodeOperation::CopyTo))
                }
                KeyCode::Char('V') => {
                    Some(AppEvent::OperateActiveNode(ActiveNodeOperation::MoveTo))
                }
                KeyCode::Char('r') => {
                    Some(AppEvent::OperateActiveNode(ActiveNodeOperation::Rename))
                }
                KeyCode::Char('M') => {
                    Some(AppEvent::OperateActiveNode(ActiveNodeOperation::Modify))
                }
                KeyCode::Char('S') => {
                    Some(AppEvent::OperateActiveNode(ActiveNodeOperation::LinkTo))
                }
                KeyCode::Char('D') => {
                    Some(AppEvent::OperateActiveNode(ActiveNodeOperation::UnlinkFrom))
                }
                KeyCode::Char('X') => {
                    Some(AppEvent::OperateActiveNode(ActiveNodeOperation::Delete))
                }

                KeyCode::Char('C') => Some(AppEvent::OperateActiveNode(ActiveNodeOperation::Check)),

                KeyCode::Char('p') => {
                    Some(AppEvent::OperateSelected(SelectedNodeOperation::CopyTo))
                }
                KeyCode::Char('v') => {
                    Some(AppEvent::OperateSelected(SelectedNodeOperation::MoveTo))
                }
                KeyCode::Char('d') => {
                    Some(AppEvent::OperateSelected(SelectedNodeOperation::UnlinkFrom))
                }
                KeyCode::Char('x') => {
                    Some(AppEvent::OperateSelected(SelectedNodeOperation::Delete))
                }

                KeyCode::Char('/') => Some(AppEvent::Filter(ViewFilterOperation::Filter)),
                KeyCode::Char('*') => Some(AppEvent::Filter(ViewFilterOperation::SetDepth)),
                KeyCode::Char('.') => Some(AppEvent::Filter(ViewFilterOperation::ToggleArchived)),

                KeyCode::Char('m') => Some(AppEvent::Selection(NodeSelectionOperation::RangeMark)),
                KeyCode::Char(' ') => {
                    Some(AppEvent::Selection(NodeSelectionOperation::ToggleSelect))
                }
                _ => None,
            };
        }

        TabView::Calendar => todo!(),
    }
}
