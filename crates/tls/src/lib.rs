//! TLS glue layer - minimal wrapper over vetted implementation
//!
//! This crate provides a thin wrapper around rustls for secure TLS connections.
//! It handles certificate validation using the webpki-roots certificate store.

pub mod error;
pub mod stream;

pub use error::TlsError;
pub use stream::TlsStream;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that public types are accessible
        let _: fn() -> Result<(), TlsError> = || Err(TlsError::HandshakeError("test".into()));
    }
}
