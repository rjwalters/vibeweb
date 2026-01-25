//! Text rasterization for software rendering
//!
//! This module provides text rasterization capabilities using `ab_glyph` for
//! glyph outline rendering. It integrates with the existing `Framebuffer` and
//! `Color` types to draw anti-aliased text.
//!
//! # Example
//!
//! ```no_run
//! use vw_gfx::{Color, Framebuffer, TextRenderer};
//! use vw_fonts::FontCache;
//!
//! let mut fb = Framebuffer::new(800, 600);
//! fb.clear(Color::WHITE);
//!
//! let mut fonts = FontCache::new();
//! if let Some(font) = fonts.get_fallback() {
//!     let mut renderer = TextRenderer::new(&mut fb);
//!     // Use font.as_ab_glyph() to get the underlying ab_glyph font
//!     renderer.draw_text(font.as_ab_glyph(), 16.0, 50.0, 100.0, "Hello, World!", Color::BLACK);
//! }
//! ```

use ab_glyph::{point, Font, OutlinedGlyph, ScaleFont};

use crate::color::Color;
use crate::framebuffer::Framebuffer;

/// Text renderer that draws glyphs to a framebuffer.
///
/// The `TextRenderer` wraps a mutable reference to a `Framebuffer` and provides
/// methods for drawing anti-aliased text using glyph outlines from `ab_glyph`.
pub struct TextRenderer<'a> {
    framebuffer: &'a mut Framebuffer,
}

impl<'a> TextRenderer<'a> {
    /// Create a new text renderer targeting the given framebuffer.
    pub fn new(framebuffer: &'a mut Framebuffer) -> Self {
        Self { framebuffer }
    }

    /// Draw text at the given baseline position.
    ///
    /// The position (x, y) represents the start of the text baseline, not the
    /// top-left corner. The baseline is the line on which most letters "sit".
    ///
    /// # Arguments
    ///
    /// * `font` - The font to use for rendering (implements `ab_glyph::Font`)
    /// * `size` - Font size in pixels
    /// * `x` - X coordinate of the baseline start
    /// * `y` - Y coordinate of the baseline
    /// * `text` - The text string to render
    /// * `color` - The color to render the text in
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use vw_gfx::{Color, Framebuffer, TextRenderer};
    /// # use vw_fonts::FontCache;
    /// # let mut fb = Framebuffer::new(100, 100);
    /// # let mut fonts = FontCache::new();
    /// # if let Some(font) = fonts.get_fallback() {
    /// let mut renderer = TextRenderer::new(&mut fb);
    /// renderer.draw_text(font.as_ab_glyph(), 24.0, 10.0, 50.0, "Hello!", Color::BLACK);
    /// # }
    /// ```
    pub fn draw_text<F: Font>(
        &mut self,
        font: &F,
        size: f32,
        x: f32,
        y: f32,
        text: &str,
        color: Color,
    ) {
        let scaled = font.as_scaled(size);
        let mut cursor_x = x;

        for c in text.chars() {
            let glyph_id = scaled.glyph_id(c);
            let glyph = glyph_id.with_scale_and_position(size, point(cursor_x, y));

            if let Some(outlined) = font.outline_glyph(glyph) {
                self.draw_outlined_glyph(&outlined, color);
            }

            cursor_x += scaled.h_advance(glyph_id);
        }
    }

    /// Draw text with kerning support for improved letter spacing.
    ///
    /// This method takes kerning pairs into account, which can result in
    /// more visually pleasing text for certain character combinations
    /// (e.g., "AV", "To").
    ///
    /// # Arguments
    ///
    /// Same as `draw_text`.
    pub fn draw_text_with_kerning<F: Font>(
        &mut self,
        font: &F,
        size: f32,
        x: f32,
        y: f32,
        text: &str,
        color: Color,
    ) {
        let scaled = font.as_scaled(size);
        let mut cursor_x = x;
        let mut prev_glyph_id = None;

        for c in text.chars() {
            let glyph_id = scaled.glyph_id(c);

            // Apply kerning if we have a previous glyph
            if let Some(prev) = prev_glyph_id {
                cursor_x += scaled.kern(prev, glyph_id);
            }

            let glyph = glyph_id.with_scale_and_position(size, point(cursor_x, y));

            if let Some(outlined) = font.outline_glyph(glyph) {
                self.draw_outlined_glyph(&outlined, color);
            }

            cursor_x += scaled.h_advance(glyph_id);
            prev_glyph_id = Some(glyph_id);
        }
    }

