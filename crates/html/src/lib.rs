//! HTML tokenizer and tree builder
//!
//! This crate provides HTML parsing capabilities for the vibeweb browser.
//! It implements a two-phase parsing approach:
//!
//! 1. **Tokenizer**: Converts HTML text into a stream of tokens
//! 2. **Tree Builder**: Constructs a DOM tree from the token stream
//!
//! The implementation focuses on common HTML patterns rather than full
//! HTML5 spec compliance, following the project's "subset compliance" approach.

mod tokenizer;
mod tree_builder;

pub use tokenizer::{Token, Tokenizer};
pub use tree_builder::TreeBuilder;

use vw_dom::Document;

/// Parses an HTML string into a DOM Document.
///
/// This is the main entry point for HTML parsing. It creates a tokenizer
/// and tree builder, processes all tokens, and returns the resulting document.
///
/// # Example
///
/// ```
/// use vw_html::parse;
///
/// let doc = parse("<html><body><p>Hello, World!</p></body></html>");
/// assert!(doc.document_element().is_some());
/// ```
pub fn parse(html: &str) -> Document {
    let mut builder = TreeBuilder::new();
    for token in Tokenizer::new(html) {
        builder.process(token);
    }
    builder.finish()
}

#[cfg(test)]
mod tests;
