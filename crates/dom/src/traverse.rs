//! Tree traversal iterators for DOM nodes

use crate::document::Document;
use crate::node::NodeId;

/// Iterator over the direct children of a node.
pub struct ChildIter<'a> {
    doc: &'a Document,
    current: Option<NodeId>,
}

impl<'a> ChildIter<'a> {
    /// Creates a new iterator over the children of the given parent.
    pub fn new(doc: &'a Document, parent: NodeId) -> Self {
        let first_child = doc.get(parent).and_then(|n| n.first_child);
        Self {
            doc,
            current: first_child,
        }
    }
}

impl<'a> Iterator for ChildIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = self.doc.get(current).and_then(|n| n.next_sibling);
        Some(current)
    }
}

/// Iterator over all descendants of a node in depth-first pre-order.
pub struct DescendantIter<'a> {
    doc: &'a Document,
    #[allow(dead_code)]
    root: NodeId,
    stack: Vec<NodeId>,
}

impl<'a> DescendantIter<'a> {
    /// Creates a new iterator over the descendants of the given root.
    pub fn new(doc: &'a Document, root: NodeId) -> Self {
        let mut stack = Vec::new();
        // Start with the first child of root (not root itself)
        if let Some(node) = doc.get(root) {
            if let Some(first_child) = node.first_child {
                stack.push(first_child);
            }
        }
        Self { doc, root, stack }
    }
}

impl<'a> Iterator for DescendantIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop()?;

        if let Some(node) = self.doc.get(current) {
            // Push next sibling first (will be processed after children)
            if let Some(next_sibling) = node.next_sibling {
                self.stack.push(next_sibling);
            }

            // Push first child (will be processed next, before sibling)
            if let Some(first_child) = node.first_child {
                self.stack.push(first_child);
            }
        }

        Some(current)
    }
}

/// Iterator over the ancestors of a node (from parent to root).
pub struct Ancestors<'a> {
    doc: &'a Document,
    current: Option<NodeId>,
}

impl<'a> Ancestors<'a> {
    /// Creates a new iterator over the ancestors of the given node.
    pub fn new(doc: &'a Document, node: NodeId) -> Self {
        let parent = doc.get(node).and_then(|n| n.parent);
        Self {
            doc,
            current: parent,
        }
    }
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = self.doc.get(current).and_then(|n| n.parent);
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    fn build_test_tree() -> (Document, NodeId, NodeId, NodeId, NodeId) {
        let mut doc = Document::new();
        //       root
        //        |
        //       html
        //      /    \
        //    head   body
        //            |
        //           div
        let html = doc.create_element("html");
        let head = doc.create_element("head");
        let body = doc.create_element("body");
        let div = doc.create_element("div");

        doc.append_child(doc.root(), html);
        doc.append_child(html, head);
        doc.append_child(html, body);
        doc.append_child(body, div);

        (doc, html, head, body, div)
    }

    #[test]
    fn test_child_iter() {
        let (doc, html, head, body, _div) = build_test_tree();

        let children: Vec<_> = doc.children(html).collect();
        assert_eq!(children, vec![head, body]);

        let root_children: Vec<_> = doc.children(doc.root()).collect();
        assert_eq!(root_children, vec![html]);
    }

    #[test]
    fn test_descendant_iter() {
        let (doc, html, head, body, div) = build_test_tree();

        let descendants: Vec<_> = doc.descendants(doc.root()).collect();
        assert_eq!(descendants, vec![html, head, body, div]);

        let html_descendants: Vec<_> = doc.descendants(html).collect();
        assert_eq!(html_descendants, vec![head, body, div]);

        let body_descendants: Vec<_> = doc.descendants(body).collect();
        assert_eq!(body_descendants, vec![div]);
    }

    #[test]
    fn test_ancestors() {
        let (doc, html, _, body, div) = build_test_tree();

        let ancestors: Vec<_> = doc.ancestors(div).collect();
        assert_eq!(ancestors, vec![body, html, doc.root()]);

        let html_ancestors: Vec<_> = doc.ancestors(html).collect();
        assert_eq!(html_ancestors, vec![doc.root()]);
    }

    #[test]
    fn test_empty_iterators() {
        let doc = Document::new();

        let children: Vec<_> = doc.children(doc.root()).collect();
        assert!(children.is_empty());

        let descendants: Vec<_> = doc.descendants(doc.root()).collect();
        assert!(descendants.is_empty());

        let ancestors: Vec<_> = doc.ancestors(doc.root()).collect();
        assert!(ancestors.is_empty());
    }
}
