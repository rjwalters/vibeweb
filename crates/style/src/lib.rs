//! CSS Cascade Algorithm and Computed Style Resolution.
//!
//! This crate implements the CSS style system for the vibeweb browser, including:
//!
//! - **Values**: CSS value types (lengths, colors, etc.)
//! - **Properties**: CSS property types (display, position, etc.)
//! - **Computed Styles**: Final resolved style values for each DOM node
//! - **Cascade**: The CSS cascade algorithm for determining winning declarations
//! - **Defaults**: User-agent stylesheet and element defaults
//! - **Style Tree**: Document-wide style computation
//!
//! # Architecture
//!
//! The style system follows the CSS specification's cascade algorithm:
//!
//! ```text
//! Stylesheets → Selector Matching → Cascade → Computed Style
//!                    ↓                 ↓
//!               vw-css crate     This crate
//! ```
//!
//! # Usage
//!
//! ```
//! use vw_style::tree::{NodeInfo, compute_styles_default};
//!
//! // Build list of nodes in tree order (parents before children)
//! let nodes = vec![
//!     NodeInfo::element(0, None, "html"),
//!     NodeInfo::element(1, Some(0), "body"),
//!     NodeInfo::element(2, Some(1), "div"),
//!     NodeInfo::text(3, Some(2)),
//! ];
//!
//! // Compute styles using default (user-agent) rules only
//! let style_tree = compute_styles_default(&nodes);
//!
//! // Get computed style for a node
//! if let Some(style) = style_tree.get(2) {
//!     println!("div display: {}", style.display);
//!     println!("div color: {}", style.color);
//! }
//! ```
//!
//! # With Custom Rules
//!
//! To use author stylesheets, implement the `RuleMatcher` trait:
//!
//! ```
//! use vw_style::tree::{NodeInfo, RuleMatcher, StyleTreeBuilder};
//! use vw_style::cascade::MatchedRule;
//!
//! struct MyMatcher {
//!     // Your stylesheet data
//! }
//!
//! impl RuleMatcher for MyMatcher {
//!     fn match_rules(&self, node: &NodeInfo) -> Vec<MatchedRule> {
//!         // Return rules matching this node, sorted by specificity
//!         Vec::new()
//!     }
//! }
//!
//! let nodes = vec![
//!     NodeInfo::element(0, None, "html"),
//! ];
//!
//! let matcher = MyMatcher { /* ... */ };
//! let builder = StyleTreeBuilder::new(&matcher);
//! let style_tree = builder.build(&nodes);
//! ```

pub mod cascade;
pub mod computed;
pub mod defaults;
pub mod properties;
pub mod tree;
pub mod values;

// Re-export commonly used types at the crate root
pub use cascade::{Cascade, CssValue, Declaration, MatchedRule, Origin, PropertyId, Specificity};
pub use computed::ComputedStyle;
pub use defaults::{default_display_for_element, user_agent_rules_for_element};
pub use properties::{
    BoxSizing, Clear, Display, Float, FontWeight, LineHeight, Overflow, Position, TextAlign,
    VerticalAlign, Visibility, WhiteSpace,
};
pub use tree::{compute_styles_default, NodeId, NodeInfo, RuleMatcher, StyleTree, StyleTreeBuilder};
pub use values::{Color, Length, LengthOrAuto, Sides};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_to_end_simple() {
        // Simple document: html > body > p > "text"
        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "body"),
            NodeInfo::element(2, Some(1), "p"),
            NodeInfo::text(3, Some(2)),
        ];

        let tree = compute_styles_default(&nodes);

        // All nodes should have styles
        assert_eq!(tree.len(), 4);

        // Check expected display values
        let html = tree.get(0).unwrap();
        let body = tree.get(1).unwrap();
        let p = tree.get(2).unwrap();
        let text = tree.get(3).unwrap();

        assert_eq!(html.display, Display::Block);
        assert_eq!(body.display, Display::Block);
        assert_eq!(p.display, Display::Block);

        // Text inherits parent's display
        assert_eq!(text.display, Display::Block);

        // Root font size should be set
        assert_eq!(tree.root_font_size(), 16.0);
    }

    #[test]
    fn test_end_to_end_mixed_content() {
        // Document with inline and block elements
        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "body"),
            NodeInfo::element(2, Some(1), "p"),
            NodeInfo::element(3, Some(2), "strong"),
            NodeInfo::text(4, Some(3)),
            NodeInfo::element(5, Some(2), "em"),
        ];

        let tree = compute_styles_default(&nodes);

        let p = tree.get(2).unwrap();
        let strong = tree.get(3).unwrap();
        let em = tree.get(5).unwrap();

        // p is block, strong and em are inline
        assert_eq!(p.display, Display::Block);
        assert_eq!(strong.display, Display::Inline);
        assert_eq!(em.display, Display::Inline);

        // strong should be bold
        assert!(strong.font_weight.is_bold());
    }

    #[test]
    fn test_cascade_with_author_rules() {
        use cascade::{CssValue, Declaration, MatchedRule, Origin, PropertyId, Specificity};

        // Create a matcher that adds author styles
        struct AuthorMatcher;

        impl RuleMatcher for AuthorMatcher {
            fn match_rules(&self, node: &NodeInfo) -> Vec<MatchedRule> {
                let mut rules = defaults::user_agent_rules_for_element(
                    node.tag_name.as_deref().unwrap_or(""),
                );

                // Add author rule: all divs are red
                if node.tag_name.as_deref() == Some("div") {
                    rules.push(MatchedRule::new(
                        vec![Declaration::new(
                            PropertyId::Color,
                            CssValue::Color(Color::rgb(255, 0, 0)),
                        )],
                        Specificity::new(0, 0, 1), // Type selector specificity
                        Origin::Author,
                        1, // After UA rules
                    ));
                }

                rules
            }
        }

        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "body"),
            NodeInfo::element(2, Some(1), "div"),
        ];

        let matcher = AuthorMatcher;
        let builder = StyleTreeBuilder::new(&matcher);
        let tree = builder.build(&nodes);

        // Div should be red (author style)
        let div = tree.get(2).unwrap();
        assert_eq!(div.color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn test_inheritance_chain() {
        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "body"),
            NodeInfo::element(2, Some(1), "div"),
            NodeInfo::element(3, Some(2), "div"),
            NodeInfo::element(4, Some(3), "div"),
            NodeInfo::text(5, Some(4)),
        ];

        let tree = compute_styles_default(&nodes);

        // All should inherit the same color (black default)
        let body = tree.get(1).unwrap();
        let div3 = tree.get(4).unwrap();
        let text = tree.get(5).unwrap();

        assert_eq!(body.color, div3.color);
        assert_eq!(div3.color, text.color);
    }

    #[test]
    fn test_hidden_elements() {
        let nodes = vec![
            NodeInfo::element(0, None, "html"),
            NodeInfo::element(1, Some(0), "head"),
            NodeInfo::element(2, Some(1), "title"),
            NodeInfo::element(3, Some(1), "style"),
            NodeInfo::element(4, Some(0), "body"),
        ];

        let tree = compute_styles_default(&nodes);

        let head = tree.get(1).unwrap();
        let title = tree.get(2).unwrap();
        let style = tree.get(3).unwrap();
        let body = tree.get(4).unwrap();

        // Head, title, style should be display:none
        assert_eq!(head.display, Display::None);
        assert_eq!(title.display, Display::None);
        assert_eq!(style.display, Display::None);

        // Body should be block
        assert_eq!(body.display, Display::Block);
    }
}
