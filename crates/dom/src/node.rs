//! DOM node types and data structures.

use crate::NodeId;
use std::collections::HashMap;

/// A node in the DOM tree.
#[derive(Debug)]
pub struct Node {
    /// The type-specific data for this node
    pub data: NodeData,
    /// Parent node ID, if any
    pub parent: Option<NodeId>,
    /// Child node IDs
    pub children: Vec<NodeId>,
}

impl Node {
    /// Returns true if this node is an element.
    pub fn is_element(&self) -> bool {
        matches!(self.data, NodeData::Element(_))
    }

    /// Returns true if this node is a text node.
    pub fn is_text(&self) -> bool {
        matches!(self.data, NodeData::Text(_))
    }

    /// Returns the element data if this is an element node.
    pub fn as_element(&self) -> Option<&ElementData> {
        match &self.data {
            NodeData::Element(data) => Some(data),
            _ => None,
        }
    }

    /// Returns the text content if this is a text node.
    pub fn as_text(&self) -> Option<&str> {
        match &self.data {
            NodeData::Text(text) => Some(text),
            _ => None,
        }
    }
}

/// The data associated with different node types.
#[derive(Debug)]
pub enum NodeData {
    /// The document root node
    Document,
    /// A DOCTYPE declaration (e.g., <!DOCTYPE html>)
    DocType(String),
    /// An element node (e.g., <div>, <p>)
    Element(ElementData),
    /// A text node
    Text(String),
    /// A comment node (e.g., <!-- comment -->)
    Comment(String),
}

/// Data specific to element nodes.
#[derive(Debug)]
pub struct ElementData {
    /// The tag name (lowercase)
    pub tag_name: String,
    /// Element attributes
    pub attributes: HashMap<String, String>,
}

impl ElementData {
    /// Creates a new ElementData with the given tag name.
    pub fn new(tag_name: &str) -> Self {
        ElementData {
            tag_name: tag_name.to_lowercase(),
            attributes: HashMap::new(),
        }
    }

    /// Gets an attribute value by name.
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|s| s.as_str())
    }

    /// Sets an attribute value.
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    /// Returns true if this element has a specific class.
    pub fn has_class(&self, class: &str) -> bool {
        self.attributes
            .get("class")
            .map(|classes| classes.split_whitespace().any(|c| c == class))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_data() {
        let mut elem = ElementData::new("DIV");
        assert_eq!(elem.tag_name, "div"); // Should be lowercase

        elem.set_attribute("class", "foo bar");
        assert_eq!(elem.get_attribute("class"), Some("foo bar"));
        assert!(elem.has_class("foo"));
        assert!(elem.has_class("bar"));
        assert!(!elem.has_class("baz"));
    }

    #[test]
    fn test_node_type_checks() {
        let elem_node = Node {
            data: NodeData::Element(ElementData::new("p")),
            parent: None,
            children: Vec::new(),
        };
        assert!(elem_node.is_element());
        assert!(!elem_node.is_text());

        let text_node = Node {
            data: NodeData::Text("Hello".to_string()),
            parent: None,
            children: Vec::new(),
        };
        assert!(!text_node.is_element());
        assert!(text_node.is_text());
        assert_eq!(text_node.as_text(), Some("Hello"));
    }
}
