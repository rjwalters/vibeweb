//! Network layer - URL fetching, redirects, caching
//!
//! This crate provides HTTP/1.1 client functionality for the vibeweb browser.
//! It supports both HTTP and HTTPS protocols with automatic redirect handling.

pub mod client;
pub mod error;
pub mod headers;
pub mod request;
pub mod response;
pub mod url;

pub use client::Client;
pub use error::NetError;
pub use headers::Headers;
pub use request::{Method, Request};
pub use response::Response;
pub use url::Url;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that public types are accessible
        let _url = Url::parse("http://example.com").unwrap();
        let _method = Method::Get;
        let _headers = Headers::new();
    }
}
