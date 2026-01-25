//! Font loading and glyph metrics

use ab_glyph::{Font as AbFont, FontArc, GlyphId, ScaleFont};

/// A loaded font ready for measurement
#[derive(Clone)]
pub struct Font {
    inner: FontArc,
}

impl Font {
    /// Load font from raw data
    ///
    /// # Arguments
    /// * `data` - Raw font file data (TTF, OTF, etc.)
    /// * `_index` - Font collection index (for TTC files)
    pub fn from_data(data: &[u8], _index: u32) -> Option<Self> {
        // FontArc requires owned data, so we need to copy the slice
        FontArc::try_from_vec(data.to_vec())
            .ok()
            .map(|inner| Font { inner })
    }

    /// Get metrics at a specific font size
    pub fn metrics(&self, size: f32) -> FontMetrics {
        let scaled = self.inner.as_scaled(size);
        FontMetrics {
            ascent: scaled.ascent(),
            descent: scaled.descent(),
            line_gap: scaled.line_gap(),
            units_per_em: self.inner.units_per_em().unwrap_or(1000.0),
            size,
        }
    }

    /// Get the glyph ID for a character
    pub fn glyph_id(&self, c: char) -> GlyphId {
        self.inner.glyph_id(c)
    }

    /// Measure a single character at a given size
    pub fn measure_char(&self, c: char, size: f32) -> GlyphMetrics {
        let scaled = self.inner.as_scaled(size);
        let glyph_id = self.inner.glyph_id(c);
        let advance = scaled.h_advance(glyph_id);

        // Get glyph bounds if available
        let bounds = self
            .inner
            .outline_glyph(glyph_id.with_scale(size))
            .map(|g| g.px_bounds());

        GlyphMetrics {
            advance_width: advance,
            bounds,
        }
    }

    /// Measure a string (simple left-to-right, no shaping)
    ///
    /// This is a basic measurement that works for simple Latin text.
    /// For complex scripts (Arabic, Thai, etc.), proper text shaping
    /// with harfbuzz would be needed.
    pub fn measure_text(&self, text: &str, size: f32) -> TextMetrics {
        let scaled = self.inner.as_scaled(size);
        let mut width = 0.0;

        for c in text.chars() {
            let glyph_id = self.inner.glyph_id(c);
            width += scaled.h_advance(glyph_id);
        }

        let metrics = self.metrics(size);
        TextMetrics {
            width,
            ascent: metrics.ascent,
            descent: metrics.descent,
        }
    }

    /// Measure text with kerning (slightly more accurate)
    ///
    /// Takes kerning pairs into account for more accurate width measurement.
    pub fn measure_text_with_kerning(&self, text: &str, size: f32) -> TextMetrics {
        let scaled = self.inner.as_scaled(size);
        let mut width = 0.0;
        let mut prev_glyph: Option<GlyphId> = None;

        for c in text.chars() {
            let glyph_id = self.inner.glyph_id(c);

            // Add kerning if we have a previous glyph
            if let Some(prev) = prev_glyph {
                width += scaled.kern(prev, glyph_id);
            }

            width += scaled.h_advance(glyph_id);
            prev_glyph = Some(glyph_id);
        }

        let metrics = self.metrics(size);
        TextMetrics {
            width,
            ascent: metrics.ascent,
            descent: metrics.descent,
        }
    }

    /// Check if the font has a glyph for a character
    pub fn has_glyph(&self, c: char) -> bool {
        let glyph_id = self.inner.glyph_id(c);
        // Glyph ID 0 is typically the .notdef glyph (missing glyph)
        glyph_id.0 != 0
    }
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("units_per_em", &self.inner.units_per_em())
            .finish()
    }
}

/// Font-wide metrics at a specific size
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// Distance from baseline to top of tallest glyph
    pub ascent: f32,
    /// Distance from baseline to bottom of lowest glyph (typically negative)
    pub descent: f32,
    /// Extra space between lines (line gap)
    pub line_gap: f32,
    /// Font design units per em
    pub units_per_em: f32,
    /// Font size these metrics are for
    pub size: f32,
}

impl FontMetrics {
    /// Calculate line height for single spacing
    ///
    /// This is the recommended distance between baselines.
    pub fn line_height(&self) -> f32 {
        self.ascent - self.descent + self.line_gap
    }

    /// Get the total height of the em square at this size
    pub fn em_height(&self) -> f32 {
        self.size
    }

