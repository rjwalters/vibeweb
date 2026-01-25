//! HTML tree builder
//!
//! Constructs a DOM tree from HTML tokens. This implements a simplified
//! version of the HTML tree construction algorithm, handling:
//!
//! - Proper parent-child relationships
//! - Implied tags (html, head, body)
//! - Self-closing elements (br, hr, img, input, meta, etc.)
//! - Basic error recovery for unclosed tags
//!
//! The implementation focuses on correctly parsing well-formed HTML and
//! providing reasonable behavior for common malformed patterns.

use crate::tokenizer::Token;
use vw_dom::{Document, NodeId};

/// Set of void elements that don't have closing tags.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

// Note: FORMATTING_ELEMENTS would be used for adoption agency algorithm
// which is not implemented in this simplified parser.
// const FORMATTING_ELEMENTS: &[&str] = &[
//     "a", "b", "big", "code", "em", "font", "i", "nobr",
//     "s", "small", "strike", "strong", "tt", "u",
// ];

/// HTML tree builder that constructs a DOM from tokens.
pub struct TreeBuilder {
    /// The document being built.
    document: Document,
    /// Stack of open elements (for tree construction).
    open_elements: Vec<NodeId>,
    /// Whether we've seen the html element.
    seen_html: bool,
    /// Whether we've seen the head element.
    seen_head: bool,
    /// Whether we've seen the body element.
    seen_body: bool,
    /// The head element, if created.
    head_element: Option<NodeId>,
    /// The body element, if created.
    body_element: Option<NodeId>,
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeBuilder {
    /// Creates a new tree builder.
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            open_elements: Vec::new(),
            seen_html: false,
            seen_head: false,
            seen_body: false,
            head_element: None,
            body_element: None,
        }
    }

    /// Processes a single token.
    pub fn process(&mut self, token: Token) {
        match token {
            Token::Doctype { name } => self.process_doctype(name),
            Token::StartTag {
                name,
                attributes,
                self_closing,
            } => {
                self.process_start_tag(&name, attributes, self_closing);
            }
            Token::EndTag { name } => self.process_end_tag(&name),
            Token::Text(text) => self.process_text(&text),
            Token::Comment(text) => self.process_comment(&text),
        }
    }

    /// Finishes building and returns the document.
    pub fn finish(self) -> Document {
        self.document
    }

    /// Returns the current insertion point (last open element or document root).
    fn current_node(&self) -> NodeId {
        self.open_elements
            .last()
            .copied()
            .unwrap_or(self.document.root())
    }

    // Note: is_in_scope would be used for more sophisticated tree construction
    // which is not implemented in this simplified parser.
    // fn is_in_scope(&self, tag_name: &str) -> bool { ... }

    /// Ensures the html element exists.
    fn ensure_html(&mut self) {
        if !self.seen_html {
            let html = self.document.create_element("html");
            self.document.append_child(self.document.root(), html);
            self.open_elements.push(html);
            self.seen_html = true;
        }
    }

    /// Ensures the head element exists.
    fn ensure_head(&mut self) {
        self.ensure_html();
        if !self.seen_head {
            let html = self.open_elements.first().copied().unwrap();
            let head = self.document.create_element("head");
            self.document.append_child(html, head);
            self.head_element = Some(head);
            self.seen_head = true;
        }
    }

    /// Ensures the body element exists.
    fn ensure_body(&mut self) {
        self.ensure_head();
        if !self.seen_body {
            let html = self.open_elements.first().copied().unwrap();
            let body = self.document.create_element("body");
            self.document.append_child(html, body);
            self.open_elements.push(body);
            self.body_element = Some(body);
            self.seen_body = true;
        }
    }

    /// Processes a DOCTYPE token.
    fn process_doctype(&mut self, name: Option<String>) {
        let doctype = self.document.create_doctype(&name.unwrap_or_default());
        self.document.append_child(self.document.root(), doctype);
    }

    /// Processes a start tag.
    fn process_start_tag(
        &mut self,
        name: &str,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    ) {
        match name {
            "html" => {
                if !self.seen_html {
                    let html = self
                        .document
                        .create_element_with_attributes("html", attributes);
                    self.document.append_child(self.document.root(), html);
                    self.open_elements.push(html);
                    self.seen_html = true;
                }
            }
            "head" => {
                self.ensure_html();
                if !self.seen_head {
                    let html = self.open_elements.first().copied().unwrap();
                    let head = self
                        .document
                        .create_element_with_attributes("head", attributes);
                    self.document.append_child(html, head);
                    self.open_elements.push(head);
                    self.head_element = Some(head);
                    self.seen_head = true;
                }
            }
            "body" => {
                self.ensure_head();
                // Pop head from open elements if it's there
                if let Some(&last) = self.open_elements.last() {
                    if let Some(node) = self.document.get(last) {
                        if node
                            .as_element()
                            .map(|e| e.tag_name == "head")
                            .unwrap_or(false)
                        {
                            self.open_elements.pop();
                        }
                    }
                }
                if !self.seen_body {
                    let html = self.open_elements.first().copied().unwrap();
                    let body = self
                        .document
                        .create_element_with_attributes("body", attributes);
                    self.document.append_child(html, body);
                    self.open_elements.push(body);
                    self.body_element = Some(body);
                    self.seen_body = true;
                }
            }
            // Elements that go in head
            "title" | "base" | "link" | "meta" | "style" | "script" if !self.seen_body => {
                self.ensure_head();
                let parent = self.head_element.unwrap_or(self.current_node());
                let elem = self
                    .document
                    .create_element_with_attributes(name, attributes);
                self.document.append_child(parent, elem);
                if !self_closing && !is_void_element(name) {
                    self.open_elements.push(elem);
                }
            }
            // Void elements
            _ if is_void_element(name) || self_closing => {
                self.ensure_body();
                let elem = self
                    .document
                    .create_element_with_attributes(name, attributes);
                self.document.append_child(self.current_node(), elem);
            }
            // Regular elements
            _ => {
                self.ensure_body();
                let elem = self
                    .document
                    .create_element_with_attributes(name, attributes);
                self.document.append_child(self.current_node(), elem);
                self.open_elements.push(elem);
            }
        }
    }

    /// Processes an end tag.
    fn process_end_tag(&mut self, name: &str) {
        // Void elements don't have end tags
        if is_void_element(name) {
            return;
        }

        // Special handling for html, head, body
        match name {
            "html" => {
                self.open_elements.clear();
                return;
            }
            "head" => {
                if let Some(pos) = self.find_in_stack("head") {
                    self.open_elements.truncate(pos);
                }
                return;
            }
            "body" => {
                if let Some(pos) = self.find_in_stack("body") {
                    self.open_elements.truncate(pos);
                }
                return;
            }
            _ => {}
        }

        // Find the matching open element and pop to it
        if let Some(pos) = self.find_in_stack(name) {
            self.open_elements.truncate(pos);
        }
    }

    /// Finds the position of an element with the given tag name in the stack.
    fn find_in_stack(&self, tag_name: &str) -> Option<usize> {
        for (i, &node_id) in self.open_elements.iter().enumerate().rev() {
            if let Some(node) = self.document.get(node_id) {
                if let Some(elem) = node.as_element() {
                    if elem.tag_name == tag_name {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    /// Checks if the current node is in the head section.
    fn is_in_head(&self) -> bool {
        if let Some(&current) = self.open_elements.last() {
            // Check if current node or any ancestor is head
            if let Some(head) = self.head_element {
                if current == head {
                    return true;
                }
            }
            // Check if parent is head
            for &node_id in self.open_elements.iter().rev() {
                if let Some(node) = self.document.get(node_id) {
                    if let Some(elem) = node.as_element() {
                        if elem.tag_name == "head" {
                            return true;
                        }
                        if elem.tag_name == "body" {
                            return false;
                        }
                    }
                }
            }
        }
        false
    }

    /// Processes text content.
    fn process_text(&mut self, text: &str) {
        // Don't add empty text nodes
        if text.is_empty() {
            return;
        }

        // If we're currently inside head elements, add text there
        if self.is_in_head() {
            let text_node = self.document.create_text(text);
            self.document.append_child(self.current_node(), text_node);
            return;
        }

        // Skip whitespace-only text before body
        if !self.seen_body && text.chars().all(|c| c.is_ascii_whitespace()) {
            return;
        }

        self.ensure_body();

        let text_node = self.document.create_text(text);
        self.document.append_child(self.current_node(), text_node);
    }

    /// Processes a comment.
    fn process_comment(&mut self, text: &str) {
        let comment = self.document.create_comment(text);
        let parent = if self.open_elements.is_empty() {
            self.document.root()
        } else {
            self.current_node()
        };
        self.document.append_child(parent, comment);
    }
}

/// Checks if the given tag name is a void element.
fn is_void_element(name: &str) -> bool {
    VOID_ELEMENTS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tokenizer;

    fn parse(html: &str) -> Document {
        let mut builder = TreeBuilder::new();
        for token in Tokenizer::new(html) {
            builder.process(token);
        }
        builder.finish()
    }

    #[test]
    fn test_simple_document() {
        let doc = parse("<html><body><p>Hello</p></body></html>");

        // Should have html, body, p
        let html = doc.document_element().unwrap();
        assert_eq!(
            doc.get(html).unwrap().as_element().unwrap().tag_name,
            "html"
        );

        let body = doc.get_element_by_tag_name("body").unwrap();
        assert!(doc.get(body).is_some());

        let p = doc.get_element_by_tag_name("p").unwrap();
        assert_eq!(doc.text_content(p), "Hello");
    }

    #[test]
    fn test_implied_html_body() {
        let doc = parse("<p>Hello</p>");

        // Should have implied html and body
        let html = doc.document_element().unwrap();
        assert_eq!(
            doc.get(html).unwrap().as_element().unwrap().tag_name,
            "html"
        );

        let body = doc.get_element_by_tag_name("body").unwrap();
        assert!(doc.get(body).is_some());

        let p = doc.get_element_by_tag_name("p").unwrap();
        assert_eq!(doc.text_content(p), "Hello");
    }

    #[test]
    fn test_doctype() {
        let doc = parse("<!DOCTYPE html><html><body></body></html>");

        let mut found_doctype = false;
        for child_id in doc.children(doc.root()) {
            if let Some(node) = doc.get(child_id) {
                if matches!(node.data, vw_dom::NodeData::Doctype { .. }) {
                    found_doctype = true;
                    break;
                }
            }
        }
        assert!(found_doctype);
    }

    #[test]
    fn test_void_elements() {
        let doc = parse("<p>Line 1<br>Line 2</p>");

        let br = doc.get_element_by_tag_name("br").unwrap();
        assert!(doc.get(br).is_some());

        let p = doc.get_element_by_tag_name("p").unwrap();
        // br should be a child of p
        let children: Vec<_> = doc.children(p).collect();
        assert!(children.contains(&br));
    }

    #[test]
    fn test_nested_elements() {
        let doc = parse("<div><p><span>Hello</span></p></div>");

        let div = doc.get_element_by_tag_name("div").unwrap();
        let p = doc.get_element_by_tag_name("p").unwrap();
        let span = doc.get_element_by_tag_name("span").unwrap();

        // Check parent relationships
        assert_eq!(doc.get(p).unwrap().parent, Some(div));
        assert_eq!(doc.get(span).unwrap().parent, Some(p));
        assert_eq!(doc.text_content(span), "Hello");
    }

    #[test]
    fn test_attributes_preserved() {
        let doc = parse(r#"<div class="container" id="main">Content</div>"#);

        let div = doc.get_element_by_tag_name("div").unwrap();
        let elem = doc.get(div).unwrap().as_element().unwrap();
        assert_eq!(elem.get_attribute("class"), Some("container"));
        assert_eq!(elem.get_attribute("id"), Some("main"));
    }

    #[test]
    fn test_unclosed_tags() {
        // Parser should handle unclosed tags gracefully
        let doc = parse("<div><p>Hello");

        let p = doc.get_element_by_tag_name("p").unwrap();
        assert_eq!(doc.text_content(p), "Hello");
    }

    #[test]
    fn test_head_elements() {
        let doc = parse("<html><head><title>Test</title></head><body></body></html>");

        let title = doc.get_element_by_tag_name("title").unwrap();
        let head = doc.get_element_by_tag_name("head").unwrap();

        // title should be in head
        assert_eq!(doc.get(title).unwrap().parent, Some(head));
    }

    #[test]
    fn test_comment() {
        let doc = parse("<!-- Hello --><p>World</p>");

        // Should have a comment node
        let mut found_comment = false;
        for id in doc.descendants(doc.root()) {
            if let Some(node) = doc.get(id) {
                if matches!(node.data, vw_dom::NodeData::Comment(_)) {
                    found_comment = true;
                    break;
                }
            }
        }
        assert!(found_comment);
    }

    #[test]
    fn test_self_closing_syntax() {
        let doc = parse("<img src=\"test.png\"/>");

        let img = doc.get_element_by_tag_name("img").unwrap();
        let elem = doc.get(img).unwrap().as_element().unwrap();
        assert_eq!(elem.get_attribute("src"), Some("test.png"));
    }
}
