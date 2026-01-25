//! Render tree for painting.
//!
//! The RenderTree holds the laid-out boxes ready for painting.
//! It combines the layout tree with display list generation.

use vw_gfx::Framebuffer;
use vw_layout::LayoutBox;

/// A render tree ready for painting.
///
/// The RenderTree contains the result of layout: positioned boxes
/// with computed dimensions. It can be painted to a framebuffer.
pub struct RenderTree {
    /// The root layout box (if any).
    root: Option<LayoutBox>,
    /// Viewport width used during layout.
    viewport_width: u32,
    /// Viewport height used during layout.
    viewport_height: u32,
}

impl RenderTree {
    /// Create a new render tree from a layout root.
    pub fn new(root: Option<LayoutBox>, viewport_width: u32, viewport_height: u32) -> Self {
        RenderTree {
            root,
            viewport_width,
            viewport_height,
        }
    }

    /// Get the viewport width used during layout.
    pub fn viewport_width(&self) -> u32 {
        self.viewport_width
    }

    /// Get the viewport height used during layout.
    pub fn viewport_height(&self) -> u32 {
        self.viewport_height
    }

    /// Get a reference to the root layout box.
    pub fn root(&self) -> Option<&LayoutBox> {
        self.root.as_ref()
    }

    /// Paint this render tree into a framebuffer.
    ///
    /// This walks the layout tree and paints each box's background
    /// and content into the framebuffer.
    pub fn paint(&self, fb: &mut Framebuffer) {
        crate::paint::paint_render_tree(self, fb);
    }
}

impl std::fmt::Debug for RenderTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderTree")
            .field("viewport_width", &self.viewport_width)
            .field("viewport_height", &self.viewport_height)
            .field("has_root", &self.root.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_render_tree() {
        let tree = RenderTree::new(None, 800, 600);

        assert_eq!(tree.viewport_width(), 800);
        assert_eq!(tree.viewport_height(), 600);
        assert!(tree.root().is_none());
    }

    #[test]
    fn test_render_tree_paint() {
        let tree = RenderTree::new(None, 100, 100);
        let mut fb = Framebuffer::new(100, 100);

        tree.paint(&mut fb);

        // Should not panic with empty tree
        assert_eq!(fb.width(), 100);
    }
}
