use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum ErrorType {
    #[error("Invalid index: '{0}'")]
    InvalidIndex(usize),

    #[error("Malformed index: '{0}'")]
    MalformedIndex(String),

    #[error("Invalid alias: '{0}'")]
    InvalidAlias(String),

    #[error("Invalid date: '{0}'")]
    InvalidDate(String),

    #[error("Malformed date string: '{0}'")]
    MalformedDate(String),

    #[error("Graph looped back: {0}->...->{1}->{0}")]
    GraphLooped(usize, usize),

    #[error("Node is not a Task node: {0}")]
    NotTaskNode(usize),
}
