use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::{app::App, components::tabs::TabView};

// TODO: oh god the Copy and Clone derived trait implementations. Check ownership conflict at
// cmdline.rs under process_input method.

/// Possible Tab Change Directions.
#[derive(PartialEq, Clone, Copy)]
pub enum TabDirection {
    Next,     // ]
    Previous, // [
}

/// Possible Navigation Actions.
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub enum ActiveNodeOperation {
    CopyTo,            // P
    LinkTo,            // S
    Modify,            // M
    Delete,            // X
    MoveTo,            // V
    Rename,            // R
    UnlinkFrom,        // D
    Check,             // C
    AddPseudoToParent, // U
    AddPseudoToActive, // u
    AddToParent,       // A
    AddToActive,       // a
}

/// Operations done with selected nodes.
#[derive(Clone, Copy)]
pub enum SelectedNodeOperation {
    CopyTo,     // p
    Delete,     // x
    LinkTo,     // s
    MoveTo,     // v
    UnlinkFrom, // d
    Check,      // c
}

/// Node filtering operations.
#[derive(Clone, Copy)]
pub enum ViewFilterOperation {
    Filter,         // /
    SetDepth,       // *
    ToggleArchived, // .
    JumpNext,
    JumpPrev,
}

/// Node selection operations
#[derive(Clone, Copy)]
pub enum NodeSelectionOperation {
    RangeMark,    // m
    ToggleSelect, // space
}

#[derive(Clone, Copy)]
pub enum AskPromptType {
    Confirmation(OperationalEvent),
    Input(OperationalEvent),
    Continual(OperationalEvent),
}
/// Internal Events
pub enum InternalEvent {
    // TODO: should we use callbacks?????
    AskPrompt(AskPromptType, String),
    StopPrompt,
    ForwardKey(KeyCode),
}

/// App Events
#[derive(Clone, Copy)]
pub enum OperationalEvent {
    Filter(ViewFilterOperation),
    Help, // H or F1
    Navigate(NavDirection),
    OperateActiveNode(ActiveNodeOperation),
    OperateSelected(SelectedNodeOperation),
    Quit, // q
    Selection(NodeSelectionOperation),
    TabChange(TabDirection),
}

