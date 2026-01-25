//! Debug output for DOM trees.
//!
//! This module provides human-readable text output for DOM trees,
//! implementing the "text debug mode" viewer required by the M1 milestone.

use crate::{Document, NodeData, NodeId};
use std::fmt::Write;

impl Document {
    /// Returns a pretty-printed string representation of the DOM tree.
    ///
    /// The output format is designed to be human-readable, with proper
    /// indentation showing the tree structure.
    ///
    /// # Example Output
    ///
    /// ```text
    /// #document
    ///   <!DOCTYPE html>
    ///   <html>
    ///     <head>
    ///       <title>
    ///         "Test"
    ///     <body class="main">
    ///       <p>
    ///         "Hello, world!"
    /// ```
    pub fn debug_tree(&self) -> String {
        let mut output = String::new();
        self.write_node(&mut output, self.root(), 0);
        output
    }

    /// Writes a node and its children to the output string with proper indentation.
    fn write_node(&self, output: &mut String, id: NodeId, depth: usize) {
        let indent = "  ".repeat(depth);

        if let Some(node) = self.get(id) {
            match &node.data {
                NodeData::Document => {
                    writeln!(output, "{}#document", indent).unwrap();
                }
                NodeData::Doctype { name } => {
                    writeln!(output, "{}<!DOCTYPE {}>", indent, name).unwrap();
                }
                NodeData::Element(el) => {
                    if el.attributes.is_empty() {
                        writeln!(output, "{}<{}>", indent, el.tag_name).unwrap();
                    } else {
                        // Sort attributes for deterministic output
                        let mut attrs: Vec<_> = el.attributes.iter().collect();
                        attrs.sort_by(|a, b| a.0.cmp(&b.0));
                        let attr_str: Vec<_> = attrs
                            .iter()
                            .map(|(k, v)| format!("{}=\"{}\"", k, escape_attr_value(v)))
                            .collect();
                        writeln!(output, "{}<{} {}>", indent, el.tag_name, attr_str.join(" "))
                            .unwrap();
                    }
                }
                NodeData::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        // Escape special characters and limit line length
                        let escaped = escape_text(trimmed);
                        writeln!(output, "{}\"{}\"", indent, escaped).unwrap();
                    }
                }
                NodeData::Comment(text) => {
                    writeln!(output, "{}<!-- {} -->", indent, text.trim()).unwrap();
                }
            }

            // Recurse into children
            for child_id in self.children(id) {
                self.write_node(output, child_id, depth + 1);
            }
        }
    }

    /// Returns a compact single-line representation of the DOM tree.
    ///
    /// Useful for logging and assertions.
    pub fn debug_compact(&self) -> String {
        let mut output = String::new();
        self.write_node_compact(&mut output, self.root());
        output
    }

    fn write_node_compact(&self, output: &mut String, id: NodeId) {
        if let Some(node) = self.get(id) {
            match &node.data {
                NodeData::Document => {
                    output.push_str("#doc");
                }
                NodeData::Doctype { name } => {
                    write!(output, "<!DOCTYPE {}>", name).unwrap();
                }
                NodeData::Element(el) => {
                    write!(output, "<{}", el.tag_name).unwrap();
                    if !el.attributes.is_empty() {
                        let mut attrs: Vec<_> = el.attributes.iter().collect();
                        attrs.sort_by(|a, b| a.0.cmp(&b.0));
                        for (k, v) in attrs {
                            write!(output, " {}=\"{}\"", k, escape_attr_value(v)).unwrap();
                        }
                    }
                    output.push('>');
                }
                NodeData::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        write!(output, "\"{}\"", escape_text(trimmed)).unwrap();
                    }
                }
                NodeData::Comment(text) => {
                    write!(output, "<!--{}-->", text.trim()).unwrap();
                }
            }

            let children: Vec<_> = self.children(id).collect();
            if !children.is_empty() {
                output.push('[');
                for (i, child_id) in children.iter().enumerate() {
                    if i > 0 {
                        output.push(',');
                    }
                    self.write_node_compact(output, *child_id);
                }
                output.push(']');
            }
        }
    }
}

