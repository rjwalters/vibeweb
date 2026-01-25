//! DOM abstraction for layout.
//!
//! This module provides a minimal DOM interface that layout needs.
//! It defines traits that can be implemented by the actual DOM in `vw-dom`.

use crate::box_types::NodeId;

/// Type of DOM node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Document root
    Document,
    /// Element node (e.g., <div>, <p>)
    Element,
    /// Text node
    Text,
    /// Comment node
    Comment,
}

/// Information about a DOM node needed for layout.
#[derive(Debug, Clone)]
pub struct NodeData {
    /// Unique identifier for this node
    pub id: NodeId,
    /// Type of node
    pub node_type: NodeType,
    /// Tag name for element nodes (e.g., "div", "p")
    pub tag_name: Option<String>,
    /// Text content for text nodes
    pub text_content: Option<String>,
    /// Child node IDs
    pub children: Vec<NodeId>,
}

impl NodeData {
    /// Create a new element node.
    pub fn element(id: NodeId, tag_name: &str) -> Self {
        NodeData {
            id,
            node_type: NodeType::Element,
            tag_name: Some(tag_name.to_string()),
            text_content: None,
            children: Vec::new(),
        }
    }

    /// Create a new text node.
    pub fn text(id: NodeId, content: &str) -> Self {
        NodeData {
            id,
            node_type: NodeType::Text,
            tag_name: None,
            text_content: Some(content.to_string()),
            children: Vec::new(),
        }
    }

    /// Create a document node.
    pub fn document(id: NodeId) -> Self {
        NodeData {
            id,
            node_type: NodeType::Document,
            tag_name: None,
            text_content: None,
            children: Vec::new(),
        }
    }

    /// Add a child node ID.
    pub fn with_child(mut self, child_id: NodeId) -> Self {
        self.children.push(child_id);
        self
    }

    /// Returns true if this is an element node.
    pub fn is_element(&self) -> bool {
        self.node_type == NodeType::Element
    }

    /// Returns true if this is a text node.
    pub fn is_text(&self) -> bool {
        self.node_type == NodeType::Text
    }
}

/// A simple document structure for testing and standalone use.
///
/// This can be replaced with the actual `vw-dom::Document` when integrated.
#[derive(Debug, Clone, Default)]
pub struct Document {
    nodes: Vec<NodeData>,
    root_id: Option<NodeId>,
}

impl Document {
    /// Create a new empty document.
    pub fn new() -> Self {
        Document {
            nodes: Vec::new(),
            root_id: None,
        }
    }

    /// Add a node to the document and return its ID.
    pub fn add_node(&mut self, mut node: NodeData) -> NodeId {
        let id = NodeId(self.nodes.len());
        node.id = id;
        self.nodes.push(node);
        id
    }

    /// Set the root node ID.
    pub fn set_root(&mut self, id: NodeId) {
        self.root_id = Some(id);
    }

    /// Get the root node ID.
    pub fn root(&self) -> Option<NodeId> {
        self.root_id
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&NodeData> {
        self.nodes.get(id.0)
    }

    /// Get a mutable reference to a node.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.nodes.get_mut(id.0)
    }

    /// Iterate over all nodes.
    pub fn nodes(&self) -> impl Iterator<Item = &NodeData> {
        self.nodes.iter()
    }

    /// Get children of a node.
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = &NodeData> {
        self.get_node(id)
            .map(|node| node.children.iter())
            .into_iter()
            .flatten()
            .filter_map(move |child_id| self.get_node(*child_id))
    }
}

/// Builder for creating documents programmatically.
pub struct DocumentBuilder {
    document: Document,
    current_id: Option<NodeId>,
}

impl DocumentBuilder {
    /// Create a new document builder.
    pub fn new() -> Self {
        DocumentBuilder {
            document: Document::new(),
            current_id: None,
        }
    }

    /// Add a document root.
    pub fn document_root(mut self) -> Self {
        let id = self.document.add_node(NodeData::document(NodeId(0)));
        self.document.set_root(id);
        self.current_id = Some(id);
        self
    }

    /// Add an element as a child of the current node.
    pub fn element(mut self, tag_name: &str) -> Self {
        let id = self
            .document
            .add_node(NodeData::element(NodeId(0), tag_name));
        if let Some(parent_id) = self.current_id {
            if let Some(parent) = self.document.get_node_mut(parent_id) {
                parent.children.push(id);
            }
        } else {
            self.document.set_root(id);
        }
        self.current_id = Some(id);
        self
    }

    /// Add a text node as a child of the current node.
    pub fn text(mut self, content: &str) -> Self {
        let id = self.document.add_node(NodeData::text(NodeId(0), content));
        if let Some(parent_id) = self.current_id {
            if let Some(parent) = self.document.get_node_mut(parent_id) {
                parent.children.push(id);
            }
        }
        self.current_id = Some(id);
        self
    }

    /// Move up to the parent node.
    pub fn up(mut self) -> Self {
        // For simplicity, this is a no-op in the current implementation
        // A full implementation would track the parent chain
        self.current_id = None;
        self
    }

    /// Build the document.
    pub fn build(self) -> Document {
        self.document
    }
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_data_creation() {
        let elem = NodeData::element(NodeId(0), "div");
        assert!(elem.is_element());
        assert_eq!(elem.tag_name, Some("div".to_string()));

        let text = NodeData::text(NodeId(1), "Hello");
        assert!(text.is_text());
        assert_eq!(text.text_content, Some("Hello".to_string()));
    }

    #[test]
    fn test_document_construction() {
        let mut doc = Document::new();

        let root = doc.add_node(NodeData::element(NodeId(0), "html"));
        doc.set_root(root);

        let body = doc.add_node(NodeData::element(NodeId(0), "body"));
        let text = doc.add_node(NodeData::text(NodeId(0), "Hello, World!"));

        // Link them
        doc.get_node_mut(root).unwrap().children.push(body);
        doc.get_node_mut(body).unwrap().children.push(text);

        assert_eq!(doc.root(), Some(root));
        assert_eq!(
            doc.get_node(root).unwrap().tag_name,
            Some("html".to_string())
        );

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].tag_name, Some("body".to_string()));
    }
}
