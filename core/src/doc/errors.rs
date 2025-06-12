use serde_yaml_ng;
use thiserror::Error;

/// Error enums for the save file parsing.
#[derive(Debug, Error)]
pub enum ErrorType {
    #[error("Invalid index: '{0}'")]
    InvalidIndex(usize),

    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    YAMLError(#[from] serde_yaml_ng::Error),

    // Homeless?? :(
    #[error("No home directory available!")]
    NoHome,

    // TODO: Split this into more sub types
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Datetime parse error: {0}")]
    ChronoError(#[from] chrono::ParseError),
}