    /// Draw an outlined glyph to the framebuffer.
    ///
    /// This is the core rasterization function that converts glyph outlines
    /// to pixels with anti-aliasing based on coverage values.
    fn draw_outlined_glyph(&mut self, outlined: &OutlinedGlyph, color: Color) {
        let bounds = outlined.px_bounds();

        outlined.draw(|px, py, coverage| {
            // Calculate absolute pixel position
            let abs_x = bounds.min.x as i32 + px as i32;
            let abs_y = bounds.min.y as i32 + py as i32;

            // Skip pixels outside framebuffer bounds
            if abs_x < 0 || abs_y < 0 {
                return;
            }

            let abs_x = abs_x as u32;
            let abs_y = abs_y as u32;

            if abs_x >= self.framebuffer.width() || abs_y >= self.framebuffer.height() {
                return;
            }

            // Convert coverage (0.0 - 1.0) to alpha (0 - 255)
            let alpha = (coverage * 255.0) as u8;

            if alpha == 0 {
                return;
            }

            // Create source color with coverage-based alpha
            let src = color.with_alpha(alpha);

            if alpha == 255 {
                // Fully opaque - direct write
                self.framebuffer.set_pixel(abs_x, abs_y, color);
            } else {
                // Semi-transparent - alpha blend
                let dst = self.framebuffer.get_pixel(abs_x, abs_y);
                let blended = src.blend_over(dst);
                self.framebuffer.set_pixel(abs_x, abs_y, blended);
            }
        });
    }

    /// Get the width of the underlying framebuffer.
    pub fn width(&self) -> u32 {
        self.framebuffer.width()
    }

