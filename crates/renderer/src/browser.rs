//! Browser type for coordinating the render pipeline.
//!
//! The `Browser` type orchestrates the full rendering lifecycle from
//! HTML/CSS loading through painting. It manages document state and
//! coordinates layout invalidation and repainting.

use crate::document::Document;
use crate::error::Result;
use crate::render_tree::RenderTree;
use crate::viewport::Viewport;
use vw_gfx::Framebuffer;

/// Main render loop coordinator.
///
/// The Browser holds the loaded document and viewport state. It coordinates
/// the render pipeline, triggering layout and painting when needed.
///
/// # Example
///
/// ```
/// use vw_renderer::Browser;
/// use vw_gfx::Framebuffer;
///
/// let mut browser = Browser::new(
///     "<html><body><p>Hello, World!</p></body></html>",
///     "",
///     800,
///     600
/// ).expect("Failed to create browser");
///
/// let mut fb = Framebuffer::new(800, 600);
/// browser.paint(&mut fb);
/// ```
pub struct Browser {
    /// The loaded document.
    document: Document,
    /// Current viewport dimensions.
    viewport: Viewport,
    /// Cached render tree (invalidated on viewport change or DOM mutation).
    render_tree: Option<RenderTree>,
}

impl Browser {
    /// Create a new Browser with the given HTML and CSS.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML source to render
    /// * `css` - The CSS source (can be empty)
    /// * `width` - Initial viewport width
    /// * `height` - Initial viewport height
    ///
    /// # Returns
    ///
    /// A Browser ready for rendering, or an error if parsing fails.
    pub fn new(html: &str, css: &str, width: u32, height: u32) -> Result<Self> {
        let document = Document::load(html, css)?;
        let viewport = Viewport::new(width, height);

        Ok(Browser {
            document,
            viewport,
            render_tree: None,
        })
    }

    /// Get the current viewport.
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Resize the viewport.
    ///
    /// This invalidates the render tree, causing a re-layout on the next paint.
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.viewport.width() != width || self.viewport.height() != height {
            self.viewport = Viewport::new(width, height);
            self.invalidate();
        }
    }

    /// Invalidate the render tree.
    ///
    /// Call this when the document structure changes (e.g., DOM mutations)
    /// to force a re-layout on the next paint.
    pub fn invalidate(&mut self) {
        self.render_tree = None;
    }

    /// Ensure the render tree is up to date.
    ///
    /// This performs layout if the render tree has been invalidated.
    fn ensure_render_tree(&mut self) {
        if self.render_tree.is_none() {
            self.render_tree = Some(self.document.render_tree(&self.viewport));
        }
    }

    /// Paint the current document into a framebuffer.
    ///
    /// This ensures the render tree is up to date, then paints it.
    pub fn paint(&mut self, fb: &mut Framebuffer) {
        // Ensure framebuffer matches viewport
        if fb.width() != self.viewport.width() || fb.height() != self.viewport.height() {
            fb.resize(self.viewport.width(), self.viewport.height());
        }

        // Ensure layout is computed
        self.ensure_render_tree();

        // Paint the render tree
        if let Some(ref tree) = self.render_tree {
            tree.paint(fb);
        }
    }

    /// Get a reference to the underlying document.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Get the render tree if available.
    ///
    /// Returns `None` if the render tree has been invalidated.
    pub fn render_tree(&self) -> Option<&RenderTree> {
        self.render_tree.as_ref()
    }
}

impl std::fmt::Debug for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Browser")
            .field("viewport", &self.viewport)
            .field("has_render_tree", &self.render_tree.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_creation() {
        let browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();

        assert_eq!(browser.viewport().width(), 800);
        assert_eq!(browser.viewport().height(), 600);
    }

    #[test]
    fn test_browser_paint() {
        let mut browser = Browser::new("<html><body><p>Hello</p></body></html>", "", 800, 600).unwrap();
        let mut fb = Framebuffer::new(800, 600);

        browser.paint(&mut fb);

        // After paint, render tree should be cached
        assert!(browser.render_tree().is_some());
    }

    #[test]
    fn test_browser_resize() {
        let mut browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();
        let mut fb = Framebuffer::new(800, 600);

        // First paint
        browser.paint(&mut fb);
        assert!(browser.render_tree().is_some());

        // Resize invalidates render tree
        browser.resize(1024, 768);
        assert!(browser.render_tree().is_none());
        assert_eq!(browser.viewport().width(), 1024);
        assert_eq!(browser.viewport().height(), 768);

        // Paint at new size
        fb.resize(1024, 768);
        browser.paint(&mut fb);
        assert!(browser.render_tree().is_some());
    }

    #[test]
    fn test_browser_invalidate() {
        let mut browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();
        let mut fb = Framebuffer::new(800, 600);

        browser.paint(&mut fb);
        assert!(browser.render_tree().is_some());

        browser.invalidate();
        assert!(browser.render_tree().is_none());
    }

    #[test]
    fn test_same_size_resize_no_invalidate() {
        let mut browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();
        let mut fb = Framebuffer::new(800, 600);

        browser.paint(&mut fb);
        assert!(browser.render_tree().is_some());

        // Resize to same size should not invalidate
        browser.resize(800, 600);
        assert!(browser.render_tree().is_some());
    }
}
