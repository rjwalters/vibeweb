//! Block layout algorithm.
//!
//! This module implements CSS 2.1 block formatting context layout.
//! Block-level boxes are laid out vertically, one after another.

use crate::box_types::{Dimensions, Edges, LayoutBox, Rect};
use crate::style::{ComputedStyle, Length};

impl LayoutBox {
    /// Lay out this box and its children in a block formatting context.
    ///
    /// The containing block provides the available width and position.
    pub fn layout_block(&mut self, containing_block: &Dimensions) {
        // Calculate width first (depends on containing block)
        self.calculate_block_width(containing_block);

        // Calculate position
        self.calculate_block_position(containing_block);

        // Lay out children
        self.layout_block_children();

        // Calculate height (may depend on children if height is auto)
        self.calculate_block_height(containing_block);
    }

    /// Calculate the width of a block-level box.
    ///
    /// Implements the CSS 2.1 width calculation algorithm:
    /// 'margin-left' + 'border-left-width' + 'padding-left' + 'width' +
    /// 'padding-right' + 'border-right-width' + 'margin-right' = width of containing block
    pub fn calculate_block_width_with_style(
        &mut self,
        containing_block: &Dimensions,
        style: &ComputedStyle,
    ) {
        let containing_width = containing_block.content.width;

        // Get specified values
        let width = style.width;
        let margin_left = style.margin_left;
        let margin_right = style.margin_right;

        // Border and padding are never auto
        let border_left = style.border_left_width;
        let border_right = style.border_right_width;
        let padding_left = style.padding_left;
        let padding_right = style.padding_right;

        // Calculate total non-auto horizontal space
        let total_fixed = border_left + padding_left + padding_right + border_right;

        // Resolve width and margins
        let (resolved_width, resolved_margin_left, resolved_margin_right) = self
            .resolve_width_and_margins(
                containing_width,
                width,
                margin_left,
                margin_right,
                total_fixed,
            );

        // Set dimensions
        self.dimensions.content.width = resolved_width;
        self.dimensions.padding = Edges::new(
            style.padding_top,
            padding_right,
            style.padding_bottom,
            padding_left,
        );
        self.dimensions.border = Edges::new(
            style.border_top_width,
            border_right,
            style.border_bottom_width,
            border_left,
        );
        self.dimensions.margin = Edges::new(
            style.margin_top.to_px(containing_width).unwrap_or(0.0),
            resolved_margin_right,
            style.margin_bottom.to_px(containing_width).unwrap_or(0.0),
            resolved_margin_left,
        );
    }

    /// Resolve width and auto margins according to CSS 2.1 rules.
    fn resolve_width_and_margins(
        &self,
        containing_width: f32,
        width: Length,
        margin_left: Length,
        margin_right: Length,
        total_fixed: f32,
    ) -> (f32, f32, f32) {
        // Count auto values
        let width_auto = width.is_auto();
        let margin_left_auto = margin_left.is_auto();
        let margin_right_auto = margin_right.is_auto();

        // Resolve non-auto values
        let width_px = width.to_px(containing_width);
        let margin_left_px = margin_left.to_px(containing_width);
        let margin_right_px = margin_right.to_px(containing_width);

        if width_auto {
            // If width is auto, set margins to 0 if auto, then width fills remaining space
            let ml = margin_left_px.unwrap_or(0.0);
            let mr = margin_right_px.unwrap_or(0.0);
            let w = (containing_width - total_fixed - ml - mr).max(0.0);
            (w, ml, mr)
        } else {
            let w = width_px.unwrap();

            if !margin_left_auto && !margin_right_auto {
                // Over-constrained: ignore margin-right
                let ml = margin_left_px.unwrap_or(0.0);
                let remaining = containing_width - total_fixed - w - ml;
                (w, ml, remaining)
            } else if margin_left_auto && margin_right_auto {
                // Both auto: center the element
                let remaining = containing_width - total_fixed - w;
                let margin = (remaining / 2.0).max(0.0);
                (w, margin, margin)
            } else if margin_left_auto {
                // Only left auto: it takes remaining space
                let mr = margin_right_px.unwrap_or(0.0);
                let ml = (containing_width - total_fixed - w - mr).max(0.0);
                (w, ml, mr)
            } else {
                // Only right auto: it takes remaining space
                let ml = margin_left_px.unwrap_or(0.0);
                let mr = (containing_width - total_fixed - w - ml).max(0.0);
                (w, ml, mr)
            }
        }
    }

    /// Calculate width using default style (for boxes without explicit style).
    fn calculate_block_width(&mut self, containing_block: &Dimensions) {
        // Default: width auto (fill available space), margins 0
        let containing_width = containing_block.content.width;
        let total_horizontal = self.dimensions.horizontal_space();
        self.dimensions.content.width = (containing_width - total_horizontal).max(0.0);
    }

    /// Calculate the position of a block-level box.
    fn calculate_block_position(&mut self, containing_block: &Dimensions) {
        let d = &mut self.dimensions;

        // x position: containing block x + margin-left + border-left + padding-left
        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;

        // y position: at the top of the containing block's content area
        // (For child boxes, position_at_y is used instead to place them vertically)
        d.content.y = containing_block.content.y + d.margin.top + d.border.top + d.padding.top;
    }

    /// Position this box at a specific y-coordinate.
    pub fn position_at_y(&mut self, y: f32, containing_block: &Dimensions) {
        let d = &mut self.dimensions;
        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;
        d.content.y = y + d.margin.top + d.border.top + d.padding.top;
    }

