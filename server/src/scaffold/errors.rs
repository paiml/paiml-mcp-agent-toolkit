// Error types for scaffolding - TICKET-PMAT-5001
// Extends existing ScaffoldError from agent module

use std::path::PathBuf;
use std::fmt;

pub type Result<T> = std::result::Result<T, ScaffoldError>;

/// Scaffolding errors (extends agent::ScaffoldError)
#[derive(Debug)]
pub enum ScaffoldError {
    InvalidProjectName(String),
    DirectoryExists(PathBuf),
    IoError(std::io::Error),
    GitError(String),
    // Re-export agent scaffold errors
    Agent(Box<crate::scaffold::agent::error::ScaffoldError>),
}

impl fmt::Display for ScaffoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaffoldError::InvalidProjectName(name) => {
                write!(f, "Invalid project name: {}", name)
            }
            ScaffoldError::DirectoryExists(path) => {
                write!(f, "Directory already exists: {}", path.display())
            }
            ScaffoldError::IoError(e) => {
                write!(f, "I/O error: {}", e)
            }
            ScaffoldError::GitError(msg) => {
                write!(f, "Git error: {}", msg)
            }
            ScaffoldError::Agent(e) => {
                write!(f, "Agent scaffolding error: {}", e)
            }
        }
    }
}

impl std::error::Error for ScaffoldError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ScaffoldError::IoError(e) => Some(e),
            ScaffoldError::Agent(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ScaffoldError {
    fn from(e: std::io::Error) -> Self {
        ScaffoldError::IoError(e)
    }
}

impl From<crate::scaffold::agent::error::ScaffoldError> for ScaffoldError {
    fn from(e: crate::scaffold::agent::error::ScaffoldError) -> Self {
        ScaffoldError::Agent(Box::new(e))
    }
}
