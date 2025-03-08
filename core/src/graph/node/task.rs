use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskData {
    pub state: TaskState,
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
pub enum TaskState {
    #[default]
    None,
    Partial,
    Done,
}
