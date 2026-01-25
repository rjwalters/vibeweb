//! HTML tokenizer and tree builder.
//!
//! This crate provides HTML parsing functionality for vibeweb.
//! It implements a simple recursive descent parser that produces
//! a DOM tree from HTML input.

mod parser;
mod tokenizer;

pub use parser::parse;
pub use tokenizer::Token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let doc = parse("<p>Hello</p>");
        let output = doc.debug_tree();
        assert!(output.contains("<p>"));
        assert!(output.contains("\"Hello\""));
    }

    #[test]
    fn test_parse_nested() {
        let doc = parse("<div><p>Text</p></div>");
        let output = doc.debug_tree();
        assert!(output.contains("<div>"));
        assert!(output.contains("<p>"));
    }

    #[test]
    fn test_parse_with_doctype() {
        let doc = parse("<!DOCTYPE html><html><body></body></html>");
        let output = doc.debug_tree();
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("<html>"));
        assert!(output.contains("<body>"));
    }

    #[test]
    fn test_parse_with_attributes() {
        let doc = parse("<div class=\"container\" id=\"main\">Content</div>");
        let output = doc.debug_tree();
        assert!(output.contains("class=\"container\""));
        assert!(output.contains("id=\"main\""));
    }

    #[test]
    fn test_parse_void_elements() {
        let doc = parse("<p>Hello<br>World</p>");
        let output = doc.debug_tree();
        assert!(output.contains("<br>"));
    }

    #[test]
    fn test_parse_comment() {
        let doc = parse("<!-- comment --><p>text</p>");
        let output = doc.debug_tree();
        assert!(output.contains("<!-- comment -->"));
    }
}
