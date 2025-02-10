use std::fmt;

use thiserror::Error;
use tuecore::graph;
use tuecore::doc;

#[derive(Error)]
pub(crate) enum AppError {
    #[error("Graph error: {0}\n")]
    GraphError(#[from] graph::errors::ErrorType),

    #[error("Load/save operation error: {0}")]
    DocError(#[from] doc::errors::ErrorType),

    #[error("Conflicting arguments: {0}")]
    ConflictingArgs(String),

    #[error("Node doesn't have children to pick from")]
    NodeNoChildren,

    #[error("No subcommand given, try --help to view all subcommands and options")]
    NoSubcommand,

    #[error("Invalid argument(s): {0}")]
    InvalidArg(String),

    #[error("Malformed date argument: {0}")]
    MalformedDate(String),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error)
}

// The default Debug implementation displays the enum like so:
// InvalidArg("content inside") -- which is not quite helpful since this used to
// display the item using the Termination trait at our main function.
impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
    
}
