//! Layout tree, block/inline layout, text measurement.
//!
//! This crate implements CSS 2.1 visual formatting model layout.
//! It transforms a styled DOM tree into a positioned layout tree
//! with concrete pixel coordinates for rendering.
//!
//! # Overview
//!
//! The layout process works in several phases:
//!
//! 1. **Tree Building**: Convert DOM + styles into a box tree
//! 2. **Block Layout**: Position block-level boxes vertically
//! 3. **Inline Layout**: Flow inline content into line boxes
//! 4. **Output**: A tree of positioned boxes ready for painting
//!
//! # Example
//!
//! ```
//! use vw_layout::{layout, Viewport};
//! use vw_layout::dom::Document;
//! use vw_layout::style::StyleTree;
//!
//! // Create a simple document
//! let document = Document::new();
//! let styles = StyleTree::new();
//!
//! // Perform layout
//! let viewport = Viewport::new(800.0, 600.0);
//! let layout_tree = layout(&document, &styles, &viewport);
//!
//! // The layout tree can now be traversed for painting
//! if let Some(root) = layout_tree {
//!     println!("Root box: {:?}", root.dimensions.content);
//! }
//! ```
//!
//! # Box Model
//!
//! Each layout box has dimensions following the CSS box model:
//!
//! ```text
//! +---------------------------+
//! |         margin            |
//! |  +---------------------+  |
//! |  |      border         |  |
//! |  |  +---------------+  |  |
//! |  |  |    padding    |  |  |
//! |  |  |  +---------+  |  |  |
//! |  |  |  | content |  |  |  |
//! |  |  |  +---------+  |  |  |
//! |  |  +---------------+  |  |
//! |  +---------------------+  |
//! +---------------------------+
//! ```

// Module declarations
pub mod block;
pub mod box_types;
pub mod dom;
pub mod fonts;
pub mod inline;
pub mod style;
pub mod tree;

// Re-exports for convenient access
pub use box_types::{BoxType, Dimensions, Edges, LayoutBox, NodeId, Rect};
pub use fonts::{FixedFontMetrics, FontMetrics, TextMetrics};
pub use inline::{InlineFragment, InlineLayoutResult, LineBox};
pub use style::{ComputedStyle, Display, Length, LineHeight, StyleTree};

/// Viewport dimensions for layout.
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// Viewport width in pixels
    pub width: f32,
    /// Viewport height in pixels
    pub height: f32,
}

impl Viewport {
    /// Create a new viewport with the given dimensions.
    pub fn new(width: f32, height: f32) -> Self {
        Viewport { width, height }
    }

    /// Convert to a containing block dimensions.
    pub fn to_containing_block(&self) -> Dimensions {
        Dimensions::from_viewport(self.width, self.height)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            width: 800.0,
            height: 600.0,
        }
    }
}

/// Compute layout for a styled document.
///
/// This is the main entry point for the layout engine. It takes a DOM document,
/// computed styles, and viewport dimensions, and returns a fully laid-out
/// box tree with computed positions and sizes.
///
/// # Arguments
///
/// * `document` - The DOM document to lay out
/// * `styles` - Computed styles for each node
/// * `viewport` - Viewport dimensions
///
/// # Returns
///
/// The root layout box, or None if the document has no layoutable content.
pub fn layout(
    document: &dom::Document,
    styles: &StyleTree,
    viewport: &Viewport,
) -> Option<LayoutBox> {
    // Build the layout tree from DOM + styles
    let mut root = tree::build_layout_tree(document, styles)?;

    // Perform block layout starting from the viewport
    let containing_block = viewport.to_containing_block();
    root.layout_block(&containing_block);

    Some(root)
}

/// Compute layout with custom font metrics.
///
/// This variant allows providing a custom font metrics implementation
/// for accurate text measurement.
pub fn layout_with_fonts<F: FontMetrics>(
    document: &dom::Document,
    styles: &StyleTree,
    viewport: &Viewport,
    fonts: &F,
) -> Option<LayoutBox> {
    // Build the layout tree
    let mut root = tree::build_layout_tree(document, styles)?;

    // Perform layout with font-aware inline handling
    let containing_block = viewport.to_containing_block();
    layout_box_recursive(&mut root, &containing_block, fonts, 16.0);

    Some(root)
}

/// Recursively lay out a box and its children.
fn layout_box_recursive<F: FontMetrics>(
    layout_box: &mut LayoutBox,
    containing_block: &Dimensions,
    fonts: &F,
    font_size: f32,
) {
    match layout_box.box_type {
        BoxType::Block | BoxType::Anonymous => {
            layout_box.layout_block(containing_block);
        }
        BoxType::Inline | BoxType::InlineBlock => {
            // Inline boxes are handled by their containing block's
            // inline formatting context
        }
    }

    // Layout children
    let child_containing = Dimensions {
        content: layout_box.dimensions.content,
        ..Default::default()
    };

    for child in &mut layout_box.children {
        layout_box_recursive(child, &child_containing, fonts, font_size);
    }
}

