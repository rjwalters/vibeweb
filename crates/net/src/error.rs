//! Network error types

use thiserror::Error;

/// Errors that can occur during network operations
#[derive(Debug, Error)]
pub enum NetError {
    /// Invalid URL format
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// Unsupported URL scheme (not http or https)
    #[error("unsupported scheme: {0}")]
    UnsupportedScheme(String),

    /// Failed to resolve hostname
    #[error("DNS resolution failed for {0}: {1}")]
    DnsError(String, String),

    /// Failed to connect to server
    #[error("connection failed to {0}:{1}: {2}")]
    ConnectionFailed(String, u16, String),

    /// I/O error during network operation
    #[error("network I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// TLS error
    #[error("TLS error: {0}")]
    TlsError(#[from] vw_tls::TlsError),

    /// Invalid HTTP response
    #[error("invalid HTTP response: {0}")]
    InvalidResponse(String),

    /// HTTP protocol error
    #[error("HTTP protocol error: {0}")]
    ProtocolError(String),

    /// Too many redirects
    #[error("too many redirects (max {0})")]
    TooManyRedirects(u32),

    /// Response timeout
    #[error("request timed out")]
    Timeout,

    /// Invalid header value
    #[error("invalid header: {0}")]
    InvalidHeader(String),
}
