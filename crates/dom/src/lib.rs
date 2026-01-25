//! DOM types, traversal, and mutation APIs
//!
//! This crate provides the core DOM tree representation used by vibeweb.
//! It implements a simple arena-based document model with support for
//! elements, text nodes, comments, and document types.

mod debug;
mod node;

// Note: debug module adds methods to Document via impl blocks, not standalone exports
pub use node::*;

use std::collections::HashMap;

/// A unique identifier for a node within a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    /// Creates a new NodeId from a raw index.
    pub fn new(index: usize) -> Self {
        NodeId(index)
    }

    /// Returns the raw index of this NodeId.
    pub fn index(&self) -> usize {
        self.0
    }
}

/// The DOM document, containing all nodes in an arena.
#[derive(Debug)]
pub struct Document {
    /// Arena storage for all nodes
    nodes: Vec<Node>,
    /// The root node ID (always 0 for a valid document)
    root: NodeId,
}

impl Document {
    /// Creates a new empty document with a document root node.
    pub fn new() -> Self {
        let mut doc = Document {
            nodes: Vec::new(),
            root: NodeId(0),
        };
        // Create the document root node
        doc.nodes.push(Node {
            data: NodeData::Document,
            parent: None,
            children: Vec::new(),
        });
        doc
    }

    /// Returns the root node ID.
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Gets a reference to a node by ID.
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    /// Gets a mutable reference to a node by ID.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.0)
    }

    /// Creates a new element node and returns its ID.
    pub fn create_element(&mut self, tag_name: &str) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            data: NodeData::Element(ElementData {
                tag_name: tag_name.to_lowercase(),
                attributes: HashMap::new(),
            }),
            parent: None,
            children: Vec::new(),
        });
        id
    }

    /// Creates a new element node with attributes and returns its ID.
    pub fn create_element_with_attrs(
        &mut self,
        tag_name: &str,
        attributes: HashMap<String, String>,
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            data: NodeData::Element(ElementData {
                tag_name: tag_name.to_lowercase(),
                attributes,
            }),
            parent: None,
            children: Vec::new(),
        });
        id
    }

    /// Creates a new text node and returns its ID.
    pub fn create_text(&mut self, content: &str) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            data: NodeData::Text(content.to_string()),
            parent: None,
            children: Vec::new(),
        });
        id
    }

    /// Creates a new comment node and returns its ID.
    pub fn create_comment(&mut self, content: &str) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            data: NodeData::Comment(content.to_string()),
            parent: None,
            children: Vec::new(),
        });
        id
    }

    /// Creates a DOCTYPE node and returns its ID.
    pub fn create_doctype(&mut self, name: &str) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            data: NodeData::DocType(name.to_string()),
            parent: None,
            children: Vec::new(),
        });
        id
    }

    /// Appends a child node to a parent node.
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        // Set the child's parent
        if let Some(child) = self.nodes.get_mut(child_id.0) {
            child.parent = Some(parent_id);
        }
        // Add child to parent's children list
        if let Some(parent) = self.nodes.get_mut(parent_id.0) {
            parent.children.push(child_id);
        }
    }

    /// Returns an iterator over the children of a node.
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.get(id)
            .map(|n| n.children.iter().copied())
            .into_iter()
            .flatten()
    }

    /// Returns the number of nodes in the document.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the document contains only the root node.
    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document() {
        let doc = Document::new();
        assert_eq!(doc.len(), 1);
        // is_empty() is true when only the root node exists (no content)
        assert!(doc.is_empty());

        let root = doc.get(doc.root()).unwrap();
        assert!(matches!(root.data, NodeData::Document));
    }

    #[test]
    fn test_create_element() {
        let mut doc = Document::new();
        let elem_id = doc.create_element("div");

        let elem = doc.get(elem_id).unwrap();
        if let NodeData::Element(data) = &elem.data {
            assert_eq!(data.tag_name, "div");
        } else {
            panic!("Expected element node");
        }
    }

    #[test]
    fn test_append_child() {
        let mut doc = Document::new();
        let html_id = doc.create_element("html");
        let body_id = doc.create_element("body");

        doc.append_child(doc.root(), html_id);
        doc.append_child(html_id, body_id);

        // Check parent-child relationships
        let html = doc.get(html_id).unwrap();
        assert_eq!(html.parent, Some(doc.root()));
        assert_eq!(html.children.len(), 1);

        let body = doc.get(body_id).unwrap();
        assert_eq!(body.parent, Some(html_id));
    }

    #[test]
    fn test_debug_tree_simple() {
        let mut doc = Document::new();
        let html_id = doc.create_element("html");
        let body_id = doc.create_element("body");
        let text_id = doc.create_text("Hello, world!");

        doc.append_child(doc.root(), html_id);
        doc.append_child(html_id, body_id);
        doc.append_child(body_id, text_id);

        let output = doc.debug_tree();
        assert!(output.contains("#document"));
        assert!(output.contains("<html>"));
        assert!(output.contains("<body>"));
        assert!(output.contains("\"Hello, world!\""));
    }
}
