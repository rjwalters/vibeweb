//! Error types for the renderer crate.

use thiserror::Error;

/// Unified error type for the browser rendering pipeline.
///
/// This error type aggregates all errors that can occur during the rendering
/// process, including errors from network, TLS, platform, and image crates.
/// This provides a single error type for the browser layer to handle.
#[derive(Error, Debug)]
pub enum Error {
    /// HTML parsing produced an invalid document structure.
    #[error("invalid document structure: {0}")]
    InvalidDocument(String),

    /// Network error during resource fetching.
    #[error("network error: {0}")]
    Network(#[from] vw_net::NetError),

    /// TLS/SSL error during secure connection.
    #[error("TLS error: {0}")]
    Tls(#[from] vw_tls::TlsError),

    /// Platform/windowing error.
    #[error("platform error: {0}")]
    Platform(#[from] vw_platform::PlatformError),

    /// Image decoding error.
    #[error("image error: {0}")]
    Image(#[from] vw_image::ImageError),

    /// Resource not found (404 or missing local file).
    #[error("resource not found: {0}")]
    NotFound(String),

    /// Resource loading timed out.
    #[error("timeout loading resource: {0}")]
    Timeout(String),
}

/// Result type for renderer operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    // Note: We don't test basic Display/From implementations since those are
    // handled by the thiserror macro. Tests focus on actual error handling logic.

    #[test]
    fn error_chain_preservation() {
        // Test that we can downcast to source errors
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let net_err = vw_net::NetError::from(io_err);
        let renderer_err = Error::from(net_err);

        // Verify the error chain is preserved
        assert!(matches!(renderer_err, Error::Network(_)));
        assert!(format!("{}", renderer_err).contains("network error"));
    }
}
