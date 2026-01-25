//! Error types for the renderer crate.

use thiserror::Error;

/// Errors that can occur during rendering.
#[derive(Error, Debug)]
pub enum Error {
    /// HTML parsing produced an invalid document structure.
    #[error("invalid document structure: {0}")]
    InvalidDocument(String),
}

/// Result type for renderer operations.
pub type Result<T> = std::result::Result<T, Error>;
