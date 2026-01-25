//! Style Tree: Computed styles for all nodes in a document.
//!
//! This module provides the `StyleTree` struct which stores computed styles
//! for all DOM nodes, indexed by node ID.

use crate::cascade::{Cascade, MatchedRule};
use crate::computed::ComputedStyle;
use crate::defaults::user_agent_rules_for_element;
use std::collections::HashMap;

/// A unique identifier for a DOM node.
///
/// This type is designed to be compatible with the vw-dom crate's NodeId.
pub type NodeId = usize;

/// Information about a DOM node needed for style computation.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// The node's unique identifier.
    pub id: NodeId,
    /// The parent node's ID, if any.
    pub parent: Option<NodeId>,
    /// The element's tag name (lowercase), or None for text/comment nodes.
    pub tag_name: Option<String>,
    /// Whether this is the root element.
    pub is_root: bool,
}

impl NodeInfo {
    /// Create info for an element node.
    pub fn element(id: NodeId, parent: Option<NodeId>, tag_name: &str) -> Self {
        NodeInfo {
            id,
            parent,
            tag_name: Some(tag_name.to_lowercase()),
            is_root: parent.is_none(),
        }
    }

    /// Create info for a text node.
    pub fn text(id: NodeId, parent: Option<NodeId>) -> Self {
        NodeInfo {
            id,
            parent,
            tag_name: None,
            is_root: false,
        }
    }

    /// Check if this is an element node.
    pub fn is_element(&self) -> bool {
        self.tag_name.is_some()
    }

    /// Check if this is a text node.
    pub fn is_text(&self) -> bool {
        self.tag_name.is_none()
    }
}

/// The computed style tree for a document.
///
/// Stores computed styles for all nodes, indexed by NodeId.
#[derive(Debug, Default)]
pub struct StyleTree {
    /// Computed styles indexed by node ID.
    styles: HashMap<NodeId, ComputedStyle>,
    /// The root font size (used for rem units).
    root_font_size: f32,
}

impl StyleTree {
    /// Create a new empty style tree.
    pub fn new() -> Self {
        StyleTree {
            styles: HashMap::new(),
            root_font_size: 16.0,
        }
    }

    /// Get the computed style for a node.
    pub fn get(&self, id: NodeId) -> Option<&ComputedStyle> {
        self.styles.get(&id)
    }

    /// Get a mutable reference to the computed style for a node.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut ComputedStyle> {
        self.styles.get_mut(&id)
    }

    /// Set the computed style for a node.
    pub fn set(&mut self, id: NodeId, style: ComputedStyle) {
        self.styles.insert(id, style);
    }

    /// Get the root font size.
    pub fn root_font_size(&self) -> f32 {
        self.root_font_size
    }

    /// Set the root font size.
    pub fn set_root_font_size(&mut self, size: f32) {
        self.root_font_size = size;
    }

    /// Get the number of nodes with computed styles.
    pub fn len(&self) -> usize {
        self.styles.len()
    }

    /// Check if the style tree is empty.
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    /// Iterate over all styles.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &ComputedStyle)> {
        self.styles.iter().map(|(&id, style)| (id, style))
    }
}

/// A rule matcher that can find rules matching a given element.
///
/// This trait abstracts the selector matching process, which will be
/// implemented by the vw-css crate's selector matching engine.
pub trait RuleMatcher {
    /// Find all rules matching the given element.
    ///
    /// Returns rules sorted by specificity and source order.
    fn match_rules(&self, node: &NodeInfo) -> Vec<MatchedRule>;
}

/// A simple rule matcher that only uses user-agent defaults.
///
/// This is useful for testing and for when no author stylesheets are present.
#[derive(Debug, Default)]
pub struct DefaultMatcher;

impl RuleMatcher for DefaultMatcher {
    fn match_rules(&self, node: &NodeInfo) -> Vec<MatchedRule> {
        if let Some(tag) = &node.tag_name {
            user_agent_rules_for_element(tag)
        } else {
            Vec::new()
        }
    }
}

/// Style tree builder that computes styles for a document.
pub struct StyleTreeBuilder<'a, M: RuleMatcher> {
    matcher: &'a M,
}

impl<'a, M: RuleMatcher> StyleTreeBuilder<'a, M> {
    /// Create a new style tree builder.
    pub fn new(matcher: &'a M) -> Self {
        StyleTreeBuilder { matcher }
    }

