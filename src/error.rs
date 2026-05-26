//! Custom error types for gpufetch
//! All errors are properly typed and documented

use std::fmt;

/// Main error type for gpufetch operations
#[derive(Debug)]
pub enum GpufetchError {
    /// GPU detection failed with specific reason
    DetectionFailed(String),
    
    /// Command execution failed (IO error)
    CommandFailed(std::io::Error),
    
    /// Failed to parse system data
    ParseError(String),
    
    /// No GPU found with given criteria
    NoGpuFound,
    
    /// Backend not available
    BackendNotAvailable(String),
}

impl fmt::Display for GpufetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DetectionFailed(msg) => write!(f, "GPU detection failed: {}", msg),
            Self::CommandFailed(err) => write!(f, "Command execution failed: {}", err),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::NoGpuFound => write!(f, "No GPU found"),
            Self::BackendNotAvailable(backend) => write!(f, "Backend not available: {}", backend),
        }
    }
}

impl std::error::Error for GpufetchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CommandFailed(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for GpufetchError {
    fn from(err: std::io::Error) -> Self {
        Self::CommandFailed(err)
    }
}

impl From<std::num::ParseIntError> for GpufetchError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::ParseError(format!("Failed to parse integer: {err}"))
    }
}

impl From<std::string::FromUtf8Error> for GpufetchError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::ParseError(format!("Invalid UTF-8: {err}"))
    }
}

/// Convenience Result type using our custom error
pub type Result<T> = std::result::Result<T, GpufetchError>;
