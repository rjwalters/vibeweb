//! HTTP headers collection

use std::collections::HashMap;

/// A case-insensitive HTTP headers collection
///
/// Header names are stored in lowercase for case-insensitive lookup,
/// but the original casing is preserved for display.
#[derive(Debug, Clone, Default)]
pub struct Headers {
    /// Internal storage: lowercase name -> (original name, value)
    inner: HashMap<String, (String, String)>,
}

impl Headers {
    /// Create a new empty headers collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a header value, replacing any existing value
    ///
    /// # Examples
    ///
    /// ```
    /// use vw_net::Headers;
    ///
    /// let mut headers = Headers::new();
    /// headers.set("Content-Type", "text/html");
    /// assert_eq!(headers.get("content-type"), Some("text/html"));
    /// ```
    pub fn set(&mut self, name: &str, value: &str) {
        self.inner
            .insert(name.to_lowercase(), (name.to_string(), value.to_string()));
    }

    /// Get a header value by name (case-insensitive)
    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner
            .get(&name.to_lowercase())
            .map(|(_, v)| v.as_str())
    }

    /// Check if a header exists
    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(&name.to_lowercase())
    }

    /// Remove a header and return its value
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.inner.remove(&name.to_lowercase()).map(|(_, v)| v)
    }

    /// Iterate over all headers
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.values().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Get the number of headers
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the headers collection is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Parse headers from raw HTTP header lines
    ///
    /// Each line should be in the format "Name: Value"
    pub fn parse_lines<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Self, &'static str> {
        let mut headers = Self::new();

        for line in lines {
            if line.is_empty() {
                continue;
            }

            let (name, value) = line
                .split_once(':')
                .ok_or("invalid header format: missing colon")?;

            headers.set(name.trim(), value.trim());
        }

        Ok(headers)
    }

    /// Format headers for HTTP request/response
    pub fn format(&self) -> String {
        let mut result = String::new();
        for (name, value) in self.iter() {
            result.push_str(name);
            result.push_str(": ");
            result.push_str(value);
            result.push_str("\r\n");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut headers = Headers::new();
        headers.set("Content-Type", "text/html");
        assert_eq!(headers.get("Content-Type"), Some("text/html"));
        assert_eq!(headers.get("content-type"), Some("text/html"));
        assert_eq!(headers.get("CONTENT-TYPE"), Some("text/html"));
    }

    #[test]
    fn test_replace() {
        let mut headers = Headers::new();
        headers.set("Content-Type", "text/html");
        headers.set("Content-Type", "application/json");
        assert_eq!(headers.get("content-type"), Some("application/json"));
    }

    #[test]
    fn test_contains() {
        let mut headers = Headers::new();
        headers.set("Accept", "*/*");
        assert!(headers.contains("Accept"));
        assert!(headers.contains("accept"));
        assert!(!headers.contains("Content-Type"));
    }

    #[test]
    fn test_parse_lines() {
        let lines = vec!["Content-Type: text/html", "Content-Length: 42", ""];
        let headers = Headers::parse_lines(lines.iter().map(|s| *s)).unwrap();
        assert_eq!(headers.get("content-type"), Some("text/html"));
        assert_eq!(headers.get("content-length"), Some("42"));
    }

    #[test]
    fn test_format() {
        let mut headers = Headers::new();
        headers.set("Host", "example.com");
        let formatted = headers.format();
        assert!(formatted.contains("Host: example.com\r\n"));
    }
}