    /// Get the height from top of ascenders to bottom of descenders
    pub fn glyph_height(&self) -> f32 {
        self.ascent - self.descent
    }
}

/// Metrics for a single glyph
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    /// Horizontal advance width (distance to next character)
    pub advance_width: f32,
    /// Bounding box of the glyph (if available)
    pub bounds: Option<ab_glyph::Rect>,
}

impl GlyphMetrics {
    /// Get the width of the glyph's visual bounds (if available)
    pub fn visual_width(&self) -> Option<f32> {
        self.bounds.map(|b| b.width())
    }

    /// Get the height of the glyph's visual bounds (if available)
    pub fn visual_height(&self) -> Option<f32> {
        self.bounds.map(|b| b.height())
    }
}

/// Metrics for a measured text string
#[derive(Debug, Clone, Copy)]
pub struct TextMetrics {
    /// Total width of the text
    pub width: f32,
    /// Ascent from the font metrics
    pub ascent: f32,
    /// Descent from the font metrics (typically negative)
    pub descent: f32,
}

impl TextMetrics {
    /// Get the total height of the text line
    pub fn height(&self) -> f32 {
        self.ascent - self.descent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FontDb;

    fn get_test_font() -> Option<Font> {
        let db = FontDb::new();
        let id = db.fallback()?;
        db.load_font(id)
    }

    #[test]
    fn test_font_metrics() {
        if let Some(font) = get_test_font() {
            let metrics = font.metrics(16.0);

            // Ascent should be positive
            assert!(metrics.ascent > 0.0, "Ascent should be positive");

            // Descent should be negative or zero
            assert!(metrics.descent <= 0.0, "Descent should be negative or zero");

            // Line height should be positive
            assert!(metrics.line_height() > 0.0, "Line height should be positive");

            // Size should match what we requested
            assert_eq!(metrics.size, 16.0);
        }
    }

    #[test]
    fn test_measure_char() {
        if let Some(font) = get_test_font() {
            let metrics = font.measure_char('A', 16.0);

            // Advance width should be positive
            assert!(metrics.advance_width > 0.0, "Advance width should be positive");

            // Space should have different width than 'W'
            let space_metrics = font.measure_char(' ', 16.0);
            let w_metrics = font.measure_char('W', 16.0);
            assert!(
                (space_metrics.advance_width - w_metrics.advance_width).abs() > 0.1,
                "Space and W should have different widths"
            );
        }
    }

    #[test]
    fn test_measure_text() {
        if let Some(font) = get_test_font() {
            let metrics = font.measure_text("Hello", 16.0);

            // Width should be positive
            assert!(metrics.width > 0.0, "Text width should be positive");

            // Height should be positive
            assert!(metrics.height() > 0.0, "Text height should be positive");

            // Longer text should be wider
            let long_metrics = font.measure_text("Hello World", 16.0);
            assert!(
                long_metrics.width > metrics.width,
                "Longer text should be wider"
            );
        }
    }

    #[test]
    fn test_measure_text_with_kerning() {
        if let Some(font) = get_test_font() {
            let without_kerning = font.measure_text("AV", 16.0);
            let with_kerning = font.measure_text_with_kerning("AV", 16.0);

            // With kerning should typically be slightly narrower for AV pair
            // (though this depends on the font having kerning data)
            // At minimum, both should give reasonable widths
            assert!(without_kerning.width > 0.0);
            assert!(with_kerning.width > 0.0);
        }
    }

    #[test]
    fn test_empty_text() {
        if let Some(font) = get_test_font() {
            let metrics = font.measure_text("", 16.0);
            assert_eq!(metrics.width, 0.0, "Empty text should have zero width");
        }
    }

    #[test]
    fn test_has_glyph() {
        if let Some(font) = get_test_font() {
            // ASCII characters should be present
            assert!(font.has_glyph('A'), "Font should have glyph for 'A'");
            assert!(font.has_glyph('a'), "Font should have glyph for 'a'");
            assert!(font.has_glyph('0'), "Font should have glyph for '0'");
        }
    }

    #[test]
    fn test_font_size_scaling() {
        if let Some(font) = get_test_font() {
            let metrics_16 = font.measure_text("Hello", 16.0);
            let metrics_32 = font.measure_text("Hello", 32.0);

            // At double the size, width should be approximately double
            let ratio = metrics_32.width / metrics_16.width;
            assert!(
                (ratio - 2.0).abs() < 0.1,
                "Width should scale linearly with size"
            );
        }
    }
}
