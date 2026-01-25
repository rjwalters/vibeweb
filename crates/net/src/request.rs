//! HTTP request types

use crate::{Headers, Url};

/// HTTP request method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// GET request
    Get,
    /// POST request
    Post,
    /// HEAD request (like GET but no body)
    Head,
    /// PUT request
    Put,
    /// DELETE request
    Delete,
    /// OPTIONS request
    Options,
}

impl Method {
    /// Get the method name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Head => "HEAD",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Options => "OPTIONS",
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An HTTP request
#[derive(Debug)]
pub struct Request {
    /// The HTTP method
    pub method: Method,
    /// The target URL
    pub url: Url,
    /// Request headers
    pub headers: Headers,
    /// Optional request body
    pub body: Option<Vec<u8>>,
}

impl Request {
    /// Create a new GET request for the given URL
    pub fn get(url: Url) -> Self {
        Self {
            method: Method::Get,
            url,
            headers: Headers::new(),
            body: None,
        }
    }

    /// Create a new POST request with the given body
    pub fn post(url: Url, body: Vec<u8>) -> Self {
        let mut headers = Headers::new();
        headers.set("Content-Length", &body.len().to_string());

        Self {
            method: Method::Post,
            url,
            headers,
            body: Some(body),
        }
    }

    /// Create a new HEAD request
    pub fn head(url: Url) -> Self {
        Self {
            method: Method::Head,
            url,
            headers: Headers::new(),
            body: None,
        }
    }

    /// Set a header value
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.set(name, value);
        self
    }

    /// Format the request as HTTP/1.1 wire format
    ///
    /// This produces a valid HTTP/1.1 request ready to send over the wire.
    pub fn format(&self) -> Vec<u8> {
        let mut result = String::new();

        // Request line
        result.push_str(self.method.as_str());
        result.push(' ');
        result.push_str(&self.url.request_target());
        result.push_str(" HTTP/1.1\r\n");

        // Add Host header if not present
        if !self.headers.contains("Host") {
            result.push_str("Host: ");
            result.push_str(&self.url.host_header());
            result.push_str("\r\n");
        }

        // Add default headers
        if !self.headers.contains("User-Agent") {
            result.push_str("User-Agent: vibeweb/0.1\r\n");
        }

        if !self.headers.contains("Accept") {
            result.push_str("Accept: */*\r\n");
        }

        if !self.headers.contains("Connection") {
            result.push_str("Connection: close\r\n");
        }

        // Add remaining headers
        result.push_str(&self.headers.format());

        // End of headers
        result.push_str("\r\n");

        let mut bytes = result.into_bytes();

        // Add body if present
        if let Some(ref body) = self.body {
            bytes.extend_from_slice(body);
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_request() {
        let url = Url::parse("http://example.com/path").unwrap();
        let req = Request::get(url);
        assert_eq!(req.method, Method::Get);
        assert!(req.body.is_none());
    }

    #[test]
    fn test_post_request() {
        let url = Url::parse("http://example.com/api").unwrap();
        let body = b"hello".to_vec();
        let req = Request::post(url, body);
        assert_eq!(req.method, Method::Post);
        assert_eq!(req.headers.get("content-length"), Some("5"));
    }

    #[test]
    fn test_format_request() {
        let url = Url::parse("http://example.com/path?foo=bar").unwrap();
        let req = Request::get(url);
        let formatted = String::from_utf8(req.format()).unwrap();

        assert!(formatted.starts_with("GET /path?foo=bar HTTP/1.1\r\n"));
        assert!(formatted.contains("Host: example.com\r\n"));
        assert!(formatted.contains("User-Agent: vibeweb/0.1\r\n"));
        assert!(formatted.ends_with("\r\n\r\n"));
    }

    #[test]
    fn test_custom_header() {
        let url = Url::parse("http://example.com/").unwrap();
        let req = Request::get(url).header("Accept", "text/html");
        let formatted = String::from_utf8(req.format()).unwrap();
        assert!(formatted.contains("Accept: text/html\r\n"));
    }
}
