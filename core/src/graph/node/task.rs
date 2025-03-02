use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskData {
    pub state: TaskState,
}

// TODO: Decouple this from core
impl std::fmt::Display for TaskData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.state)
    }
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
pub enum TaskState {
    #[default]
    None,
    Partial,
    Done,
}

// TODO: Decouple this from core
impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskState::None => write!(f, " "),
            TaskState::Partial => write!(f, "~"),
            TaskState::Done => write!(f, "x"),
        }
    }
}
