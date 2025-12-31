//! Error handling for the mixed-pickles CLI
use std::{io, path::PathBuf};
use thiserror::Error;

use crate::config::ConfigError;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CLIError {
    #[error("Path '{0}' is not a git repository")]
    NotARepository(PathBuf),
    #[error("Path '{0}' does not exist")]
    PathNotFound(PathBuf),
    #[error("Git command failed: {0}")]
    GitCommandFailed(String),
    #[error("No commits found in repository")]
    NoCommitsFound,
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Found {0} commits with validation issues")]
    ValidationFailed(usize),
    #[error("Invalid validation type: {0}")]
    InvalidValidation(String),
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
}
