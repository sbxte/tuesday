use thiserror::Error;
use serde_yaml_ng;
use serde_json;

/// Error enums for the save file parsing.
#[derive(Debug, Error)]
pub enum ErrorType {
    #[error("Invalid index: '{0}'")]
    InvalidIndex(usize),

    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),

    // Some annoying person decided to make the error enum private
    #[error("Load/save operation error: {0}")]
    YAMLError(#[from] serde_yaml_ng::Error),

    // Some annoying person decided to make the error enum private
    #[error("Load/save operation error: {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("No home directory available!")]
    NoHome,

    #[error("Parse error: {0}")]
    ParseError(String),
}
