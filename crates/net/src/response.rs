//! HTTP response types

use crate::{Headers, NetError};
use std::io::{BufRead, BufReader, Read};

/// An HTTP response
#[derive(Debug)]
pub struct Response {
    /// HTTP status code (e.g., 200, 404, 500)
    pub status: u16,
    /// Status reason phrase (e.g., "OK", "Not Found")
    pub reason: String,
    /// Response headers
    pub headers: Headers,
    /// Response body
    pub body: Vec<u8>,
}

impl Response {
    /// Check if the response indicates success (2xx status)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if the response is a redirect (3xx status)
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status)
    }

    /// Check if the response is a client error (4xx status)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if the response is a server error (5xx status)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Get the redirect location if this is a redirect response
    pub fn redirect_location(&self) -> Option<&str> {
        if self.is_redirect() {
            self.headers.get("location")
        } else {
            None
        }
    }

    /// Get the Content-Type header
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type")
    }

    /// Get the body as a UTF-8 string
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Parse an HTTP response from a reader
    ///
    /// This reads the status line, headers, and body according to HTTP/1.1 rules.
    pub fn parse<R: Read>(reader: R) -> Result<Self, NetError> {
        let mut buf_reader = BufReader::new(reader);

        // Read status line
        let mut status_line = String::new();
        buf_reader.read_line(&mut status_line).map_err(|e| {
            NetError::InvalidResponse(format!("failed to read status line: {}", e))
        })?;

        let (status, reason) = Self::parse_status_line(&status_line)?;

        // Read headers until empty line
        let mut headers = Headers::new();
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).map_err(|e| {
                NetError::InvalidResponse(format!("failed to read header: {}", e))
            })?;

            let line = line.trim_end_matches(['\r', '\n']);

            if line.is_empty() {
                break;
            }

            let (name, value) = line.split_once(':').ok_or_else(|| {
                NetError::InvalidResponse(format!("invalid header line: {}", line))
            })?;

            headers.set(name.trim(), value.trim());
        }

        // Read body based on headers
        let body = Self::read_body(&mut buf_reader, &headers)?;

        Ok(Response {
            status,
            reason,
            headers,
            body,
        })
    }

    /// Parse the HTTP status line
    fn parse_status_line(line: &str) -> Result<(u16, String), NetError> {
        let line = line.trim();

        // Format: HTTP/1.1 200 OK
        let mut parts = line.splitn(3, ' ');

        let version = parts.next().ok_or_else(|| {
            NetError::InvalidResponse("empty status line".to_string())
        })?;

        if !version.starts_with("HTTP/") {
            return Err(NetError::InvalidResponse(format!(
                "invalid HTTP version: {}",
                version
            )));
        }

        let status_str = parts.next().ok_or_else(|| {
            NetError::InvalidResponse("missing status code".to_string())
        })?;

        let status = status_str.parse::<u16>().map_err(|_| {
            NetError::InvalidResponse(format!("invalid status code: {}", status_str))
        })?;

        let reason = parts.next().unwrap_or("").to_string();

        Ok((status, reason))
    }

    /// Read the response body according to HTTP/1.1 rules
    fn read_body<R: BufRead>(reader: &mut R, headers: &Headers) -> Result<Vec<u8>, NetError> {
        // Check for Content-Length
        if let Some(length_str) = headers.get("content-length") {
            let length = length_str.parse::<usize>().map_err(|_| {
                NetError::InvalidResponse(format!("invalid Content-Length: {}", length_str))
            })?;

            let mut body = vec![0u8; length];
            reader.read_exact(&mut body).map_err(|e| {
                NetError::InvalidResponse(format!("failed to read body: {}", e))
            })?;

            return Ok(body);
        }

        // Check for chunked transfer encoding
        if let Some(encoding) = headers.get("transfer-encoding") {
            if encoding.to_lowercase().contains("chunked") {
                return Self::read_chunked_body(reader);
            }
        }

        // Read until connection close
        let mut body = Vec::new();
        reader.read_to_end(&mut body)?;
        Ok(body)
    }

    /// Read a chunked transfer-encoded body
    fn read_chunked_body<R: BufRead>(reader: &mut R) -> Result<Vec<u8>, NetError> {
        let mut body = Vec::new();

        loop {
            // Read chunk size line
            let mut size_line = String::new();
            reader.read_line(&mut size_line)?;

            let size_str = size_line.trim();
            // Handle chunk extensions (ignore them)
            let size_str = size_str.split(';').next().unwrap_or(size_str);

            let chunk_size = usize::from_str_radix(size_str, 16).map_err(|_| {
                NetError::InvalidResponse(format!("invalid chunk size: {}", size_str))
            })?;

            if chunk_size == 0 {
                // Read trailing headers (usually empty)
                let mut trailer = String::new();
                reader.read_line(&mut trailer)?;
                break;
            }

            // Read chunk data
            let mut chunk = vec![0u8; chunk_size];
            reader.read_exact(&mut chunk)?;
            body.extend_from_slice(&chunk);

            // Read trailing CRLF
            let mut crlf = [0u8; 2];
            reader.read_exact(&mut crlf)?;
        }

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_simple_response() {
        let raw = "HTTP/1.1 200 OK\r\n\
                   Content-Length: 5\r\n\
                   Content-Type: text/plain\r\n\
                   \r\n\
                   hello";

        let response = Response::parse(Cursor::new(raw)).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.reason, "OK");
        assert_eq!(response.headers.get("content-type"), Some("text/plain"));
        assert_eq!(response.body, b"hello");
    }

    #[test]
    fn test_parse_redirect() {
        let raw = "HTTP/1.1 301 Moved Permanently\r\n\
                   Location: https://example.com/new\r\n\
                   Content-Length: 0\r\n\
                   \r\n";

        let response = Response::parse(Cursor::new(raw)).unwrap();
        assert!(response.is_redirect());
        assert_eq!(
            response.redirect_location(),
            Some("https://example.com/new")
        );
    }

    #[test]
    fn test_parse_chunked() {
        let raw = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n";

        let response = Response::parse(Cursor::new(raw)).unwrap();
        assert_eq!(response.body, b"hello world");
    }

    #[test]
    fn test_status_checks() {
        let make_response = |status| Response {
            status,
            reason: String::new(),
            headers: Headers::new(),
            body: Vec::new(),
        };

        assert!(make_response(200).is_success());
        assert!(make_response(301).is_redirect());
        assert!(make_response(404).is_client_error());
        assert!(make_response(500).is_server_error());
    }

    #[test]
    fn test_text() {
        let response = Response {
            status: 200,
            reason: "OK".to_string(),
            headers: Headers::new(),
            body: b"Hello, World!".to_vec(),
        };

        assert_eq!(response.text().unwrap(), "Hello, World!");
    }
}
