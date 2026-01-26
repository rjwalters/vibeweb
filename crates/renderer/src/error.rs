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

    #[test]
    fn error_display_invalid_document() {
        let err = Error::InvalidDocument("missing root element".to_string());
        assert_eq!(
            format!("{}", err),
            "invalid document structure: missing root element"
        );
    }

    #[test]
    fn error_display_not_found() {
        let err = Error::NotFound("/path/to/resource".to_string());
        assert_eq!(format!("{}", err), "resource not found: /path/to/resource");
    }

    #[test]
    fn error_display_timeout() {
        let err = Error::Timeout("https://example.com".to_string());
        assert_eq!(
            format!("{}", err),
            "timeout loading resource: https://example.com"
        );
    }

    #[test]
    fn error_from_net_error() {
        let net_err = vw_net::NetError::InvalidUrl("not a url".to_string());
        let renderer_err: Error = net_err.into();
        assert!(matches!(renderer_err, Error::Network(_)));
    }

    #[test]
    fn error_from_tls_error() {
        let tls_err = vw_tls::TlsError::InvalidServerName("bad name".to_string());
        let renderer_err: Error = tls_err.into();
        assert!(matches!(renderer_err, Error::Tls(_)));
    }

    #[test]
    fn error_from_platform_error() {
        let platform_err = vw_platform::PlatformError::WindowCreation("failed".to_string());
        let renderer_err: Error = platform_err.into();
        assert!(matches!(renderer_err, Error::Platform(_)));
    }

    #[test]
    fn error_from_image_error() {
        let image_err = vw_image::ImageError::UnsupportedFormat;
        let renderer_err: Error = image_err.into();
        assert!(matches!(renderer_err, Error::Image(_)));
    }

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
