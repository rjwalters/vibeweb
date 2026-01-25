//! Viewport dimensions for rendering.

/// Viewport dimensions in pixels.
///
/// The viewport represents the visible area where content will be rendered.
/// It determines the containing block for layout calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    width: u32,
    height: u32,
}

impl Viewport {
    /// Create a new viewport with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Get the viewport width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the viewport height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Convert to vw_layout::Viewport for layout calculations.
    pub(crate) fn to_layout_viewport(self) -> vw_layout::Viewport {
        vw_layout::Viewport::new(self.width as f32, self.height as f32)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_creation() {
        let vp = Viewport::new(1024, 768);
        assert_eq!(vp.width(), 1024);
        assert_eq!(vp.height(), 768);
    }

    #[test]
    fn test_viewport_default() {
        let vp = Viewport::default();
        assert_eq!(vp.width(), 800);
        assert_eq!(vp.height(), 600);
    }

    #[test]
    fn test_to_layout_viewport() {
        let vp = Viewport::new(800, 600);
        let layout_vp = vp.to_layout_viewport();
        assert_eq!(layout_vp.width, 800.0);
        assert_eq!(layout_vp.height, 600.0);
    }
}
