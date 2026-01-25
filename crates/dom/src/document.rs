//! Document type and tree manipulation APIs

use crate::node::{Node, NodeData, NodeId, ElementData};

/// A DOM document containing an arena of nodes.
///
/// The Document owns all nodes and provides methods for creating,
/// traversing, and mutating the DOM tree.
#[derive(Debug)]
pub struct Document {
    /// The arena of all nodes in the document.
    nodes: Vec<Node>,
    /// The root document node ID.
    root: NodeId,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    /// Creates a new empty document with a root document node.
    pub fn new() -> Self {
        let root_node = Node::new(NodeData::Document);
        Self {
            nodes: vec![root_node],
            root: NodeId(0),
        }
    }

    /// Returns the root document node ID.
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Returns a reference to a node by its ID, if it exists.
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    /// Returns a mutable reference to a node by its ID, if it exists.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.0)
    }

    /// Creates a new element node and returns its ID.
    pub fn create_element(&mut self, tag_name: &str) -> NodeId {
        self.create_element_with_attributes(tag_name, Vec::new())
    }

    /// Creates a new element node with attributes and returns its ID.
    pub fn create_element_with_attributes(
        &mut self,
        tag_name: &str,
        attributes: Vec<(String, String)>,
    ) -> NodeId {
        let data = ElementData::with_attributes(tag_name, attributes);
        let node = Node::new(NodeData::Element(data));
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    /// Creates a new text node and returns its ID.
    pub fn create_text(&mut self, content: &str) -> NodeId {
        let node = Node::new(NodeData::Text(content.to_string()));
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    /// Creates a new comment node and returns its ID.
    pub fn create_comment(&mut self, content: &str) -> NodeId {
        let node = Node::new(NodeData::Comment(content.to_string()));
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    /// Creates a new doctype node and returns its ID.
    pub fn create_doctype(&mut self, name: &str) -> NodeId {
        let node = Node::new(NodeData::Doctype {
            name: name.to_string(),
        });
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    /// Appends a child node to a parent node.
    ///
    /// This establishes the parent-child relationship and updates sibling links.
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        // Get the current last child of parent
        let last_child = self.get(parent).and_then(|p| p.last_child);

        // Update the child's parent reference
        if let Some(child_node) = self.get_mut(child) {
            child_node.parent = Some(parent);
            child_node.prev_sibling = last_child;
            child_node.next_sibling = None;
        }

        // Update the previous last child's next_sibling
        if let Some(last) = last_child {
            if let Some(last_node) = self.get_mut(last) {
                last_node.next_sibling = Some(child);
            }
        }

        // Update the parent's first_child and last_child
        if let Some(parent_node) = self.get_mut(parent) {
            if parent_node.first_child.is_none() {
                parent_node.first_child = Some(child);
            }
            parent_node.last_child = Some(child);
        }
    }

    /// Inserts a child before a reference node.
    ///
    /// The reference node must be a child of the parent.
    pub fn insert_before(&mut self, parent: NodeId, child: NodeId, reference: NodeId) {
        let prev_sibling = self.get(reference).and_then(|r| r.prev_sibling);

        // Update the child's relationships
        if let Some(child_node) = self.get_mut(child) {
            child_node.parent = Some(parent);
            child_node.prev_sibling = prev_sibling;
            child_node.next_sibling = Some(reference);
        }

        // Update the reference node's prev_sibling
        if let Some(ref_node) = self.get_mut(reference) {
            ref_node.prev_sibling = Some(child);
        }

        // Update the previous sibling's next_sibling
        if let Some(prev) = prev_sibling {
            if let Some(prev_node) = self.get_mut(prev) {
                prev_node.next_sibling = Some(child);
            }
        } else {
            // Child becomes the first child
            if let Some(parent_node) = self.get_mut(parent) {
                parent_node.first_child = Some(child);
            }
        }
    }

    /// Removes a child from its parent.
    ///
    /// The node remains in the arena but is disconnected from the tree.
    pub fn remove_child(&mut self, child: NodeId) {
        let (parent, prev, next) = {
            let node = match self.get(child) {
                Some(n) => n,
                None => return,
            };
            (node.parent, node.prev_sibling, node.next_sibling)
        };

        // Update previous sibling
        if let Some(prev_id) = prev {
            if let Some(prev_node) = self.get_mut(prev_id) {
                prev_node.next_sibling = next;
            }
        }

        // Update next sibling
        if let Some(next_id) = next {
            if let Some(next_node) = self.get_mut(next_id) {
                next_node.prev_sibling = prev;
            }
        }

        // Update parent
        if let Some(parent_id) = parent {
            if let Some(parent_node) = self.get_mut(parent_id) {
                if parent_node.first_child == Some(child) {
                    parent_node.first_child = next;
                }
                if parent_node.last_child == Some(child) {
                    parent_node.last_child = prev;
                }
            }
        }

        // Clear the child's relationships
        if let Some(child_node) = self.get_mut(child) {
            child_node.parent = None;
            child_node.prev_sibling = None;
            child_node.next_sibling = None;
        }
    }

    /// Returns an iterator over the direct children of a node.
    pub fn children(&self, parent: NodeId) -> crate::traverse::ChildIter<'_> {
        crate::traverse::ChildIter::new(self, parent)
    }

    /// Returns an iterator over all descendants of a node (depth-first).
    pub fn descendants(&self, root: NodeId) -> crate::traverse::DescendantIter<'_> {
        crate::traverse::DescendantIter::new(self, root)
    }

    /// Returns an iterator over the ancestors of a node (from parent to root).
    pub fn ancestors(&self, node: NodeId) -> crate::traverse::Ancestors<'_> {
        crate::traverse::Ancestors::new(self, node)
    }

    /// Returns the total number of nodes in the document.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the document element (usually <html>), if present.
    pub fn document_element(&self) -> Option<NodeId> {
        self.children(self.root)
            .find(|&id| {
                self.get(id)
                    .and_then(|n| n.as_element())
                    .is_some()
            })
    }

    /// Finds the first element with the given tag name (case-insensitive).
    pub fn get_element_by_tag_name(&self, tag_name: &str) -> Option<NodeId> {
        let tag_lower = tag_name.to_ascii_lowercase();
        self.descendants(self.root).find(|&id| {
            self.get(id)
                .and_then(|n| n.as_element())
                .map(|e| e.tag_name.to_ascii_lowercase() == tag_lower)
                .unwrap_or(false)
        })
    }

    /// Collects all text content from a subtree.
    pub fn text_content(&self, node: NodeId) -> String {
        let mut result = String::new();
        self.collect_text_content(node, &mut result);
        result
    }

    fn collect_text_content(&self, node: NodeId, result: &mut String) {
        if let Some(n) = self.get(node) {
            if let Some(text) = n.as_text() {
                result.push_str(text);
            }
            // Recurse into children
            let mut child = n.first_child;
            while let Some(child_id) = child {
                self.collect_text_content(child_id, result);
                child = self.get(child_id).and_then(|c| c.next_sibling);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_document() {
        let doc = Document::new();
        assert_eq!(doc.node_count(), 1);
        assert!(doc.get(doc.root()).unwrap().is_document());
    }

    #[test]
    fn test_create_elements() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let p = doc.create_element("p");

        assert_eq!(doc.node_count(), 3);
        assert_eq!(doc.get(div).unwrap().as_element().unwrap().tag_name, "div");
        assert_eq!(doc.get(p).unwrap().as_element().unwrap().tag_name, "p");
    }

    #[test]
    fn test_append_child() {
        let mut doc = Document::new();
        let html = doc.create_element("html");
        let body = doc.create_element("body");
        let p = doc.create_element("p");

        doc.append_child(doc.root(), html);
        doc.append_child(html, body);
        doc.append_child(body, p);

        // Check parent relationships
        assert_eq!(doc.get(html).unwrap().parent, Some(doc.root()));
        assert_eq!(doc.get(body).unwrap().parent, Some(html));
        assert_eq!(doc.get(p).unwrap().parent, Some(body));

        // Check children
        assert_eq!(doc.get(doc.root()).unwrap().first_child, Some(html));
        assert_eq!(doc.get(html).unwrap().first_child, Some(body));
        assert_eq!(doc.get(body).unwrap().first_child, Some(p));
    }

    #[test]
    fn test_siblings() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let child1 = doc.create_element("p");
        let child2 = doc.create_element("span");
        let child3 = doc.create_element("a");

        doc.append_child(doc.root(), parent);
        doc.append_child(parent, child1);
        doc.append_child(parent, child2);
        doc.append_child(parent, child3);

        // Check sibling relationships
        assert_eq!(doc.get(child1).unwrap().next_sibling, Some(child2));
        assert_eq!(doc.get(child2).unwrap().prev_sibling, Some(child1));
        assert_eq!(doc.get(child2).unwrap().next_sibling, Some(child3));
        assert_eq!(doc.get(child3).unwrap().prev_sibling, Some(child2));

        // Check first/last child
        assert_eq!(doc.get(parent).unwrap().first_child, Some(child1));
        assert_eq!(doc.get(parent).unwrap().last_child, Some(child3));
    }

    #[test]
    fn test_text_content() {
        let mut doc = Document::new();
        let p = doc.create_element("p");
        let text1 = doc.create_text("Hello ");
        let span = doc.create_element("span");
        let text2 = doc.create_text("World");
        let text3 = doc.create_text("!");

        doc.append_child(doc.root(), p);
        doc.append_child(p, text1);
        doc.append_child(p, span);
        doc.append_child(span, text2);
        doc.append_child(p, text3);

        assert_eq!(doc.text_content(p), "Hello World!");
    }
}
