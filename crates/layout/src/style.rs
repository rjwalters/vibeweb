//! Computed style types for layout.
//!
//! This module defines the computed style interface that layout needs.
//! These types represent the resolved CSS values after cascade and inheritance.
//!
//! Note: When `vw-style` is fully implemented, these types may be replaced
//! with re-exports from that crate.

use crate::box_types::NodeId;
use std::collections::HashMap;

/// A computed length value in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Length {
    /// Absolute length in pixels
    Px(f32),
    /// Percentage of containing block
    Percent(f32),
    /// Auto (browser computes the value)
    #[default]
    Auto,
}

impl Length {
    /// Resolve this length to pixels given the containing block dimension.
    pub fn to_px(&self, containing: f32) -> Option<f32> {
        match *self {
            Length::Px(px) => Some(px),
            Length::Percent(pct) => Some(containing * pct / 100.0),
            Length::Auto => None,
        }
    }

    /// Returns true if this is `Auto`.
    pub fn is_auto(&self) -> bool {
        matches!(self, Length::Auto)
    }
}

/// The CSS `display` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    /// Block-level display
    #[default]
    Block,
    /// Inline-level display
    Inline,
    /// Inline-block display
    InlineBlock,
    /// Element is not rendered
    None,
}

impl Display {
    /// Returns true if this display value generates a block-level box.
    pub fn is_block(&self) -> bool {
        matches!(self, Display::Block)
    }

    /// Returns true if this display value generates an inline-level box.
    pub fn is_inline(&self) -> bool {
        matches!(self, Display::Inline | Display::InlineBlock)
    }

    /// Returns true if this display value means the element is not rendered.
    pub fn is_none(&self) -> bool {
        matches!(self, Display::None)
    }
}

/// Computed style values for a single element.
///
/// This contains all the CSS properties needed for layout, with their
/// computed (resolved) values.
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    /// The display property
    pub display: Display,

    // Sizing properties
    /// Width of the content area
    pub width: Length,
    /// Height of the content area
    pub height: Length,

    // Margin properties
    /// Top margin
    pub margin_top: Length,
    /// Right margin
    pub margin_right: Length,
    /// Bottom margin
    pub margin_bottom: Length,
    /// Left margin
    pub margin_left: Length,

    // Padding properties (cannot be negative or auto)
    /// Top padding in pixels
    pub padding_top: f32,
    /// Right padding in pixels
    pub padding_right: f32,
    /// Bottom padding in pixels
    pub padding_bottom: f32,
    /// Left padding in pixels
    pub padding_left: f32,

    // Border properties
    /// Top border width in pixels
    pub border_top_width: f32,
    /// Right border width in pixels
    pub border_right_width: f32,
    /// Bottom border width in pixels
    pub border_bottom_width: f32,
    /// Left border width in pixels
    pub border_left_width: f32,

    // Text properties
    /// Font size in pixels
    pub font_size: f32,
    /// Line height (multiplier or absolute)
    pub line_height: LineHeight,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        ComputedStyle {
            display: Display::Block,
            width: Length::Auto,
            height: Length::Auto,
            margin_top: Length::Px(0.0),
            margin_right: Length::Px(0.0),
            margin_bottom: Length::Px(0.0),
            margin_left: Length::Px(0.0),
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            border_top_width: 0.0,
            border_right_width: 0.0,
            border_bottom_width: 0.0,
            border_left_width: 0.0,
            font_size: 16.0,
            line_height: LineHeight::Normal,
        }
    }
}

/// Line height value.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineHeight {
    /// Normal line height (typically 1.2)
    #[default]
    Normal,
    /// Unitless multiplier (e.g., 1.5)
    Number(f32),
    /// Absolute length in pixels
    Px(f32),
}

impl LineHeight {
    /// Resolve line height to pixels given the font size.
    pub fn to_px(&self, font_size: f32) -> f32 {
        match *self {
            LineHeight::Normal => font_size * 1.2,
            LineHeight::Number(n) => font_size * n,
            LineHeight::Px(px) => px,
        }
    }
}

/// A collection of computed styles for all nodes in the document.
///
/// The style tree maps node IDs to their computed styles after cascade
/// and inheritance are applied.
#[derive(Debug, Clone, Default)]
pub struct StyleTree {
    styles: HashMap<NodeId, ComputedStyle>,
}

impl StyleTree {
    /// Create a new empty style tree.
    pub fn new() -> Self {
        StyleTree {
            styles: HashMap::new(),
        }
    }

    /// Get the computed style for a node.
    pub fn get(&self, node_id: NodeId) -> Option<&ComputedStyle> {
        self.styles.get(&node_id)
    }

    /// Set the computed style for a node.
    pub fn insert(&mut self, node_id: NodeId, style: ComputedStyle) {
        self.styles.insert(node_id, style);
    }

    /// Check if a style exists for a node.
    pub fn contains(&self, node_id: NodeId) -> bool {
        self.styles.contains_key(&node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_resolution() {
        assert_eq!(Length::Px(100.0).to_px(500.0), Some(100.0));
        assert_eq!(Length::Percent(50.0).to_px(500.0), Some(250.0));
        assert_eq!(Length::Auto.to_px(500.0), None);
    }

    #[test]
    fn test_display_classification() {
        assert!(Display::Block.is_block());
        assert!(!Display::Inline.is_block());

        assert!(Display::Inline.is_inline());
        assert!(Display::InlineBlock.is_inline());
        assert!(!Display::Block.is_inline());

        assert!(Display::None.is_none());
    }

    #[test]
    fn test_line_height_resolution() {
        let font_size = 16.0;
        assert_eq!(LineHeight::Normal.to_px(font_size), 19.2); // 16 * 1.2
        assert_eq!(LineHeight::Number(1.5).to_px(font_size), 24.0);
        assert_eq!(LineHeight::Px(20.0).to_px(font_size), 20.0);
    }

    #[test]
    fn test_style_tree() {
        let mut tree = StyleTree::new();
        let node_id = NodeId(1);

        assert!(!tree.contains(node_id));

        let style = ComputedStyle {
            display: Display::Inline,
            ..Default::default()
        };
        tree.insert(node_id, style);

        assert!(tree.contains(node_id));
        assert_eq!(tree.get(node_id).unwrap().display, Display::Inline);
    }
}
