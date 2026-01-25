//! HTML parser that builds a DOM tree from tokens.
//!
//! This is a simplified tree builder that handles common HTML patterns.
//! It's not a full HTML5 spec-compliant parser, but handles well-formed
//! HTML and gracefully recovers from common malformed patterns.

use crate::tokenizer::{Token, Tokenizer};
use vw_dom::{Document, NodeId};

/// Parses an HTML string and returns a DOM Document.
///
/// # Example
///
/// ```
/// use vw_html::parse;
///
/// let doc = parse("<p>Hello, <b>world</b>!</p>");
/// println!("{}", doc.debug_tree());
/// ```
pub fn parse(html: &str) -> Document {
    let parser = Parser::new(html);
    parser.parse()
}

/// HTML parser state.
struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    document: Document,
    /// Stack of open elements (for tree construction)
    open_elements: Vec<NodeId>,
}

/// List of void elements that don't have closing tags.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// List of elements that auto-close certain other elements.
const AUTO_CLOSE_RULES: &[(&str, &[&str])] = &[
    // <p> auto-closes another <p>
    ("p", &["p"]),
    // <li> auto-closes another <li>
    ("li", &["li"]),
    // <dt> and <dd> auto-close each other
    ("dt", &["dt", "dd"]),
    ("dd", &["dt", "dd"]),
    // Table row/cell elements
    ("tr", &["tr", "th", "td"]),
    ("th", &["th", "td"]),
    ("td", &["th", "td"]),
    // Option elements
    ("option", &["option"]),
];

impl<'a> Parser<'a> {
    /// Creates a new parser for the given HTML input.
    fn new(html: &'a str) -> Self {
        let document = Document::new();
        let root = document.root();
        Parser {
            tokenizer: Tokenizer::new(html),
            document,
            open_elements: vec![root],
        }
    }

    /// Parses the HTML and returns the completed document.
    fn parse(mut self) -> Document {
        loop {
            let token = self.tokenizer.next_token();
            match token {
                Token::Eof => break,
                Token::DocType(name) => self.handle_doctype(&name),
                Token::StartTag {
                    name,
                    attributes,
                    self_closing,
                } => self.handle_start_tag(&name, attributes, self_closing),
                Token::EndTag(name) => self.handle_end_tag(&name),
                Token::Text(text) => self.handle_text(&text),
                Token::Comment(text) => self.handle_comment(&text),
            }
        }

        self.document
    }

    /// Returns the current parent node ID.
    fn current_node(&self) -> NodeId {
        *self.open_elements.last().unwrap_or(&self.document.root())
    }

    /// Handles a DOCTYPE token.
    fn handle_doctype(&mut self, name: &str) {
        let doctype_id = self.document.create_doctype(name);
        self.document.append_child(self.document.root(), doctype_id);
    }

    /// Handles a start tag token.
    fn handle_start_tag(
        &mut self,
        name: &str,
        attributes: std::collections::HashMap<String, String>,
        self_closing: bool,
    ) {
        // Handle auto-closing rules
        self.apply_auto_close_rules(name);

        // Create the element
        let elem_id = self.document.create_element_with_attrs(name, attributes);
        let parent = self.current_node();
        self.document.append_child(parent, elem_id);

        // Check if this is a void element or self-closing
        let is_void = VOID_ELEMENTS.contains(&name);
        if !is_void && !self_closing {
            self.open_elements.push(elem_id);
        }
    }

    /// Applies auto-close rules for the given tag.
    fn apply_auto_close_rules(&mut self, new_tag: &str) {
        for (tag, closes) in AUTO_CLOSE_RULES {
            if *tag == new_tag {
                // Check if any of the elements to close are on the stack
                loop {
                    if let Some(top_id) = self.open_elements.last() {
                        if let Some(node) = self.document.get(*top_id) {
                            if let Some(elem) = node.as_element() {
                                if closes.contains(&elem.tag_name.as_str()) {
                                    self.open_elements.pop();
                                    continue;
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    /// Handles an end tag token.
    fn handle_end_tag(&mut self, name: &str) {
        // Find matching open element and pop back to it
        let mut found_index = None;
        for (i, node_id) in self.open_elements.iter().enumerate().rev() {
            if let Some(node) = self.document.get(*node_id) {
                if let Some(elem) = node.as_element() {
                    if elem.tag_name == name {
                        found_index = Some(i);
                        break;
                    }
                }
            }
        }

        if let Some(index) = found_index {
            // Pop all elements back to and including the matching one
            self.open_elements.truncate(index);
        }
        // If no matching element found, ignore the end tag (error recovery)
    }

    /// Handles text content.
    fn handle_text(&mut self, text: &str) {
        // Skip whitespace-only text nodes at certain positions
        if text.trim().is_empty() {
            return;
        }

        let text_id = self.document.create_text(text);
        let parent = self.current_node();
        self.document.append_child(parent, text_id);
    }

    /// Handles a comment.
    fn handle_comment(&mut self, text: &str) {
        let comment_id = self.document.create_comment(text);
        let parent = self.current_node();
        self.document.append_child(parent, comment_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let doc = parse("");
        assert_eq!(doc.len(), 1); // Just the document root
    }

    #[test]
    fn test_parse_simple_element() {
        let doc = parse("<div>Hello</div>");
        let output = doc.debug_tree();
        assert!(output.contains("<div>"));
        assert!(output.contains("\"Hello\""));
    }

    #[test]
    fn test_parse_nested_elements() {
        let doc = parse("<div><p>Text</p></div>");
        let output = doc.debug_tree();

        // Check that p is nested inside div
        let lines: Vec<&str> = output.lines().collect();
        let div_indent = lines
            .iter()
            .find(|l| l.contains("<div>"))
            .map(|l| l.len() - l.trim_start().len())
            .unwrap();
        let p_indent = lines
            .iter()
            .find(|l| l.contains("<p>"))
            .map(|l| l.len() - l.trim_start().len())
            .unwrap();
        assert!(p_indent > div_indent);
    }

    #[test]
    fn test_parse_void_element() {
        let doc = parse("<p>Line1<br>Line2</p>");
        let output = doc.debug_tree();
        assert!(output.contains("<br>"));
        // br should not cause Line2 to be outside p
        assert!(output.contains("\"Line2\""));
    }

    #[test]
    fn test_parse_self_closing() {
        let doc = parse("<div><img src=\"test.png\"/></div>");
        let output = doc.debug_tree();
        assert!(output.contains("<img"));
        assert!(output.contains("src=\"test.png\""));
    }

    #[test]
    fn test_auto_close_p() {
        let doc = parse("<p>Para 1<p>Para 2</p>");
        // Both paragraphs should be siblings, not nested
        let output = doc.debug_tree();
        let lines: Vec<&str> = output.lines().collect();

        let p_lines: Vec<_> = lines.iter().filter(|l| l.contains("<p>")).collect();
        assert_eq!(p_lines.len(), 2);

        // Both <p> tags should have the same indentation
        let indent1 = p_lines[0].len() - p_lines[0].trim_start().len();
        let indent2 = p_lines[1].len() - p_lines[1].trim_start().len();
        assert_eq!(indent1, indent2);
    }

    #[test]
    fn test_mismatched_tags() {
        // Parser should handle mismatched tags gracefully
        let doc = parse("<div><p>Text</div>");
        let output = doc.debug_tree();
        // Should still produce valid output
        assert!(output.contains("<div>"));
        assert!(output.contains("<p>"));
        assert!(output.contains("\"Text\""));
    }
}