    /// Get the height of the underlying framebuffer.
    pub fn height(&self) -> u32 {
        self.framebuffer.height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vw_fonts::FontCache;

    fn get_test_font_cache() -> FontCache {
        FontCache::new()
    }

    #[test]
    fn test_text_renderer_creation() {
        let mut fb = Framebuffer::new(100, 100);
        let renderer = TextRenderer::new(&mut fb);
        assert_eq!(renderer.width(), 100);
        assert_eq!(renderer.height(), 100);
    }

    #[test]
    fn test_draw_text_basic() {
        let mut cache = get_test_font_cache();
        if let Some(font) = cache.get_fallback() {
            let mut fb = Framebuffer::new(200, 100);
            fb.clear(Color::WHITE);

            {
                let mut renderer = TextRenderer::new(&mut fb);
                renderer.draw_text(font.as_ab_glyph(), 16.0, 10.0, 50.0, "Hello", Color::BLACK);
            }

            // Just verify framebuffer is valid after render
            assert_eq!(fb.width(), 200);
        }
    }

    #[test]
    fn test_draw_text_changes_pixels() {
        let mut cache = get_test_font_cache();
        if let Some(font) = cache.get_fallback() {
            let mut fb = Framebuffer::new(200, 100);
            fb.clear(Color::WHITE);

            // Store initial pixel count of white pixels
            let initial_white = fb
                .pixels()
                .iter()
                .filter(|&&p| p == Color::WHITE.to_argb())
                .count();

            // The framebuffer should start all white
            assert_eq!(initial_white, 20000);

            {
                let mut renderer = TextRenderer::new(&mut fb);
                renderer.draw_text(font.as_ab_glyph(), 24.0, 10.0, 50.0, "Test", Color::BLACK);
            }

            // After drawing, some pixels should have changed
            let final_white = fb
                .pixels()
                .iter()
                .filter(|&&p| p == Color::WHITE.to_argb())
                .count();

            assert!(
                final_white < initial_white,
                "Drawing text should change some pixels"
            );
        }
    }

    #[test]
    fn test_empty_text() {
        let mut cache = get_test_font_cache();
        if let Some(font) = cache.get_fallback() {
            let mut fb = Framebuffer::new(100, 100);
            fb.clear(Color::WHITE);

            let initial_pixels = fb.pixels().to_vec();

            {
                let mut renderer = TextRenderer::new(&mut fb);
                renderer.draw_text(font.as_ab_glyph(), 16.0, 10.0, 50.0, "", Color::BLACK);
            }

            // Verify framebuffer unchanged for empty text
            assert_eq!(fb.pixels(), initial_pixels.as_slice());
        }
    }

    #[test]
    fn test_text_with_kerning() {
        let mut cache = get_test_font_cache();
        if let Some(font) = cache.get_fallback() {
            let mut fb = Framebuffer::new(200, 100);
            fb.clear(Color::WHITE);

            {
                let mut renderer = TextRenderer::new(&mut fb);
                // AV is a common kerning pair
                renderer.draw_text_with_kerning(
                    font.as_ab_glyph(),
                    24.0,
                    10.0,
                    50.0,
                    "AVATAR",
                    Color::BLACK,
                );
            }

            // Verify text was drawn (some pixels changed)
            let white_count = fb
                .pixels()
                .iter()
                .filter(|&&p| p == Color::WHITE.to_argb())
                .count();

            assert!(
                white_count < 20000,
                "Drawing text with kerning should change some pixels"
            );
        }
    }

    #[test]
    fn test_colored_text() {
        let mut cache = get_test_font_cache();
        if let Some(font) = cache.get_fallback() {
            let mut fb = Framebuffer::new(200, 100);
            fb.clear(Color::WHITE);

            {
                let mut renderer = TextRenderer::new(&mut fb);
                renderer.draw_text(font.as_ab_glyph(), 24.0, 10.0, 50.0, "Red", Color::RED);
            }

            // Check that some pixels are now red or red-tinted (anti-aliased)
            let has_red = fb.pixels().iter().any(|&p| {
                let color = Color::from_argb(p);
                color.r > color.g && color.r > color.b && color.r > 100
            });

            assert!(has_red, "Drawing red text should produce red pixels");
        }
    }

    #[test]
    fn test_text_out_of_bounds() {
        let mut cache = get_test_font_cache();
        if let Some(font) = cache.get_fallback() {
            // Text drawn completely outside framebuffer should not panic
            let mut fb = Framebuffer::new(10, 10);
            fb.clear(Color::WHITE);

            {
                let mut renderer = TextRenderer::new(&mut fb);
                // Draw text way outside the framebuffer
                renderer.draw_text(
                    font.as_ab_glyph(),
                    16.0,
                    1000.0,
                    1000.0,
                    "Outside",
                    Color::BLACK,
                );
            }

            assert_eq!(fb.width(), 10);
        }
    }

    #[test]
    fn test_text_partially_clipped() {
        let mut cache = get_test_font_cache();
        if let Some(font) = cache.get_fallback() {
            let mut fb = Framebuffer::new(50, 50);
            fb.clear(Color::WHITE);

            {
                let mut renderer = TextRenderer::new(&mut fb);
                // Draw text that starts inside but extends outside
                renderer.draw_text(
                    font.as_ab_glyph(),
                    48.0,
                    20.0,
                    30.0,
                    "This is a long text",
                    Color::BLACK,
                );
            }

            // Should not panic and some pixels should be changed
            let white_count = fb
                .pixels()
                .iter()
                .filter(|&&p| p == Color::WHITE.to_argb())
                .count();

            assert!(
                white_count < 2500,
                "Partially clipped text should still be drawn"
            );
        }
    }

    #[test]
    fn test_multiple_font_sizes() {
        let mut cache = get_test_font_cache();
        if let Some(font) = cache.get_fallback() {
            let mut fb = Framebuffer::new(400, 200);
            fb.clear(Color::WHITE);

            {
                let mut renderer = TextRenderer::new(&mut fb);
                // Draw at various sizes
                renderer.draw_text(font.as_ab_glyph(), 8.0, 10.0, 20.0, "Small", Color::BLACK);
                renderer.draw_text(font.as_ab_glyph(), 16.0, 10.0, 50.0, "Medium", Color::BLACK);
                renderer.draw_text(font.as_ab_glyph(), 32.0, 10.0, 100.0, "Large", Color::BLACK);
                renderer.draw_text(font.as_ab_glyph(), 64.0, 10.0, 170.0, "Huge", Color::BLACK);
            }

            // All sizes should render without issues
            let white_count = fb
                .pixels()
                .iter()
                .filter(|&&p| p == Color::WHITE.to_argb())
                .count();

            assert!(
                white_count < 80000,
                "Multiple text sizes should all be rendered"
            );
        }
    }
}