/// Escapes special characters in text content for display.
fn escape_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            _ => result.push(c),
        }
    }
    result
}

/// Escapes special characters in attribute values for display.
fn escape_attr_value(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '"' => result.push_str("&quot;"),
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    #[test]
    fn test_empty_document() {
        let doc = Document::new();
        let output = doc.debug_tree();
        assert!(output.contains("#document"));
    }

    #[test]
    fn test_element_with_attributes() {
        let mut doc = Document::new();
        let attrs = vec![
            ("class".to_string(), "container".to_string()),
            ("id".to_string(), "main".to_string()),
        ];
        let elem_id = doc.create_element_with_attributes("div", attrs);
        doc.append_child(doc.root(), elem_id);

        let output = doc.debug_tree();
        assert!(output.contains("<div"));
        assert!(output.contains("class=\"container\""));
        assert!(output.contains("id=\"main\""));
    }

    #[test]
    fn test_nested_structure() {
        let mut doc = Document::new();
        let html_id = doc.create_element("html");
        let head_id = doc.create_element("head");
        let title_id = doc.create_element("title");
        let title_text = doc.create_text("Test Page");
        let body_id = doc.create_element("body");
        let p_id = doc.create_element("p");
        let p_text = doc.create_text("Hello!");

        doc.append_child(doc.root(), html_id);
        doc.append_child(html_id, head_id);
        doc.append_child(head_id, title_id);
        doc.append_child(title_id, title_text);
        doc.append_child(html_id, body_id);
        doc.append_child(body_id, p_id);
        doc.append_child(p_id, p_text);

        let output = doc.debug_tree();

        // Check proper nesting via indentation
        let lines: Vec<&str> = output.lines().collect();

        // Find the relevant lines
        let html_line = lines.iter().find(|l| l.contains("<html>")).unwrap();
        let head_line = lines.iter().find(|l| l.contains("<head>")).unwrap();
        let body_line = lines.iter().find(|l| l.contains("<body>")).unwrap();

        // html should be less indented than head and body
        assert!(html_line.len() - html_line.trim_start().len() <
                head_line.len() - head_line.trim_start().len());
        assert!(html_line.len() - html_line.trim_start().len() <
                body_line.len() - body_line.trim_start().len());
    }

    #[test]
    fn test_doctype() {
        let mut doc = Document::new();
        let doctype_id = doc.create_doctype("html");
        let html_id = doc.create_element("html");

        doc.append_child(doc.root(), doctype_id);
        doc.append_child(doc.root(), html_id);

        let output = doc.debug_tree();
        assert!(output.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_comment() {
        let mut doc = Document::new();
        let comment_id = doc.create_comment("This is a comment");
        doc.append_child(doc.root(), comment_id);

        let output = doc.debug_tree();
        assert!(output.contains("<!-- This is a comment -->"));
    }

    #[test]
    fn test_compact_output() {
        let mut doc = Document::new();
        let div_id = doc.create_element("div");
        let text_id = doc.create_text("Hello");
        doc.append_child(doc.root(), div_id);
        doc.append_child(div_id, text_id);

        let compact = doc.debug_compact();
        assert!(compact.contains("#doc"));
        assert!(compact.contains("<div>"));
        assert!(compact.contains("\"Hello\""));
        assert!(!compact.contains('\n'));
    }

    #[test]
    fn test_text_escaping() {
        assert_eq!(escape_text("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_text("tab\there"), "tab\\there");
        assert_eq!(escape_text("quote\"here"), "quote\\\"here");
    }

    #[test]
    fn test_attr_escaping() {
        assert_eq!(escape_attr_value("a&b"), "a&amp;b");
        assert_eq!(escape_attr_value("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_attr_value("<tag>"), "&lt;tag&gt;");
    }
}
