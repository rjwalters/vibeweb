//! Core layout box types and geometry primitives.
//!
//! This module defines the fundamental types used in CSS layout:
//! - `LayoutBox`: A positioned box in the layout tree
//! - `Dimensions`: Box model dimensions (content, padding, border, margin)
//! - `Rect`: A positioned rectangle with x, y, width, height
//! - `Edges`: Edge values for padding, border, and margin

/// A unique identifier for DOM nodes.
/// This is a simple wrapper around usize for type safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl From<usize> for NodeId {
    fn from(id: usize) -> Self {
        NodeId(id)
    }
}

/// A laid-out box with computed geometry.
///
/// Each `LayoutBox` corresponds to a CSS box in the formatting model.
/// It contains its computed dimensions, box type, and child boxes.
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Reference to the DOM node that generated this box (None for anonymous boxes)
    pub node_id: Option<NodeId>,
    /// The type of box (block, inline, etc.)
    pub box_type: BoxType,
    /// Computed dimensions including content, padding, border, and margin
    pub dimensions: Dimensions,
    /// Child boxes in tree order
    pub children: Vec<LayoutBox>,
    /// Text content for text nodes
    pub text_content: Option<String>,
}

impl LayoutBox {
    /// Create a new layout box with the given type.
    pub fn new(box_type: BoxType) -> Self {
        LayoutBox {
            node_id: None,
            box_type,
            dimensions: Dimensions::default(),
            children: Vec::new(),
            text_content: None,
        }
    }

    /// Create a new block-level box.
    pub fn block() -> Self {
        LayoutBox::new(BoxType::Block)
    }

    /// Create a new inline-level box.
    pub fn inline() -> Self {
        LayoutBox::new(BoxType::Inline)
    }

    /// Create an anonymous block box (for mixed content).
    pub fn anonymous_block() -> Self {
        LayoutBox::new(BoxType::Anonymous)
    }

    /// Set the node ID for this box.
    pub fn with_node_id(mut self, id: NodeId) -> Self {
        self.node_id = Some(id);
        self
    }

    /// Set text content for this box.
    pub fn with_text(mut self, text: String) -> Self {
        self.text_content = Some(text);
        self
    }

    /// Add a child box.
    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    /// Returns the content box rectangle.
    pub fn content_box(&self) -> Rect {
        self.dimensions.content
    }

    /// Returns the padding box rectangle.
    pub fn padding_box(&self) -> Rect {
        self.dimensions.padding_box()
    }

    /// Returns the border box rectangle.
    pub fn border_box(&self) -> Rect {
        self.dimensions.border_box()
    }

    /// Returns the margin box rectangle.
    pub fn margin_box(&self) -> Rect {
        self.dimensions.margin_box()
    }
}

/// The type of CSS box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxType {
    /// Block-level box (participates in block formatting context)
    Block,
    /// Inline-level box (participates in inline formatting context)
    Inline,
    /// Inline-block box (inline outside, block inside)
    InlineBlock,
    /// Anonymous box created for mixed block/inline content
    Anonymous,
}

impl BoxType {
    /// Returns true if this is a block-level box.
    pub fn is_block_level(&self) -> bool {
        matches!(self, BoxType::Block | BoxType::Anonymous)
    }

    /// Returns true if this is an inline-level box.
    pub fn is_inline_level(&self) -> bool {
        matches!(self, BoxType::Inline | BoxType::InlineBlock)
    }
}

/// CSS box model dimensions.
///
/// Represents the complete box model with content, padding, border, and margin.
#[derive(Debug, Clone, Copy, Default)]
pub struct Dimensions {
    /// The content area rectangle (position and size)
    pub content: Rect,
    /// Padding edges
    pub padding: Edges,
    /// Border edges
    pub border: Edges,
    /// Margin edges
    pub margin: Edges,
}

impl Dimensions {
    /// Create dimensions from a viewport rectangle (for the root box).
    pub fn from_viewport(width: f32, height: f32) -> Self {
        Dimensions {
            content: Rect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            ..Default::default()
        }
    }

    /// Returns the padding box (content + padding).
    pub fn padding_box(&self) -> Rect {
        self.content.expanded_by(&self.padding)
    }

