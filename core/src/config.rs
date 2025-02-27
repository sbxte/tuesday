use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// The default config file name
pub const DEFAULT_FILENAME: &str = ".tueconf";

/// Returns the default config file located at the user's home directory
/// If the file does not exist then it returns `None`
pub fn get_home_default() -> Option<PathBuf> {
    home::home_dir()
        .map(|mut pathbuf| {
            pathbuf.push(DEFAULT_FILENAME);
            pathbuf
        })
        .filter(|pb| pb.exists() && pb.is_file())
}

/// Returns the default config file at a given directory path
///
/// - If a/b/c is a directory, a/b/c/[`DEFAULT_FILENAME`]
/// - If none found so far, returns [`get_home_default`]
pub fn get_default_at(mut pathbuf: PathBuf) -> Option<PathBuf> {
    if pathbuf.exists() && pathbuf.is_dir() {
        pathbuf.push(DEFAULT_FILENAME);
        if pathbuf.exists() && pathbuf.is_file() {
            return Some(pathbuf);
        } else {
            return None;
        }
    }

    get_home_default()
}

/// Parses a file at path into a toml table
pub fn read_file(path: &Path) -> Result<toml::Table, ReadError> {
    if !path.exists() {
        return Err(ReadError::NonexistantFile(path.to_path_buf()));
    }

    let mut file = OpenOptions::new().read(true).open(path)?;

    let mut string = String::new();
    file.read_to_string(&mut string)?;

    Ok(string.parse::<toml::Table>()?)
}

/// Represents an error during reading a config file
#[derive(Debug, Error)]
pub enum ReadError {
    #[error("File does not exist: {0}")]
    NonexistantFile(PathBuf),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("TOML Deserialization error: {0}")]
    TOMLDeserializeErr(#[from] toml::de::Error),
}

/// Parses core configurations from a toml table
/// Any missing or malformed values will be replaced with defaults
/// See
#[allow(unused_mut, unused_variables)]
pub fn parse_config(toml: &toml::Table) -> CoreConfig {
    let mut conf = CoreConfig::new();

    // Core configurations here
    // TODO: Populate this and remove the warning allowance

    conf
}

pub fn try_parse_config(toml: &toml::Table) -> Result<CoreConfig, ParseError> {
    todo!("try_parse_config is unimplemented!");
}

#[derive(Debug, Error)]
pub enum ParseError {}

pub struct CoreConfig {}

impl CoreConfig {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self::new()
    }
}
