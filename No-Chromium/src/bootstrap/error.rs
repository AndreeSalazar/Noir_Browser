//! Errores del bootstrap
//!
//! Módulo público - exportado a través de lib.rs

use thiserror::Error;

/// Errores que pueden ocurrir durante el bootstrap
#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("Configuration error: {0}")]
    InvalidConfig(String),

    #[error("Logger initialization failed: {0}")]
    LoggerInit(String),

    #[error("Application error: {0}")]
    Application(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type BootstrapResult<T> = Result<T, BootstrapError>;

impl From<String> for BootstrapError {
    fn from(s: String) -> Self {
        BootstrapError::Application(s)
    }
}

impl From<&str> for BootstrapError {
    fn from(s: &str) -> Self {
        BootstrapError::Application(s.to_string())
    }
}

impl From<anyhow::Error> for BootstrapError {
    fn from(e: anyhow::Error) -> Self {
        BootstrapError::Application(e.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for BootstrapError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        BootstrapError::Application(e.to_string())
    }
}
