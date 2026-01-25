//! TLS stream wrapper over rustls

use crate::TlsError;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::io::{Read, Write};
use std::sync::Arc;

/// A TLS-encrypted stream wrapping an underlying transport
///
/// This provides a simple interface for establishing TLS connections
/// using rustls with the webpki-roots certificate store.
pub struct TlsStream<S: Read + Write> {
    inner: StreamOwned<ClientConnection, S>,
}

impl<S: Read + Write> TlsStream<S> {
    /// Establish a TLS connection over the given stream
    ///
    /// # Arguments
    /// * `stream` - The underlying transport (typically a TCP stream)
    /// * `server_name` - The server hostname for SNI and certificate validation
    ///
    /// # Errors
    /// Returns an error if:
    /// - The server name is invalid for DNS
    /// - TLS handshake fails
    /// - Certificate validation fails
    pub fn connect(stream: S, server_name: &str) -> Result<Self, TlsError> {
        // Build root certificate store from webpki-roots
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        // Create client configuration with safe defaults
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Parse the server name for SNI
        let server_name: ServerName<'_> = server_name.to_owned().try_into()?;

        // Create the client connection
        let conn = ClientConnection::new(Arc::new(config), server_name)?;

        // Wrap in StreamOwned for easy I/O
        let tls_stream = StreamOwned::new(conn, stream);

        Ok(TlsStream { inner: tls_stream })
    }

    /// Get a reference to the underlying TLS connection
    pub fn connection(&self) -> &ClientConnection {
        &self.inner.conn
    }

    /// Get a mutable reference to the underlying stream
    pub fn get_mut(&mut self) -> &mut StreamOwned<ClientConnection, S> {
        &mut self.inner
    }

    /// Consume the TLS stream and return the underlying transport
    pub fn into_inner(self) -> S {
        self.inner.sock
    }
}

impl<S: Read + Write> Read for TlsStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<S: Read + Write> Write for TlsStream<S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_server_name() {
        // Empty server names should fail
        use std::io::Cursor;
        let stream = Cursor::new(Vec::new());
        let result = TlsStream::connect(stream, "");
        assert!(result.is_err());
    }
}
