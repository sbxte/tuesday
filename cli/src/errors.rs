use crate::blueprints::BlueprintError;

use parse_datetime::ParseDateTimeError;
use std::fmt;
use thiserror::Error;
use tuecore::doc;
use tuecore::graph;

use crate::config::ConfigReadError;

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
    IndexRetrievalError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigReadError),

    #[error("Blueprint error: {0}")]
    BlueprintError(#[from] BlueprintError),
}

// The default Debug implementation displays the enum like so:
// InvalidArg("content inside") -- which is not quite helpful since this used to
// display the item using the Termination trait at our main function.
impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
