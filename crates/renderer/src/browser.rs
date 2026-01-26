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
    /// Horizontal scroll offset in document coordinates.
    scroll_x: f32,
    /// Vertical scroll offset in document coordinates.
    scroll_y: f32,
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
            scroll_x: 0.0,
            scroll_y: 0.0,
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

    /// Update scroll position from wheel delta.
    ///
    /// The scroll position is clamped to valid bounds based on content size.
    /// Positive delta_y scrolls content up (document moves up, revealing content below).
    ///
    /// # Arguments
    ///
    /// * `delta_x` - Horizontal scroll delta (positive = scroll right)
    /// * `delta_y` - Vertical scroll delta (positive = scroll down)
    pub fn scroll(&mut self, delta_x: f32, delta_y: f32) {
        // Get content height for clamping
        let content_height = self.content_height();
        let viewport_height = self.viewport.height() as f32;
        let max_scroll_y = (content_height - viewport_height).max(0.0);

        // Get content width for clamping (future: when horizontal overflow is supported)
        let content_width = self.content_width();
        let viewport_width = self.viewport.width() as f32;
        let max_scroll_x = (content_width - viewport_width).max(0.0);

        // Apply deltas and clamp
        self.scroll_y = (self.scroll_y + delta_y).clamp(0.0, max_scroll_y);
        self.scroll_x = (self.scroll_x + delta_x).clamp(0.0, max_scroll_x);
    }

    /// Get current scroll position.
    ///
    /// Returns the (x, y) scroll offset in document coordinates.
    pub fn scroll_position(&self) -> (f32, f32) {
        (self.scroll_x, self.scroll_y)
    }

    /// Set scroll position directly.
    ///
    /// This is useful for restoring scroll position from navigation history.
    /// The position is clamped to valid bounds.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal scroll offset
    /// * `y` - Vertical scroll offset
    pub fn set_scroll_position(&mut self, x: f32, y: f32) {
        // Get bounds for clamping
        let content_height = self.content_height();
        let viewport_height = self.viewport.height() as f32;
        let max_scroll_y = (content_height - viewport_height).max(0.0);

        let content_width = self.content_width();
        let viewport_width = self.viewport.width() as f32;
        let max_scroll_x = (content_width - viewport_width).max(0.0);

        // Clamp to valid bounds
        self.scroll_x = x.clamp(0.0, max_scroll_x);
        self.scroll_y = y.clamp(0.0, max_scroll_y);
    }

    /// Compute total content height from the render tree.
    ///
    /// Returns the bottom edge of the root layout box's border box,
    /// which represents the total document height.
    fn content_height(&self) -> f32 {
        self.render_tree
            .as_ref()
            .and_then(|tree| tree.root())
            .map(|root| root.border_box().bottom())
            .unwrap_or(0.0)
    }

    /// Compute total content width from the render tree.
    ///
    /// Returns the right edge of the root layout box's border box,
    /// which represents the total document width.
    fn content_width(&self) -> f32 {
        self.render_tree
            .as_ref()
            .and_then(|tree| tree.root())
            .map(|root| root.border_box().right())
            .unwrap_or(0.0)
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
    /// This ensures the render tree is up to date, then paints it
    /// with the current scroll offset applied.
    pub fn paint(&mut self, fb: &mut Framebuffer) {
        // Ensure framebuffer matches viewport
        if fb.width() != self.viewport.width() || fb.height() != self.viewport.height() {
            fb.resize(self.viewport.width(), self.viewport.height());
        }

        // Ensure layout is computed
        self.ensure_render_tree();

        // Paint the render tree with scroll offset
        if let Some(ref tree) = self.render_tree {
            tree.paint_with_scroll(fb, self.scroll_x, self.scroll_y);
        }
    }
    /// Navigate to a new document by loading new HTML and CSS.
    ///
    /// This replaces the current document, invalidates the render tree,
    /// and resets the scroll position to (0, 0).
    ///
    /// This method provides the foundation for future navigation features
    /// like link clicking, back/forward buttons, and reload.
    ///
    /// # Arguments
    ///
    /// * `html` - The new HTML source to render
    /// * `css` - The new CSS source (can be empty)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Navigate to new content (e.g., from a link click)
    /// browser.navigate("<html><body>New page</body></html>", "")?;
    /// ```
    pub fn navigate(&mut self, html: &str, css: &str) -> Result<()> {
        // Load the new document
        self.document = Document::load(html, css)?;

        // Invalidate render tree since we have a new document
        self.invalidate();

        // Reset scroll position on navigation
        self.scroll_x = 0.0;
        self.scroll_y = 0.0;

        Ok(())
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
            .field("scroll_x", &self.scroll_x)
            .field("scroll_y", &self.scroll_y)
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
        let mut browser =
            Browser::new("<html><body><p>Hello</p></body></html>", "", 800, 600).unwrap();
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

    #[test]
    fn test_scroll_initial_position() {
        let browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();

        // Initial scroll position should be (0, 0)
        assert_eq!(browser.scroll_position(), (0.0, 0.0));
    }

    #[test]
    fn test_scroll_updates_position() {
        let mut browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();
        let mut fb = Framebuffer::new(800, 600);

        // Paint to compute render tree (needed for content height)
        browser.paint(&mut fb);

        // Scroll by some delta
        browser.scroll(10.0, 20.0);

        // Position should be clamped at 0 since content doesn't exceed viewport
        // (content height is 0 with minimal document)
        let (x, y) = browser.scroll_position();
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_scroll_clamps_to_zero() {
        let mut browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();
        let mut fb = Framebuffer::new(800, 600);

        browser.paint(&mut fb);

        // Try to scroll to negative position
        browser.scroll(-100.0, -100.0);

        // Should be clamped to (0, 0)
        assert_eq!(browser.scroll_position(), (0.0, 0.0));
    }

    #[test]
    fn test_set_scroll_position() {
        let mut browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();
        let mut fb = Framebuffer::new(800, 600);

        browser.paint(&mut fb);

        // Set scroll position directly
        browser.set_scroll_position(50.0, 100.0);

        // Should be clamped (content doesn't exceed viewport)
        let (x, y) = browser.scroll_position();
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_set_scroll_position_clamps_negative() {
        let mut browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();
        let mut fb = Framebuffer::new(800, 600);

        browser.paint(&mut fb);

        // Try to set negative scroll position
        browser.set_scroll_position(-50.0, -100.0);

        // Should be clamped to (0, 0)
        assert_eq!(browser.scroll_position(), (0.0, 0.0));
    }

    #[test]
    fn test_paint_with_scroll() {
        let mut browser = Browser::new("<html><body>Test</body></html>", "", 800, 600).unwrap();
        let mut fb = Framebuffer::new(800, 600);

        // First paint
        browser.paint(&mut fb);

        // Scroll and repaint (should not panic)
        browser.scroll(0.0, 50.0);
        browser.paint(&mut fb);

        assert!(browser.render_tree().is_some());
    }
}

#[test]
fn test_navigate() {
    let mut browser = Browser::new("<html><body>Page 1</body></html>", "", 800, 600).unwrap();
    let mut fb = Framebuffer::new(800, 600);

    // Paint initial page
    browser.paint(&mut fb);
    assert!(browser.render_tree().is_some());

    // Scroll down a bit
    browser.set_scroll_position(0.0, 50.0);
    let (_x, y) = browser.scroll_position();
    assert_eq!(y, 0.0); // Clamped since content is small

    // Navigate to new page
    browser
        .navigate(
            "<html><body><h1>Page 2</h1><p>New content</p></body></html>",
            "",
        )
        .unwrap();

    // Render tree should be invalidated
    assert!(browser.render_tree().is_none());

    // Scroll should be reset to (0, 0)
    assert_eq!(browser.scroll_position(), (0.0, 0.0));

    // Should be able to paint new page
    browser.paint(&mut fb);
    assert!(browser.render_tree().is_some());
}
