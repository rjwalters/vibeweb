//! URL parsing and representation

use crate::NetError;

/// A parsed URL
///
/// Represents a URL broken down into its component parts.
/// Currently supports http and https schemes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    /// The URL scheme (http or https)
    pub scheme: String,
    /// The hostname
    pub host: String,
    /// Optional port number (defaults to 80 for http, 443 for https)
    pub port: Option<u16>,
    /// The path component (defaults to "/")
    pub path: String,
    /// Optional query string (without the leading '?')
    pub query: Option<String>,
    /// Optional fragment (without the leading '#')
    pub fragment: Option<String>,
}

impl Url {
    /// Parse a URL string into its components
    ///
    /// # Examples
    ///
    /// ```
    /// use vw_net::Url;
    ///
    /// let url = Url::parse("https://example.com:8080/path?query=value#section").unwrap();
    /// assert_eq!(url.scheme, "https");
    /// assert_eq!(url.host, "example.com");
    /// assert_eq!(url.port, Some(8080));
    /// assert_eq!(url.path, "/path");
    /// assert_eq!(url.query, Some("query=value".to_string()));
    /// assert_eq!(url.fragment, Some("section".to_string()));
    /// ```
    pub fn parse(url: &str) -> Result<Self, NetError> {
        // Find the scheme separator
        let (scheme, rest) = url
            .split_once("://")
            .ok_or_else(|| NetError::InvalidUrl("missing scheme separator '://'".to_string()))?;

        // Validate scheme
        let scheme = scheme.to_lowercase();
        if scheme != "http" && scheme != "https" {
            return Err(NetError::UnsupportedScheme(scheme));
        }

        // Split off fragment first (it's not sent to server)
        let (rest, fragment) = match rest.split_once('#') {
            Some((before, frag)) => (before, Some(frag.to_string())),
            None => (rest, None),
        };

        // Split off query string
        let (rest, query) = match rest.split_once('?') {
            Some((before, q)) => (before, Some(q.to_string())),
            None => (rest, None),
        };

        // Split authority (host:port) from path
        let (authority, path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };

        // Parse host and optional port
        let (host, port) = if authority.contains('[') {
            // IPv6 address: [::1]:8080
            Self::parse_ipv6_authority(authority)?
        } else {
            Self::parse_authority(authority)?
        };

        if host.is_empty() {
            return Err(NetError::InvalidUrl("empty host".to_string()));
        }

        Ok(Url {
            scheme,
            host,
            port,
            path: path.to_string(),
            query,
            fragment,
        })
    }

    /// Parse a regular (non-IPv6) authority section
    fn parse_authority(authority: &str) -> Result<(String, Option<u16>), NetError> {
        match authority.rsplit_once(':') {
            Some((host, port_str)) => {
                let port = port_str
                    .parse::<u16>()
                    .map_err(|_| NetError::InvalidUrl(format!("invalid port: {}", port_str)))?;
                Ok((host.to_string(), Some(port)))
            }
            None => Ok((authority.to_string(), None)),
        }
    }

    /// Parse an IPv6 authority section like [::1]:8080
    fn parse_ipv6_authority(authority: &str) -> Result<(String, Option<u16>), NetError> {
        let close_bracket = authority
            .find(']')
            .ok_or_else(|| NetError::InvalidUrl("unclosed IPv6 bracket".to_string()))?;

        let host = authority[1..close_bracket].to_string(); // Strip brackets
        let after_bracket = &authority[close_bracket + 1..];

        let port = if after_bracket.starts_with(':') {
            let port_str = &after_bracket[1..];
            Some(
                port_str
                    .parse::<u16>()
                    .map_err(|_| NetError::InvalidUrl(format!("invalid port: {}", port_str)))?,
            )
        } else {
            None
        };

        Ok((host, port))
    }

    /// Get the effective port (using default if not specified)
    pub fn effective_port(&self) -> u16 {
        self.port.unwrap_or(match self.scheme.as_str() {
            "https" => 443,
            _ => 80,
        })
    }

    /// Check if this URL uses HTTPS
    pub fn is_https(&self) -> bool {
        self.scheme == "https"
    }

    /// Get the host:port string for the Host header
    pub fn host_header(&self) -> String {
        match self.port {
            Some(port) => format!("{}:{}", self.host, port),
            None => self.host.clone(),
        }
    }

