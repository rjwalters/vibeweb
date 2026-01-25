//! HTTP client implementation

use crate::{NetError, Request, Response, Url};
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use vw_tls::TlsStream;

/// Maximum number of redirects to follow
const MAX_REDIRECTS: u32 = 10;

/// Default connection timeout in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// HTTP client for fetching URLs
///
/// This client supports both HTTP and HTTPS, with automatic redirect handling.
///
/// # Examples
///
/// ```ignore
/// use vw_net::{Client, Url};
///
/// let client = Client::new();
/// let response = client.fetch("http://example.com")?;
/// println!("Status: {}", response.status);
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    /// Connection timeout
    timeout: Duration,
    /// Maximum redirects to follow
    max_redirects: u32,
    /// User-Agent header value
    user_agent: String,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// Create a new HTTP client with default settings
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_redirects: MAX_REDIRECTS,
            user_agent: "vibeweb/0.1".to_string(),
        }
    }

    /// Set the connection timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum number of redirects to follow
    pub fn max_redirects(mut self, max: u32) -> Self {
        self.max_redirects = max;
        self
    }

    /// Set the User-Agent header
    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_string();
        self
    }

    /// Fetch a URL and return the response
    ///
    /// This follows redirects up to the configured maximum.
    pub fn fetch(&self, url: &str) -> Result<Response, NetError> {
        let url = Url::parse(url)?;
        self.fetch_url(&url)
    }

    /// Fetch a parsed URL
    pub fn fetch_url(&self, url: &Url) -> Result<Response, NetError> {
        self.fetch_with_redirects(url, 0)
    }

    /// Internal fetch with redirect tracking
    fn fetch_with_redirects(&self, url: &Url, redirect_count: u32) -> Result<Response, NetError> {
        if redirect_count > self.max_redirects {
            return Err(NetError::TooManyRedirects(self.max_redirects));
        }

        // Build request
        let request = Request::get(url.clone()).header("User-Agent", &self.user_agent);

        // Execute request
        let response = self.execute(&request)?;

        // Handle redirects
        if response.is_redirect() {
            if let Some(location) = response.redirect_location() {
                let new_url = url.join(location)?;
                return self.fetch_with_redirects(&new_url, redirect_count + 1);
            }
        }

        Ok(response)
    }

    /// Execute a request and return the response
    pub fn execute(&self, request: &Request) -> Result<Response, NetError> {
        let url = &request.url;

        // Connect to the server
        let addr = format!("{}:{}", url.host, url.effective_port());

        let stream = TcpStream::connect(&addr).map_err(|e| {
            NetError::ConnectionFailed(url.host.clone(), url.effective_port(), e.to_string())
        })?;

        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        if url.is_https() {
            self.execute_https(request, stream)
        } else {
            self.execute_http(request, stream)
        }
    }

    /// Execute HTTP request over plain TCP
    fn execute_http(&self, request: &Request, mut stream: TcpStream) -> Result<Response, NetError> {
        // Send request
        let request_bytes = request.format();
        stream.write_all(&request_bytes)?;
        stream.flush()?;

        // Parse response
        Response::parse(&stream)
    }

    /// Execute HTTPS request over TLS
    fn execute_https(&self, request: &Request, stream: TcpStream) -> Result<Response, NetError> {
        // Establish TLS connection
        let mut tls_stream = TlsStream::connect(stream, &request.url.host)?;

        // Send request
        let request_bytes = request.format();
        tls_stream.write_all(&request_bytes)?;
        tls_stream.flush()?;

        // Parse response
        Response::parse(&mut tls_stream)
    }

    /// Perform a GET request
    pub fn get(&self, url: &str) -> Result<Response, NetError> {
        self.fetch(url)
    }

    /// Perform a POST request with a body
    pub fn post(&self, url: &str, body: Vec<u8>) -> Result<Response, NetError> {
        let url = Url::parse(url)?;
        let request = Request::post(url, body).header("User-Agent", &self.user_agent);
        self.execute(&request)
    }

    /// Perform a HEAD request
    pub fn head(&self, url: &str) -> Result<Response, NetError> {
        let url = Url::parse(url)?;
        let request = Request::head(url).header("User-Agent", &self.user_agent);
        self.execute(&request)
    }
}

/// Convenience function to fetch a URL
///
/// This creates a temporary client and fetches the URL.
///
/// # Examples
///
/// ```ignore
/// let response = vw_net::fetch("http://example.com")?;
/// ```
pub fn fetch(url: &str) -> Result<Response, NetError> {
    Client::new().fetch(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder() {
        let client = Client::new()
            .timeout(Duration::from_secs(60))
            .max_redirects(5)
            .user_agent("test/1.0");

        assert_eq!(client.timeout, Duration::from_secs(60));
        assert_eq!(client.max_redirects, 5);
        assert_eq!(client.user_agent, "test/1.0");
    }

    #[test]
    fn test_default_client() {
        let client = Client::default();
        assert_eq!(client.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECS));
        assert_eq!(client.max_redirects, MAX_REDIRECTS);
    }

    // Note: Integration tests requiring network access should go in tests/ directory
    // or be marked #[ignore] for CI environments without network access
}