    /// Compute the style for a single node.
    pub fn compute_style(
        &self,
        node: &NodeInfo,
        parent_style: Option<&ComputedStyle>,
    ) -> ComputedStyle {
        // Text nodes inherit all styles from parent
        if node.is_text() {
            return parent_style
                .map(ComputedStyle::inherit_from)
                .unwrap_or_default();
        }

        // Get matching rules
        let rules = self.matcher.match_rules(node);

        // Build cascade
        let mut cascade = Cascade::new();
        for rule in &rules {
            cascade.add_rule(rule);
        }

        // Compute final style
        cascade.compute(parent_style)
    }

    /// Build a style tree for a list of nodes.
    ///
    /// Nodes must be in tree order (parents before children).
    pub fn build(&self, nodes: &[NodeInfo]) -> StyleTree {
        let mut tree = StyleTree::new();

        for node in nodes {
            let parent_style = node.parent.and_then(|pid| tree.get(pid));
            let style = self.compute_style(node, parent_style);

            // Update root font size if this is the root element
            if node.is_root {
                tree.set_root_font_size(style.font_size);
            }

            tree.set(node.id, style);
        }

        tree
    }
}

/// Convenience function to compute styles with default matcher only.
pub fn compute_styles_default(nodes: &[NodeInfo]) -> StyleTree {
    let matcher = DefaultMatcher;
    let builder = StyleTreeBuilder::new(&matcher);
    builder.build(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::Display;

    #[test]
    fn test_style_tree_basic() {
        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "body"),
            NodeInfo::element(2, Some(1), "div"),
            NodeInfo::text(3, Some(2)),
        ];

        let tree = compute_styles_default(&nodes);

        assert_eq!(tree.len(), 4);
        assert!(tree.get(0).is_some());
        assert!(tree.get(1).is_some());
        assert!(tree.get(2).is_some());
        assert!(tree.get(3).is_some());
    }

    #[test]
    fn test_element_defaults() {
        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "body"),
            NodeInfo::element(2, Some(1), "div"),
            NodeInfo::element(3, Some(1), "span"),
        ];

        let tree = compute_styles_default(&nodes);

        assert_eq!(tree.get(2).unwrap().display, Display::Block);
        assert_eq!(tree.get(3).unwrap().display, Display::Inline);
    }

    #[test]
    fn test_inheritance() {
        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "body"),
            NodeInfo::text(2, Some(1)),
        ];

        let tree = compute_styles_default(&nodes);

        let body = tree.get(1).unwrap();
        let text = tree.get(2).unwrap();

        // Text node should inherit from body
        assert_eq!(text.color, body.color);
        assert_eq!(text.font_size, body.font_size);
    }

    #[test]
    fn test_heading_styles() {
        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "body"),
            NodeInfo::element(2, Some(1), "h1"),
            NodeInfo::element(3, Some(1), "h2"),
            NodeInfo::element(4, Some(1), "p"),
        ];

        let tree = compute_styles_default(&nodes);

        let h1 = tree.get(2).unwrap();
        let h2 = tree.get(3).unwrap();
        let p = tree.get(4).unwrap();

        // h1 should have largest font
        assert!(h1.font_size > h2.font_size);
        assert!(h2.font_size > p.font_size);

        // All should be block
        assert_eq!(h1.display, Display::Block);
        assert_eq!(h2.display, Display::Block);
        assert_eq!(p.display, Display::Block);
    }

    #[test]
    fn test_custom_matcher() {
        use crate::cascade::{CssValue, Declaration, Origin, PropertyId, Specificity};
        use crate::values::Color;

        struct RedMatcher;

        impl RuleMatcher for RedMatcher {
            fn match_rules(&self, _node: &NodeInfo) -> Vec<MatchedRule> {
                vec![MatchedRule::new(
                    vec![Declaration::new(
                        PropertyId::Color,
                        CssValue::Color(Color::rgb(255, 0, 0)),
                    )],
                    Specificity::new(0, 1, 0),
                    Origin::Author,
                    0,
                )]
            }
        }

        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "body"),
        ];

        let matcher = RedMatcher;
        let builder = StyleTreeBuilder::new(&matcher);
        let tree = builder.build(&nodes);

        assert_eq!(tree.get(1).unwrap().color, Color::rgb(255, 0, 0));
    }
}
