//! TLS error types

use thiserror::Error;

/// Errors that can occur during TLS operations
#[derive(Debug, Error)]
pub enum TlsError {
    /// Failed to establish TLS handshake
    #[error("TLS handshake failed: {0}")]
    HandshakeError(String),

    /// Invalid server name for SNI
    #[error("invalid server name: {0}")]
    InvalidServerName(String),

    /// I/O error during TLS operation
    #[error("TLS I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Certificate validation failed
    #[error("certificate validation failed: {0}")]
    CertificateError(String),

    /// TLS protocol error
    #[error("TLS protocol error: {0}")]
    ProtocolError(String),
}

impl From<rustls::Error> for TlsError {
    fn from(err: rustls::Error) -> Self {
        TlsError::ProtocolError(err.to_string())
    }
}

impl From<rustls::pki_types::InvalidDnsNameError> for TlsError {
    fn from(err: rustls::pki_types::InvalidDnsNameError) -> Self {
        TlsError::InvalidServerName(err.to_string())
    }
}
