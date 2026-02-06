#![cfg_attr(coverage_nightly, coverage(off))]
// Error types for scaffolding - TICKET-PMAT-5001
// Extends existing ScaffoldError from agent module

use std::fmt;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, ScaffoldError>;

/// Scaffolding errors (extends agent::ScaffoldError)
#[derive(Debug)]
pub enum ScaffoldError {
    InvalidProjectName(String),
    DirectoryExists(PathBuf),
    IoError(std::io::Error),
    GitError(String),
    UnsupportedProjectType,
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
            ScaffoldError::UnsupportedProjectType => {
                write!(f, "Unsupported project type for hook generation")
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::error::Error;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_invalid_project_name_display() {
        let err = ScaffoldError::InvalidProjectName("bad-name!@#".to_string());
        assert_eq!(err.to_string(), "Invalid project name: bad-name!@#");
    }

    #[test]
    fn test_directory_exists_display() {
        let path = PathBuf::from("/home/user/existing_project");
        let err = ScaffoldError::DirectoryExists(path.clone());
        assert!(err.to_string().contains("Directory already exists"));
        assert!(err.to_string().contains("/home/user/existing_project"));
    }

    #[test]
    fn test_io_error_display() {
        let io_err = IoError::new(ErrorKind::PermissionDenied, "access denied");
        let err = ScaffoldError::IoError(io_err);
        assert!(err.to_string().contains("I/O error"));
        assert!(err.to_string().contains("access denied"));
    }

    #[test]
    fn test_git_error_display() {
        let err = ScaffoldError::GitError("failed to init repository".to_string());
        assert_eq!(err.to_string(), "Git error: failed to init repository");
    }

    #[test]
    fn test_unsupported_project_type_display() {
        let err = ScaffoldError::UnsupportedProjectType;
        assert_eq!(
            err.to_string(),
            "Unsupported project type for hook generation"
        );
    }

    #[test]
    fn test_agent_error_display() {
        let agent_err =
            crate::scaffold::agent::error::ScaffoldError::TemplateNotFound("test".to_string());
        let err = ScaffoldError::Agent(Box::new(agent_err));
        assert!(err.to_string().contains("Agent scaffolding error"));
        assert!(err.to_string().contains("Template not found"));
    }

    #[test]
    fn test_io_error_source() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let err = ScaffoldError::IoError(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_agent_error_source() {
        let agent_err = crate::scaffold::agent::error::ScaffoldError::UserCancelled;
        let err = ScaffoldError::Agent(Box::new(agent_err));
        assert!(err.source().is_some());
    }

    #[test]
    fn test_invalid_project_name_no_source() {
        let err = ScaffoldError::InvalidProjectName("test".to_string());
        assert!(err.source().is_none());
    }

    #[test]
    fn test_directory_exists_no_source() {
        let err = ScaffoldError::DirectoryExists(PathBuf::from("/tmp"));
        assert!(err.source().is_none());
    }

    #[test]
    fn test_git_error_no_source() {
        let err = ScaffoldError::GitError("error".to_string());
        assert!(err.source().is_none());
    }

    #[test]
    fn test_unsupported_project_type_no_source() {
        let err = ScaffoldError::UnsupportedProjectType;
        assert!(err.source().is_none());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = IoError::new(ErrorKind::AlreadyExists, "already exists");
        let scaffold_err: ScaffoldError = io_err.into();
        match scaffold_err {
            ScaffoldError::IoError(e) => {
                assert_eq!(e.kind(), ErrorKind::AlreadyExists);
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_from_agent_scaffold_error() {
        let agent_err = crate::scaffold::agent::error::ScaffoldError::InvalidConfiguration(
            "bad config".to_string(),
        );
        let scaffold_err: ScaffoldError = agent_err.into();
        match scaffold_err {
            ScaffoldError::Agent(e) => {
                assert!(e.to_string().contains("Invalid agent configuration"));
            }
            _ => panic!("Expected Agent variant"),
        }
    }

    #[test]
    fn test_result_type_alias_ok() {
        let result: Result<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_alias_err() {
        let result: Result<i32> = Err(ScaffoldError::UnsupportedProjectType);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_debug_impl() {
        let err = ScaffoldError::InvalidProjectName("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidProjectName"));
        assert!(debug_str.contains("test"));
    }
}
