//! Font loading and text measurement for the vibeweb browser
//!
//! This crate provides font discovery, loading, and text measurement capabilities.
//! It uses system fonts via fontdb and provides glyph metrics via ab_glyph.
//!
//! # Example
//!
//! ```no_run
//! use vw_fonts::{FontCache, FontWeight, FontStyle};
//!
//! let mut fonts = FontCache::new();
//!
//! // Measure some text
//! let metrics = fonts.measure_text(
//!     "Hello, World!",
//!     16.0,
//!     "Arial",
//!     FontWeight::Normal,
//!     FontStyle::Normal,
//! );
//!
//! if let Some(metrics) = metrics {
//!     println!("Text width: {} px", metrics.width);
//!     println!("Line height: {} px", metrics.height());
//! }
//! ```
//!
//! # Features
//!
//! - **System font discovery**: Automatically loads fonts installed on the system
//! - **Font family lookup**: Find fonts by family name or generic family (serif, sans-serif, etc.)
//! - **Font fallback**: Automatic fallback to system default when requested font is unavailable
//! - **Font caching**: Loaded fonts are cached to avoid repeated parsing
//! - **Text measurement**: Measure text width with optional kerning support
//! - **Font metrics**: Access ascent, descent, line gap, and other font metrics
//!
//! # Limitations
//!
//! - No text shaping for complex scripts (Arabic, Thai, etc.)
//! - No web font loading (@font-face)
//! - No OpenType feature support (ligatures, etc.)
//!
//! These features may be added in future milestones.

mod cache;
mod database;
mod font;

pub use cache::{FontCache, FontSpec};
pub use database::{FontDb, FontId, FontStyle, FontWeight, GenericFamily};
pub use font::{Font, FontMetrics, GlyphMetrics, TextMetrics};

/// Convenience function for one-off text measurement
///
/// Creates a temporary font cache and measures the text.
/// For repeated measurements, create a `FontCache` and reuse it.
///
/// # Arguments
///
/// * `text` - The text to measure
/// * `size` - Font size in pixels
/// * `family` - Font family name
///
/// # Returns
///
/// Text metrics if the font was found, None otherwise.
///
/// # Example
///
/// ```no_run
/// use vw_fonts::measure_text;
///
/// if let Some(metrics) = measure_text("Hello", 16.0, "Arial") {
///     println!("Width: {}", metrics.width);
/// }
/// ```
pub fn measure_text(text: &str, size: f32, family: &str) -> Option<TextMetrics> {
    let mut cache = FontCache::new();
    cache.measure_text(text, size, family, FontWeight::Normal, FontStyle::Normal)
}

/// Trait for integration with the layout engine
///
/// Implement this trait to provide text measurement capabilities
/// to the layout engine.
pub trait TextMeasurer {
    /// Measure the width and height of text
    fn measure(
        &mut self,
        text: &str,
        size: f32,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<TextMetrics>;

    /// Get font metrics for a specific font configuration
    fn font_metrics(
        &mut self,
        size: f32,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<FontMetrics>;
}

impl TextMeasurer for FontCache {
    fn measure(
        &mut self,
        text: &str,
        size: f32,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<TextMetrics> {
        self.measure_text(text, size, family, weight, style)
    }

    fn font_metrics(
        &mut self,
        size: f32,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<FontMetrics> {
        let font = self.get(family, weight, style)?;
        Some(font.metrics(size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_text_convenience() {
        let metrics = measure_text("Hello", 16.0, "sans-serif");
        assert!(metrics.is_some(), "Should be able to measure text");
    }

    #[test]
    fn test_text_measurer_trait() {
        let mut cache = FontCache::new();

        // Use through the trait
        let measurer: &mut dyn TextMeasurer = &mut cache;
        let metrics = measurer.measure(
            "Test",
            16.0,
            "sans-serif",
            FontWeight::Normal,
            FontStyle::Normal,
        );
        assert!(metrics.is_some());

        let font_metrics = measurer.font_metrics(
            16.0,
            "sans-serif",
            FontWeight::Normal,
            FontStyle::Normal,
        );
        assert!(font_metrics.is_some());
    }

    #[test]
    fn test_public_api() {
        // Verify all public types are accessible
        let _ = FontWeight::Normal;
        let _ = FontStyle::Italic;
        let _ = GenericFamily::SansSerif;

        let spec = FontSpec::new("Arial", 16.0).bold();
        assert_eq!(spec.weight, FontWeight::Bold);
    }

    #[test]
    fn test_integration_workflow() {
        // Simulate a typical usage workflow
        let mut cache = FontCache::new();

        // 1. Get font metrics for layout calculations
        if let Some(font) = cache.get("sans-serif", FontWeight::Normal, FontStyle::Normal) {
            let metrics = font.metrics(16.0);
            let line_height = metrics.line_height();
            assert!(line_height > 0.0);
        }

        // 2. Measure text for inline layout
        if let Some(metrics) = cache.measure_text(
            "Hello, World!",
            16.0,
            "sans-serif",
            FontWeight::Normal,
            FontStyle::Normal,
        ) {
            assert!(metrics.width > 0.0);
            assert!(metrics.height() > 0.0);
        }

        // 3. Measure with different weights
        if let Some(normal) = cache.measure_text(
            "Bold",
            16.0,
            "sans-serif",
            FontWeight::Normal,
            FontStyle::Normal,
        ) {
            if let Some(bold) = cache.measure_text(
                "Bold",
                16.0,
                "sans-serif",
                FontWeight::Bold,
                FontStyle::Normal,
            ) {
                // Bold text is often slightly wider
                // Just verify both measurements work
                assert!(normal.width > 0.0);
                assert!(bold.width > 0.0);
            }
        }
    }
}
