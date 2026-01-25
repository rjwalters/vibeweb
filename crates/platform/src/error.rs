//! Platform error types

use thiserror::Error;

/// Errors that can occur in platform operations
#[derive(Error, Debug)]
pub enum PlatformError {
    /// Failed to create the event loop
    #[error("failed to create event loop: {0}")]
    EventLoopCreation(String),

    /// Failed to create the window
    #[error("failed to create window: {0}")]
    WindowCreation(String),
}
