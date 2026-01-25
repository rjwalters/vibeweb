//! Rectangle type for graphics primitives

/// A rectangle with position and size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    /// X position (can be negative)
    pub x: i32,
    /// Y position (can be negative)
    pub y: i32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl Rect {
    /// Create a new rectangle
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rectangle from position and size
    pub const fn from_xywh(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self::new(x, y, width, height)
    }

    /// Check if this rectangle is empty (zero width or height)
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Get the right edge (x + width)
    pub const fn right(&self) -> i32 {
        self.x.saturating_add(self.width as i32)
    }

    /// Get the bottom edge (y + height)
    pub const fn bottom(&self) -> i32 {
        self.y.saturating_add(self.height as i32)
    }

    /// Check if a point is inside this rectangle
    pub const fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_new() {
        let rect = Rect::new(10, 20, 100, 50);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, 50);
    }

    #[test]
    fn rect_edges() {
        let rect = Rect::new(10, 20, 100, 50);
        assert_eq!(rect.right(), 110);
        assert_eq!(rect.bottom(), 70);
    }

    #[test]
    fn rect_contains() {
        let rect = Rect::new(10, 20, 100, 50);
        // Inside
        assert!(rect.contains(10, 20));
        assert!(rect.contains(50, 40));
        assert!(rect.contains(109, 69));
        // Outside
        assert!(!rect.contains(9, 20));
        assert!(!rect.contains(10, 19));
        assert!(!rect.contains(110, 20));
        assert!(!rect.contains(10, 70));
    }

    #[test]
    fn rect_is_empty() {
        assert!(Rect::new(0, 0, 0, 10).is_empty());
        assert!(Rect::new(0, 0, 10, 0).is_empty());
        assert!(Rect::new(0, 0, 0, 0).is_empty());
        assert!(!Rect::new(0, 0, 1, 1).is_empty());
    }
}
