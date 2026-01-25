//! CPU-based framebuffer for software rendering

use crate::color::Color;
use crate::rect::Rect;

/// A CPU-based framebuffer for software rendering
///
/// Stores pixels in ARGB format for compatibility with softbuffer.
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

impl Framebuffer {
    /// Create a new framebuffer with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            pixels: vec![0; size],
        }
    }

    /// Get the framebuffer width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the framebuffer height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the pixel buffer as a slice (ARGB format)
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Resize the framebuffer to new dimensions
    ///
    /// This clears the framebuffer to black.
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        let size = (width as usize) * (height as usize);
        self.pixels.resize(size, 0);
        self.pixels.fill(0);
    }

    /// Clear the entire framebuffer to a color
    pub fn clear(&mut self, color: Color) {
        let argb = color.to_argb();
        self.pixels.fill(argb);
    }

    /// Fill a rectangle with a color
    ///
    /// The rectangle is clipped to the framebuffer bounds.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        if rect.is_empty() || self.width == 0 || self.height == 0 {
            return;
        }

        // Clip rectangle to framebuffer bounds
        let x_start = rect.x.max(0) as u32;
        let y_start = rect.y.max(0) as u32;
        let x_end = (rect.right().max(0) as u32).min(self.width);
        let y_end = (rect.bottom().max(0) as u32).min(self.height);

        if x_start >= x_end || y_start >= y_end {
            return;
        }

        let argb = color.to_argb();

        for y in y_start..y_end {
            let row_start = (y * self.width + x_start) as usize;
            let row_end = (y * self.width + x_end) as usize;
            self.pixels[row_start..row_end].fill(argb);
        }
    }

    /// Set a single pixel (bounds checked)
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.pixels[idx] = color.to_argb();
        }
    }

    /// Get a single pixel (bounds checked, returns transparent if out of bounds)
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            Color::from_argb(self.pixels[idx])
        } else {
            Color::TRANSPARENT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framebuffer_new() {
        let fb = Framebuffer::new(100, 50);
        assert_eq!(fb.width(), 100);
        assert_eq!(fb.height(), 50);
        assert_eq!(fb.pixels().len(), 5000);
    }

    #[test]
    fn framebuffer_clear() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::RED);
        assert!(fb.pixels().iter().all(|&p| p == Color::RED.to_argb()));
    }

    #[test]
    fn framebuffer_fill_rect() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE);
        fb.fill_rect(Rect::new(2, 2, 4, 4), Color::BLUE);

        // Check corner pixels
        assert_eq!(fb.get_pixel(1, 1), Color::WHITE);
        assert_eq!(fb.get_pixel(2, 2), Color::BLUE);
        assert_eq!(fb.get_pixel(5, 5), Color::BLUE);
        assert_eq!(fb.get_pixel(6, 6), Color::WHITE);
    }

    #[test]
    fn framebuffer_fill_rect_clipping() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE);

        // Rectangle extends past bounds
        fb.fill_rect(Rect::new(-2, -2, 5, 5), Color::RED);

        // Check that visible part is red
        assert_eq!(fb.get_pixel(0, 0), Color::RED);
        assert_eq!(fb.get_pixel(2, 2), Color::RED);
        assert_eq!(fb.get_pixel(3, 3), Color::WHITE);
    }

    #[test]
    fn framebuffer_resize() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::RED);
        fb.resize(20, 20);
        assert_eq!(fb.width(), 20);
        assert_eq!(fb.height(), 20);
        // After resize, buffer is cleared to black
        assert_eq!(fb.get_pixel(0, 0), Color::from_argb(0));
    }

    #[test]
    fn framebuffer_set_get_pixel() {
        let mut fb = Framebuffer::new(10, 10);
        fb.set_pixel(5, 5, Color::GREEN);
        assert_eq!(fb.get_pixel(5, 5), Color::GREEN);
        assert_eq!(fb.get_pixel(0, 0), Color::from_argb(0)); // Unset pixel
    }

    #[test]
    fn framebuffer_bounds_check() {
        let mut fb = Framebuffer::new(10, 10);
        // Setting out of bounds should do nothing (not panic)
        fb.set_pixel(100, 100, Color::RED);
        // Getting out of bounds should return transparent
        assert_eq!(fb.get_pixel(100, 100), Color::TRANSPARENT);
    }
}
