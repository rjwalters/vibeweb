//! Font metrics abstraction for text layout.
//!
//! This module defines the interface for font measurement that layout needs.
//! The actual font rendering will be handled by `vw-fonts`, but layout only
//! needs to measure text dimensions.

/// Metrics for a measured text run.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextMetrics {
    /// Width of the text in pixels
    pub width: f32,
    /// Distance from baseline to top of tallest glyph
    pub ascent: f32,
    /// Distance from baseline to bottom of lowest glyph (positive value)
    pub descent: f32,
}

impl TextMetrics {
    /// Create new text metrics.
    pub fn new(width: f32, ascent: f32, descent: f32) -> Self {
        TextMetrics {
            width,
            ascent,
            descent,
        }
    }

    /// Total height (ascent + descent).
    pub fn height(&self) -> f32 {
        self.ascent + self.descent
    }
}

/// Trait for measuring text.
///
/// This trait should be implemented by the fonts crate to provide actual
/// font metrics. For testing, a fixed-width implementation is provided.
pub trait FontMetrics {
    /// Measure a text string with the given font properties.
    fn measure_text(&self, text: &str, font_size: f32, font_family: &str) -> TextMetrics;

    /// Get the line height for a font (typically ascent + descent + leading).
    fn line_height(&self, font_size: f32, font_family: &str) -> f32;
}

/// Fixed-width font metrics for testing.
///
/// This provides deterministic metrics based on simple calculations,
/// useful for unit testing layout without needing actual fonts.
#[derive(Debug, Clone)]
pub struct FixedFontMetrics {
    /// Width of each character (monospace assumption)
    pub char_width_ratio: f32,
    /// Ascent ratio (relative to font size)
    pub ascent_ratio: f32,
    /// Descent ratio (relative to font size)
    pub descent_ratio: f32,
    /// Line height ratio (relative to font size)
    pub line_height_ratio: f32,
}

impl Default for FixedFontMetrics {
    fn default() -> Self {
        // Typical ratios for a sans-serif font
        FixedFontMetrics {
            char_width_ratio: 0.6,  // Each char is 60% of font size wide
            ascent_ratio: 0.8,      // Ascent is 80% of font size
            descent_ratio: 0.2,     // Descent is 20% of font size
            line_height_ratio: 1.2, // Line height is 120% of font size
        }
    }
}

impl FontMetrics for FixedFontMetrics {
    fn measure_text(&self, text: &str, font_size: f32, _font_family: &str) -> TextMetrics {
        let char_count = text.chars().count() as f32;
        TextMetrics {
            width: char_count * font_size * self.char_width_ratio,
            ascent: font_size * self.ascent_ratio,
            descent: font_size * self.descent_ratio,
        }
    }

    fn line_height(&self, font_size: f32, _font_family: &str) -> f32 {
        font_size * self.line_height_ratio
    }
}

/// Font metrics that returns zero for all measurements.
/// Useful for testing layout logic without text considerations.
pub struct ZeroFontMetrics;

impl FontMetrics for ZeroFontMetrics {
    fn measure_text(&self, _text: &str, _font_size: f32, _font_family: &str) -> TextMetrics {
        TextMetrics::default()
    }

    fn line_height(&self, font_size: f32, _font_family: &str) -> f32 {
        font_size * 1.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_metrics_height() {
        let metrics = TextMetrics::new(100.0, 12.0, 4.0);
        assert_eq!(metrics.height(), 16.0);
    }

    #[test]
    fn test_fixed_font_metrics() {
        let fonts = FixedFontMetrics::default();
        let metrics = fonts.measure_text("Hello", 16.0, "sans-serif");

        // 5 chars * 16px * 0.6 = 48px width
        assert_eq!(metrics.width, 48.0);
        // 16px * 0.8 = 12.8px ascent
        assert!((metrics.ascent - 12.8).abs() < 0.01);
        // 16px * 0.2 = 3.2px descent
        assert!((metrics.descent - 3.2).abs() < 0.01);
    }

    #[test]
    fn test_line_height() {
        let fonts = FixedFontMetrics::default();
        let lh = fonts.line_height(16.0, "sans-serif");
        assert_eq!(lh, 19.2); // 16 * 1.2
    }
}