pub enum AppEvent {
    Operational(OperationalEvent),
    Internal(InternalEvent),
}
/// Process key inputs based on context and emit the appropriate event.
pub fn process_key(app: &App, key_event: KeyEvent) -> Option<AppEvent> {
    if key_event.kind == KeyEventKind::Release {
        return None;
    }

    // Forward key code as event if we are capturing key input
    if app.is_capturing_keys() {
        return Some(AppEvent::Internal(InternalEvent::ForwardKey(
            key_event.code,
        )));
    }

    match key_event.code {
        KeyCode::Char(']') | KeyCode::Tab => {
            return Some(AppEvent::Operational(OperationalEvent::TabChange(
                TabDirection::Next,
            )))
        }
        KeyCode::Char('[') | KeyCode::BackTab => {
            return Some(AppEvent::Operational(OperationalEvent::TabChange(
                TabDirection::Previous,
            )))
        }
        KeyCode::Char('q') => return Some(AppEvent::Operational(OperationalEvent::Quit)),
        KeyCode::F(1) => return Some(AppEvent::Operational(OperationalEvent::Help)),
        _ => (),
    }

    // Specific to current view
    match app.current_view() {
        TabView::Tasks | TabView::DateGraph => {
            if !app.graph_is_loaded() {
                return None;
            };
            match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => Some(AppEvent::Operational(
                    OperationalEvent::Navigate(NavDirection::Next),
                )),
                KeyCode::Char('k') | KeyCode::Up => Some(AppEvent::Operational(
                    OperationalEvent::Navigate(NavDirection::Previous),
                )),
                KeyCode::Char('l') | KeyCode::Right => Some(AppEvent::Operational(
                    OperationalEvent::Navigate(NavDirection::StepIn),
                )),
                KeyCode::Char('h') | KeyCode::Left => Some(AppEvent::Operational(
                    OperationalEvent::Navigate(NavDirection::StepOut),
                )),
                KeyCode::Char('`') => Some(AppEvent::Operational(OperationalEvent::Navigate(
                    NavDirection::ToRoot,
                ))),
                KeyCode::Char('~') => Some(AppEvent::Operational(OperationalEvent::Navigate(
                    NavDirection::ToggleRootView,
                ))),
                KeyCode::Char('-') => Some(AppEvent::Operational(OperationalEvent::Navigate(
                    NavDirection::LastLocation,
                ))),
                KeyCode::Char('J') => Some(AppEvent::Operational(OperationalEvent::Navigate(
                    NavDirection::JumpTo,
                ))),
                KeyCode::Char('g') => Some(AppEvent::Operational(OperationalEvent::Navigate(
                    NavDirection::First,
                ))),
                KeyCode::Char('G') => Some(AppEvent::Operational(OperationalEvent::Navigate(
                    NavDirection::Last,
                ))),

                KeyCode::Char('P') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::CopyTo),
                )),
                KeyCode::Char('V') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::MoveTo),
                )),
                KeyCode::Char('R') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::Rename),
                )),
                KeyCode::Char('M') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::Modify),
                )),
                KeyCode::Char('S') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::LinkTo),
                )),
                KeyCode::Char('D') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::UnlinkFrom),
                )),
                KeyCode::Char('X') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::Delete),
                )),
                KeyCode::Char('A') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::AddToParent),
                )),
                KeyCode::Char('a') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::AddToActive),
                )),
                KeyCode::Char('U') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::AddPseudoToParent),
                )),
                KeyCode::Char('u') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::AddPseudoToActive),
                )),

                KeyCode::Char('C') => Some(AppEvent::Operational(
                    OperationalEvent::OperateActiveNode(ActiveNodeOperation::Check),
                )),

                KeyCode::Char('p') => Some(AppEvent::Operational(
                    OperationalEvent::OperateSelected(SelectedNodeOperation::CopyTo),
                )),
                KeyCode::Char('v') => Some(AppEvent::Operational(
                    OperationalEvent::OperateSelected(SelectedNodeOperation::MoveTo),
                )),
                KeyCode::Char('d') => Some(AppEvent::Operational(
                    OperationalEvent::OperateSelected(SelectedNodeOperation::UnlinkFrom),
                )),
                KeyCode::Char('x') => Some(AppEvent::Operational(
                    OperationalEvent::OperateSelected(SelectedNodeOperation::Delete),
                )),

                KeyCode::Char('/') => Some(AppEvent::Operational(OperationalEvent::Filter(
                    ViewFilterOperation::Filter,
                ))),
                KeyCode::Char('*') => Some(AppEvent::Operational(OperationalEvent::Filter(
                    ViewFilterOperation::SetDepth,
                ))),
                KeyCode::Char('.') => Some(AppEvent::Operational(OperationalEvent::Filter(
                    ViewFilterOperation::ToggleArchived,
                ))),
                KeyCode::Char('n') => Some(AppEvent::Operational(OperationalEvent::Filter(
                    ViewFilterOperation::JumpNext,
                ))),
                KeyCode::Char('N') => Some(AppEvent::Operational(OperationalEvent::Filter(
                    ViewFilterOperation::JumpPrev,
                ))),

                KeyCode::Char('m') => Some(AppEvent::Operational(OperationalEvent::Selection(
                    NodeSelectionOperation::RangeMark,
                ))),
                KeyCode::Char(' ') => Some(AppEvent::Operational(OperationalEvent::Selection(
                    NodeSelectionOperation::ToggleSelect,
                ))),
                _ => None,
            }
        }

        TabView::Calendar => todo!(),
    }
}