/// Walk a layout tree and call a visitor function for each box.
///
/// The visitor receives the box and its depth in the tree.
pub fn walk_layout_tree<F>(root: &LayoutBox, mut visitor: F)
where
    F: FnMut(&LayoutBox, usize),
{
    walk_layout_tree_recursive(root, 0, &mut visitor);
}

fn walk_layout_tree_recursive<F>(node: &LayoutBox, depth: usize, visitor: &mut F)
where
    F: FnMut(&LayoutBox, usize),
{
    visitor(node, depth);
    for child in &node.children {
        walk_layout_tree_recursive(child, depth + 1, visitor);
    }
}

/// Debug print a layout tree.
pub fn debug_layout_tree(root: &LayoutBox) {
    walk_layout_tree(root, |node, depth| {
        let indent = "  ".repeat(depth);
        let box_type = match node.box_type {
            BoxType::Block => "block",
            BoxType::Inline => "inline",
            BoxType::InlineBlock => "inline-block",
            BoxType::Anonymous => "anonymous",
        };
        let rect = node.dimensions.content;
        println!(
            "{}{} @ ({}, {}) {}x{}",
            indent, box_type, rect.x, rect.y, rect.width, rect.height
        );
        if let Some(ref text) = node.text_content {
            println!("{}  text: {:?}", indent, text);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{Document, NodeData};

    fn create_simple_document() -> (Document, StyleTree) {
        let mut doc = Document::new();
        let mut styles = StyleTree::new();

        // Create: <html><body><div>Hello</div></body></html>
        let root_id = doc.add_node(NodeData::document(NodeId(0)));
        doc.set_root(root_id);

        let html_id = doc.add_node(NodeData::element(NodeId(0), "html"));
        let body_id = doc.add_node(NodeData::element(NodeId(0), "body"));
        let div_id = doc.add_node(NodeData::element(NodeId(0), "div"));
        let text_id = doc.add_node(NodeData::text(NodeId(0), "Hello"));

        // Link
        doc.get_node_mut(root_id).unwrap().children.push(html_id);
        doc.get_node_mut(html_id).unwrap().children.push(body_id);
        doc.get_node_mut(body_id).unwrap().children.push(div_id);
        doc.get_node_mut(div_id).unwrap().children.push(text_id);

        // Styles
        styles.insert(html_id, ComputedStyle::default());
        styles.insert(body_id, ComputedStyle::default());
        styles.insert(div_id, ComputedStyle::default());

        (doc, styles)
    }

    #[test]
    fn test_basic_layout() {
        let (doc, styles) = create_simple_document();
        let viewport = Viewport::new(800.0, 600.0);

        let layout_tree = layout(&doc, &styles, &viewport);

        assert!(layout_tree.is_some());
        let root = layout_tree.unwrap();

        // Root should be positioned at origin
        assert_eq!(root.dimensions.content.x, 0.0);
        assert_eq!(root.dimensions.content.y, 0.0);

        // Root should fill viewport width
        assert_eq!(root.dimensions.content.width, 800.0);
    }

    #[test]
    fn test_viewport_creation() {
        let viewport = Viewport::new(1024.0, 768.0);
        assert_eq!(viewport.width, 1024.0);
        assert_eq!(viewport.height, 768.0);

        let containing = viewport.to_containing_block();
        assert_eq!(containing.content.width, 1024.0);
        assert_eq!(containing.content.height, 768.0);
    }

    #[test]
    fn test_layout_with_fonts() {
        let (doc, styles) = create_simple_document();
        let viewport = Viewport::new(800.0, 600.0);
        let fonts = FixedFontMetrics::default();

        let layout_tree = layout_with_fonts(&doc, &styles, &viewport, &fonts);

        assert!(layout_tree.is_some());
    }

    #[test]
    fn test_walk_layout_tree() {
        let (doc, styles) = create_simple_document();
        let viewport = Viewport::new(800.0, 600.0);

        let layout_tree = layout(&doc, &styles, &viewport).unwrap();

        let mut count = 0;
        let mut max_depth = 0;

        walk_layout_tree(&layout_tree, |_node, depth| {
            count += 1;
            max_depth = max_depth.max(depth);
        });

        // Should have: document, html, body, div, anonymous(text)
        assert!(count >= 4);
        assert!(max_depth >= 3);
    }

    #[test]
    fn test_empty_document() {
        let doc = Document::new();
        let styles = StyleTree::new();
        let viewport = Viewport::new(800.0, 600.0);

        let layout_tree = layout(&doc, &styles, &viewport);

        assert!(layout_tree.is_none());
    }

    #[test]
    fn test_layout_determinism() {
        let (doc, styles) = create_simple_document();
        let viewport = Viewport::new(800.0, 600.0);

        // Run layout twice
        let result1 = layout(&doc, &styles, &viewport).unwrap();
        let result2 = layout(&doc, &styles, &viewport).unwrap();

        // Results should be identical
        assert_eq!(result1.dimensions.content.x, result2.dimensions.content.x);
        assert_eq!(result1.dimensions.content.y, result2.dimensions.content.y);
        assert_eq!(
            result1.dimensions.content.width,
            result2.dimensions.content.width
        );
        assert_eq!(
            result1.dimensions.content.height,
            result2.dimensions.content.height
        );
    }
}