    /// Get the request target (path + query) for the HTTP request line
    pub fn request_target(&self) -> String {
        match &self.query {
            Some(q) => format!("{}?{}", self.path, q),
            None => self.path.clone(),
        }
    }

    /// Resolve a relative URL against this URL as the base
    pub fn join(&self, relative: &str) -> Result<Url, NetError> {
        // Handle absolute URLs
        if relative.contains("://") {
            return Url::parse(relative);
        }

        // Handle protocol-relative URLs
        if relative.starts_with("//") {
            return Url::parse(&format!("{}:{}", self.scheme, relative));
        }

        // Handle absolute paths
        if relative.starts_with('/') {
            return Ok(Url {
                scheme: self.scheme.clone(),
                host: self.host.clone(),
                port: self.port,
                path: relative.to_string(),
                query: None,
                fragment: None,
            });
        }

        // Handle relative paths
        let base_path = if self.path.ends_with('/') {
            self.path.clone()
        } else {
            match self.path.rfind('/') {
                Some(idx) => self.path[..=idx].to_string(),
                None => "/".to_string(),
            }
        };

        let new_path = format!("{}{}", base_path, relative);

        Ok(Url {
            scheme: self.scheme.clone(),
            host: self.host.clone(),
            port: self.port,
            path: Self::normalize_path(&new_path),
            query: None,
            fragment: None,
        })
    }

    /// Normalize a path by resolving . and .. segments
    fn normalize_path(path: &str) -> String {
        let mut segments: Vec<&str> = Vec::new();

        for segment in path.split('/') {
            match segment {
                "" | "." => {}
                ".." => {
                    segments.pop();
                }
                s => segments.push(s),
            }
        }

        let result = format!("/{}", segments.join("/"));
        if path.ends_with('/') && !result.ends_with('/') {
            format!("{}/", result)
        } else {
            result
        }
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}", self.scheme, self.host)?;
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        write!(f, "{}", self.path)?;
        if let Some(ref query) = self.query {
            write!(f, "?{}", query)?;
        }
        if let Some(ref fragment) = self.fragment {
            write!(f, "#{}", fragment)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_http() {
        let url = Url::parse("http://example.com").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/");
        assert_eq!(url.effective_port(), 80);
    }

    #[test]
    fn test_parse_https_with_port() {
        let url = Url::parse("https://example.com:8443/path").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, Some(8443));
        assert_eq!(url.path, "/path");
        assert_eq!(url.effective_port(), 8443);
    }

    #[test]
    fn test_parse_with_query_and_fragment() {
        let url = Url::parse("http://example.com/path?key=value&foo=bar#section").unwrap();
        assert_eq!(url.query, Some("key=value&foo=bar".to_string()));
        assert_eq!(url.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_parse_ipv6() {
        let url = Url::parse("http://[::1]:8080/path").unwrap();
        assert_eq!(url.host, "::1");
        assert_eq!(url.port, Some(8080));
    }

    #[test]
    fn test_invalid_scheme() {
        let result = Url::parse("ftp://example.com");
        assert!(matches!(result, Err(NetError::UnsupportedScheme(_))));
    }

    #[test]
    fn test_missing_scheme() {
        let result = Url::parse("example.com/path");
        assert!(matches!(result, Err(NetError::InvalidUrl(_))));
    }

    #[test]
    fn test_request_target() {
        let url = Url::parse("http://example.com/path?query=1").unwrap();
        assert_eq!(url.request_target(), "/path?query=1");
    }

    #[test]
    fn test_join_absolute() {
        let base = Url::parse("http://example.com/dir/page").unwrap();
        let joined = base.join("https://other.com/new").unwrap();
        assert_eq!(joined.to_string(), "https://other.com/new");
    }

    #[test]
    fn test_join_absolute_path() {
        let base = Url::parse("http://example.com/dir/page").unwrap();
        let joined = base.join("/newpath").unwrap();
        assert_eq!(joined.to_string(), "http://example.com/newpath");
    }

    #[test]
    fn test_join_relative_path() {
        let base = Url::parse("http://example.com/dir/page").unwrap();
        let joined = base.join("other").unwrap();
        assert_eq!(joined.to_string(), "http://example.com/dir/other");
    }

    #[test]
    fn test_display() {
        let url = Url::parse("https://example.com:8080/path?q=1#frag").unwrap();
        assert_eq!(url.to_string(), "https://example.com:8080/path?q=1#frag");
    }
}
