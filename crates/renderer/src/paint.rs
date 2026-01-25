//! Painting logic for render trees.
//!
//! This module implements the painting phase of the rendering pipeline,
//! converting layout boxes into pixels in a framebuffer.

use crate::render_tree::RenderTree;
use vw_gfx::{Color, Framebuffer, Rect};
use vw_layout::{BoxType, LayoutBox, Rect as LayoutRect};

/// Paint a render tree into a framebuffer with scroll offset.
///
/// This clears the framebuffer to white, then walks the layout tree
/// painting each box's background. Box coordinates are translated by
/// the scroll offset to achieve scrolling.
///
/// # Arguments
///
/// * `tree` - The render tree to paint
/// * `fb` - The framebuffer to paint into
/// * `scroll_x` - Horizontal scroll offset (content is shifted left)
/// * `scroll_y` - Vertical scroll offset (content is shifted up)
pub fn paint_render_tree(tree: &RenderTree, fb: &mut Framebuffer, scroll_x: f32, scroll_y: f32) {
    // Clear to white background
    fb.clear(Color::WHITE);

    // Paint the layout tree if present
    if let Some(root) = tree.root() {
        paint_layout_box(root, fb, scroll_x, scroll_y);
    }
}

/// Paint a single layout box and its children with scroll offset.
fn paint_layout_box(layout_box: &LayoutBox, fb: &mut Framebuffer, scroll_x: f32, scroll_y: f32) {
    // Skip boxes that don't generate content
    match layout_box.box_type {
        BoxType::Block | BoxType::InlineBlock | BoxType::Anonymous => {
            paint_box_background(layout_box, fb, scroll_x, scroll_y);
        }
        BoxType::Inline => {
            // Inline boxes are painted as part of their line boxes
            // For now, we still paint their backgrounds
            paint_box_background(layout_box, fb, scroll_x, scroll_y);
        }
    }

    // Paint children
    for child in &layout_box.children {
        paint_layout_box(child, fb, scroll_x, scroll_y);
    }
}

/// Paint the background of a layout box with scroll offset applied.
fn paint_box_background(
    layout_box: &LayoutBox,
    _fb: &mut Framebuffer,
    scroll_x: f32,
    scroll_y: f32,
) {
    // Use the layout box's border_box() method
    let border_box = layout_box.border_box();

    // Apply scroll offset to create translated coordinates
    let translated_rect = LayoutRect {
        x: border_box.x - scroll_x,
        y: border_box.y - scroll_y,
        width: border_box.width,
        height: border_box.height,
    };

    // Convert layout Rect to gfx Rect
    let rect = layout_rect_to_gfx_rect(&translated_rect);

    // For now, we only ensure the tree is walked correctly.
    // Background colors will be added when computed styles are available.
    // The rect is computed but not yet drawn to avoid painting
    // everything with a default color.
    let _ = rect;
}

/// Convert a layout Rect to a gfx Rect.
fn layout_rect_to_gfx_rect(layout_rect: &LayoutRect) -> Rect {
    Rect::new(
        layout_rect.x as i32,
        layout_rect.y as i32,
        layout_rect.width.max(0.0) as u32,
        layout_rect.height.max(0.0) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use vw_layout::Dimensions;

    fn create_test_box(x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
        LayoutBox {
            box_type: BoxType::Block,
            dimensions: Dimensions {
                content: super::LayoutRect {
                    x,
                    y,
                    width,
                    height,
                },
                ..Default::default()
            },
            children: Vec::new(),
            node_id: None,
            text_content: None,
        }
    }

    #[test]
    fn test_paint_empty_tree() {
        let tree = RenderTree::new(None, 100, 100);
        let mut fb = Framebuffer::new(100, 100);

        paint_render_tree(&tree, &mut fb, 0.0, 0.0);

        // Should be cleared to white
        assert_eq!(fb.get_pixel(0, 0), Color::WHITE);
        assert_eq!(fb.get_pixel(50, 50), Color::WHITE);
    }

    #[test]
    fn test_paint_single_box() {
        let root = create_test_box(0.0, 0.0, 100.0, 100.0);
        let tree = RenderTree::new(Some(root), 100, 100);
        let mut fb = Framebuffer::new(100, 100);

        paint_render_tree(&tree, &mut fb, 0.0, 0.0);

        // Should not panic
        assert_eq!(fb.width(), 100);
    }

    #[test]
    fn test_paint_nested_boxes() {
        let child = create_test_box(10.0, 10.0, 50.0, 50.0);
        let mut root = create_test_box(0.0, 0.0, 100.0, 100.0);
        root.children.push(child);

        let tree = RenderTree::new(Some(root), 100, 100);
        let mut fb = Framebuffer::new(100, 100);

        paint_render_tree(&tree, &mut fb, 0.0, 0.0);

        // Should not panic with nested boxes
        assert_eq!(fb.width(), 100);
    }

    #[test]
    fn test_paint_with_scroll_offset() {
        let root = create_test_box(0.0, 0.0, 100.0, 200.0);
        let tree = RenderTree::new(Some(root), 100, 100);
        let mut fb = Framebuffer::new(100, 100);

        // Paint with vertical scroll offset
        paint_render_tree(&tree, &mut fb, 0.0, 50.0);

        // Should not panic with scroll offset
        assert_eq!(fb.width(), 100);
    }
}