    /// Returns the border box (content + padding + border).
    pub fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(&self.border)
    }

    /// Returns the margin box (content + padding + border + margin).
    pub fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(&self.margin)
    }

    /// Total horizontal space occupied (margin + border + padding on both sides).
    pub fn horizontal_space(&self) -> f32 {
        self.margin.left
            + self.border.left
            + self.padding.left
            + self.padding.right
            + self.border.right
            + self.margin.right
    }

    /// Total vertical space occupied (margin + border + padding on both sides).
    pub fn vertical_space(&self) -> f32 {
        self.margin.top
            + self.border.top
            + self.padding.top
            + self.padding.bottom
            + self.border.bottom
            + self.margin.bottom
    }
}

/// A positioned rectangle.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    /// X coordinate of the top-left corner
    pub x: f32,
    /// Y coordinate of the top-left corner
    pub y: f32,
    /// Width of the rectangle
    pub width: f32,
    /// Height of the rectangle
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rectangle at the origin with the given size.
    pub fn from_size(width: f32, height: f32) -> Self {
        Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    /// Returns the right edge x-coordinate.
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Returns the bottom edge y-coordinate.
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Expand this rectangle by the given edges.
    pub fn expanded_by(&self, edges: &Edges) -> Rect {
        Rect {
            x: self.x - edges.left,
            y: self.y - edges.top,
            width: self.width + edges.left + edges.right,
            height: self.height + edges.top + edges.bottom,
        }
    }
}

/// Edge values (used for padding, border, and margin).
#[derive(Debug, Clone, Copy, Default)]
pub struct Edges {
    /// Top edge value
    pub top: f32,
    /// Right edge value
    pub right: f32,
    /// Bottom edge value
    pub bottom: f32,
    /// Left edge value
    pub left: f32,
}

impl Edges {
    /// Create uniform edges (same value on all sides).
    pub fn uniform(value: f32) -> Self {
        Edges {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create edges with specified values.
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Edges {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Horizontal sum (left + right).
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Vertical sum (top + bottom).
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_expanded_by() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let edges = Edges::new(5.0, 10.0, 15.0, 20.0);
        let expanded = rect.expanded_by(&edges);

        assert_eq!(expanded.x, -10.0); // 10 - 20
        assert_eq!(expanded.y, 15.0); // 20 - 5
        assert_eq!(expanded.width, 130.0); // 100 + 20 + 10
        assert_eq!(expanded.height, 70.0); // 50 + 5 + 15
    }

    #[test]
    fn test_dimensions_boxes() {
        let mut dims = Dimensions::default();
        dims.content = Rect::new(50.0, 50.0, 100.0, 100.0);
        dims.padding = Edges::uniform(10.0);
        dims.border = Edges::uniform(5.0);
        dims.margin = Edges::uniform(20.0);

        let padding_box = dims.padding_box();
        assert_eq!(padding_box.x, 40.0);
        assert_eq!(padding_box.y, 40.0);
        assert_eq!(padding_box.width, 120.0);
        assert_eq!(padding_box.height, 120.0);

        let border_box = dims.border_box();
        assert_eq!(border_box.x, 35.0);
        assert_eq!(border_box.y, 35.0);
        assert_eq!(border_box.width, 130.0);
        assert_eq!(border_box.height, 130.0);

        let margin_box = dims.margin_box();
        assert_eq!(margin_box.x, 15.0);
        assert_eq!(margin_box.y, 15.0);
        assert_eq!(margin_box.width, 170.0);
        assert_eq!(margin_box.height, 170.0);
    }

    #[test]
    fn test_box_type_classification() {
        assert!(BoxType::Block.is_block_level());
        assert!(BoxType::Anonymous.is_block_level());
        assert!(!BoxType::Inline.is_block_level());
        assert!(!BoxType::InlineBlock.is_block_level());

        assert!(BoxType::Inline.is_inline_level());
        assert!(BoxType::InlineBlock.is_inline_level());
        assert!(!BoxType::Block.is_inline_level());
    }

    #[test]
    fn test_layout_box_creation() {
        let mut block = LayoutBox::block().with_node_id(NodeId(1));
        block.add_child(LayoutBox::inline().with_text("Hello".to_string()));

        assert_eq!(block.node_id, Some(NodeId(1)));
        assert_eq!(block.box_type, BoxType::Block);
        assert_eq!(block.children.len(), 1);
        assert_eq!(block.children[0].text_content, Some("Hello".to_string()));
    }
}
