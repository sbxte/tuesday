use std::fmt;

use parse_datetime::ParseDateTimeError;
use thiserror::Error;
use tuecore::graph;
use tuecore::doc;

#[derive(Error)]
pub(crate) enum AppError {
    #[error("Graph error: {0}")]
    GraphError(#[from] graph::errors::ErrorType),

    #[error("Load/save operation error: {0}")]
    DocError(#[from] doc::errors::ErrorType),

    #[error("Conflicting arguments: {0}")]
    ConflictingArgs(String),

    #[error("Node doesn't have children to pick from")]
    NodeNoChildren,

    #[error("Invalid subcommand given, try --help to view all subcommands and options")]
    InvalidSubcommand,

    #[error("Invalid argument(s): {0}")]
    InvalidArg(String),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Missing argument: {0}")]
    MissingArgument(String),

    #[error("Error parsing specified date: {0}")]
    DateParseError(#[from] ParseDateTimeError),

    #[error("Failed to get node index: {0}")]
    IndexRetrievalError(String)
}

// The default Debug implementation displays the enum like so:
// InvalidArg("content inside") -- which is not quite helpful since this used to
// display the item using the Termination trait at our main function.
impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
    
}