    /// Lay out child boxes in block formatting context.
    fn layout_block_children(&mut self) {
        let mut y_cursor = self.dimensions.content.y;

        for child in &mut self.children {
            // Create containing block for child
            let child_containing = Dimensions {
                content: Rect {
                    x: self.dimensions.content.x,
                    y: y_cursor,
                    width: self.dimensions.content.width,
                    height: 0.0, // Not used for block children
                },
                ..Default::default()
            };

            // Layout child
            child.position_at_y(y_cursor, &child_containing);
            child.layout_block_children();
            child.calculate_block_height(&child_containing);

            // Move cursor down
            y_cursor = child.dimensions.margin_box().bottom();
        }
    }

    /// Calculate the height of a block-level box.
    fn calculate_block_height(&mut self, containing_block: &Dimensions) {
        // If height is explicitly set (not auto), use it
        // For now, we always use auto height (shrink to fit content)
        let _ = containing_block; // May be used for percentage heights

        if self.children.is_empty() {
            // No children: height is 0 (unless explicitly set)
            // Could also be based on text content
            if self.text_content.is_some() {
                // Placeholder: assume single line of text at 16px
                self.dimensions.content.height = 16.0;
            }
        } else {
            // Height is distance from top to bottom of last child's margin box
            if let Some(last_child) = self.children.last() {
                let content_top = self.dimensions.content.y;
                let content_bottom = last_child.dimensions.margin_box().bottom();
                self.dimensions.content.height = content_bottom - content_top;
            }
        }
    }

    /// Calculate height with explicit style.
    pub fn calculate_block_height_with_style(
        &mut self,
        containing_block: &Dimensions,
        style: &ComputedStyle,
    ) {
        if let Some(height) = style.height.to_px(containing_block.content.height) {
            self.dimensions.content.height = height;
        } else {
            // Auto height
            self.calculate_block_height(containing_block);
        }
    }
}

/// Collapse vertical margins between adjacent block boxes.
///
/// Returns the collapsed margin value.
pub fn collapse_margins(margin_bottom: f32, margin_top: f32) -> f32 {
    // Both positive: take the larger
    // Both negative: take the more negative (smaller)
    // One of each: add them together
    if margin_bottom >= 0.0 && margin_top >= 0.0 {
        margin_bottom.max(margin_top)
    } else if margin_bottom < 0.0 && margin_top < 0.0 {
        margin_bottom.min(margin_top)
    } else {
        margin_bottom + margin_top
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport(width: f32, height: f32) -> Dimensions {
        Dimensions::from_viewport(width, height)
    }

    #[test]
    fn test_block_width_auto() {
        let mut layout_box = LayoutBox::block();
        let containing = viewport(800.0, 600.0);

        layout_box.layout_block(&containing);

        // Auto width should fill the containing block
        assert_eq!(layout_box.dimensions.content.width, 800.0);
    }

    #[test]
    fn test_block_width_with_margins() {
        let mut layout_box = LayoutBox::block();
        layout_box.dimensions.margin = Edges::new(0.0, 20.0, 0.0, 20.0);

        let containing = viewport(800.0, 600.0);
        layout_box.layout_block(&containing);

        // Width should be containing width minus margins
        assert_eq!(layout_box.dimensions.content.width, 760.0);
    }

    #[test]
    fn test_block_width_explicit() {
        let mut layout_box = LayoutBox::block();
        let containing = viewport(800.0, 600.0);

        let style = ComputedStyle {
            width: Length::Px(400.0),
            margin_left: Length::Auto,
            margin_right: Length::Auto,
            ..Default::default()
        };

        layout_box.calculate_block_width_with_style(&containing, &style);

        // Width should be 400px
        assert_eq!(layout_box.dimensions.content.width, 400.0);
        // Margins should be centered (200px each)
        assert_eq!(layout_box.dimensions.margin.left, 200.0);
        assert_eq!(layout_box.dimensions.margin.right, 200.0);
    }

    #[test]
    fn test_block_height_auto_with_children() {
        let mut parent = LayoutBox::block();
        parent.dimensions.content = Rect::new(0.0, 0.0, 800.0, 0.0);

        let mut child1 = LayoutBox::block();
        child1.dimensions.content = Rect::new(0.0, 0.0, 800.0, 100.0);

        let mut child2 = LayoutBox::block();
        child2.dimensions.content = Rect::new(0.0, 100.0, 800.0, 50.0);

        parent.children = vec![child1, child2];

        let containing = viewport(800.0, 600.0);
        parent.layout_block(&containing);

        // Height should contain both children
        assert!(parent.dimensions.content.height >= 150.0);
    }

    #[test]
    fn test_margin_collapsing() {
        // Both positive: take larger
        assert_eq!(collapse_margins(20.0, 30.0), 30.0);
        assert_eq!(collapse_margins(30.0, 20.0), 30.0);

        // Both negative: take more negative
        assert_eq!(collapse_margins(-20.0, -30.0), -30.0);

        // Mixed: add them
        assert_eq!(collapse_margins(20.0, -10.0), 10.0);
        assert_eq!(collapse_margins(-10.0, 20.0), 10.0);
    }

    #[test]
    fn test_block_position() {
        let mut layout_box = LayoutBox::block();
        layout_box.dimensions.margin = Edges::new(10.0, 20.0, 30.0, 40.0);
        layout_box.dimensions.border = Edges::new(1.0, 1.0, 1.0, 1.0);
        layout_box.dimensions.padding = Edges::new(5.0, 5.0, 5.0, 5.0);

        let containing = viewport(800.0, 600.0);
        layout_box.layout_block(&containing);

        // x = margin-left + border-left + padding-left
        assert_eq!(layout_box.dimensions.content.x, 40.0 + 1.0 + 5.0);
        // y = margin-top + border-top + padding-top
        assert_eq!(layout_box.dimensions.content.y, 10.0 + 1.0 + 5.0);
    }
}
