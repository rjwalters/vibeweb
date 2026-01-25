//! Font cache for efficient font loading and reuse

use std::collections::HashMap;

use crate::database::{FontDb, FontId, FontStyle, FontWeight, GenericFamily};
use crate::font::{Font, TextMetrics};

/// Cache for loaded fonts to avoid repeated parsing
pub struct FontCache {
    db: FontDb,
    loaded: HashMap<FontId, Font>,
    /// Cache of font ID lookups: (family, weight, style) -> FontId
    lookup_cache: HashMap<(String, FontWeight, FontStyle), FontId>,
    /// Fallback font ID (cached for performance)
    fallback_id: Option<FontId>,
}

impl FontCache {
    /// Create a new font cache with system fonts
    pub fn new() -> Self {
        let db = FontDb::new();
        let fallback_id = db.fallback();

        FontCache {
            db,
            loaded: HashMap::new(),
            lookup_cache: HashMap::new(),
            fallback_id,
        }
    }

    /// Get a font by family name, weight, and style
    ///
    /// Returns the fallback font if the requested font is not found.
    pub fn get(&mut self, family: &str, weight: FontWeight, style: FontStyle) -> Option<&Font> {
        let id = self.resolve_font_id(family, weight, style)?;
        self.load_and_cache(id)
    }

    /// Get a font by generic family
    pub fn get_generic(
        &mut self,
        family: GenericFamily,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<&Font> {
        let id = self.db.find_generic(family, weight, style)?;
        self.load_and_cache(id)
    }

    /// Get the fallback font
    pub fn get_fallback(&mut self) -> Option<&Font> {
        let id = self.fallback_id?;
        self.load_and_cache(id)
    }

    /// Measure text with specified font properties
    ///
    /// Falls back to the system default font if the requested font is not found.
    pub fn measure_text(
        &mut self,
        text: &str,
        size: f32,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<TextMetrics> {
        let font = self.get(family, weight, style)?;
        Some(font.measure_text(text, size))
    }

    /// Measure text with kerning
    pub fn measure_text_with_kerning(
        &mut self,
        text: &str,
        size: f32,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<TextMetrics> {
        let font = self.get(family, weight, style)?;
        Some(font.measure_text_with_kerning(text, size))
    }

    /// Clear the font cache
    ///
    /// This will force fonts to be reloaded on next access.
    pub fn clear(&mut self) {
        self.loaded.clear();
        self.lookup_cache.clear();
    }

    /// Get the number of loaded fonts
    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }

    /// Get the number of fonts available in the database
    pub fn available_count(&self) -> usize {
        self.db.len()
    }

    /// Resolve a font family name to a FontId
    fn resolve_font_id(
        &mut self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<FontId> {
        let cache_key = (family.to_string(), weight, style);

        // Check lookup cache first
        if let Some(&id) = self.lookup_cache.get(&cache_key) {
            return Some(id);
        }

        // Try to find the font
        let id = self
            .db
            .find_font(family, weight, style)
            .or(self.fallback_id)?;

        // Cache the lookup
        self.lookup_cache.insert(cache_key, id);
        Some(id)
    }

    /// Load a font and add it to the cache
    fn load_and_cache(&mut self, id: FontId) -> Option<&Font> {
        // Load if not already cached
        if !self.loaded.contains_key(&id) {
            let font = self.db.load_font(id)?;
            self.loaded.insert(id, font);
        }

        self.loaded.get(&id)
    }
}

impl Default for FontCache {
    fn default() -> Self {
        Self::new()
    }
}

/// A font selection specification
#[derive(Debug, Clone, PartialEq)]
pub struct FontSpec {
    /// Font family name (or generic family like "sans-serif")
    pub family: String,
    /// Font weight
    pub weight: FontWeight,
    /// Font style
    pub style: FontStyle,
    /// Font size in pixels
    pub size: f32,
}

impl FontSpec {
    /// Create a new font specification
    pub fn new(family: impl Into<String>, size: f32) -> Self {
        FontSpec {
            family: family.into(),
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            size,
        }
    }

    /// Set the font weight
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Set the font style
    pub fn style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }

    /// Set bold weight
    pub fn bold(self) -> Self {
        self.weight(FontWeight::Bold)
    }

    /// Set italic style
    pub fn italic(self) -> Self {
        self.style(FontStyle::Italic)
    }
}

impl Default for FontSpec {
    fn default() -> Self {
        FontSpec::new("sans-serif", 16.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = FontCache::new();
        assert!(cache.available_count() > 0, "Should have system fonts");
        assert_eq!(cache.loaded_count(), 0, "No fonts loaded yet");
    }

    #[test]
    fn test_get_fallback() {
        let mut cache = FontCache::new();
        let font = cache.get_fallback();
        assert!(font.is_some(), "Should have a fallback font");
        assert_eq!(cache.loaded_count(), 1, "Should have loaded one font");
    }

    #[test]
    fn test_font_caching() {
        let mut cache = FontCache::new();

        // First access loads the font
        let _ = cache.get("sans-serif", FontWeight::Normal, FontStyle::Normal);
        let count_after_first = cache.loaded_count();

        // Second access should use cached font
        let _ = cache.get("sans-serif", FontWeight::Normal, FontStyle::Normal);
        let count_after_second = cache.loaded_count();

        assert_eq!(
            count_after_first, count_after_second,
            "Should not load same font twice"
        );
    }

    #[test]
    fn test_measure_text() {
        let mut cache = FontCache::new();
        let metrics = cache.measure_text(
            "Hello, World!",
            16.0,
            "sans-serif",
            FontWeight::Normal,
            FontStyle::Normal,
        );

        assert!(metrics.is_some(), "Should be able to measure text");
        let metrics = metrics.unwrap();
        assert!(metrics.width > 0.0, "Text should have width");
    }

    #[test]
    fn test_clear_cache() {
        let mut cache = FontCache::new();
        let _ = cache.get_fallback();
        assert!(cache.loaded_count() > 0);

        cache.clear();
        assert_eq!(cache.loaded_count(), 0, "Cache should be empty after clear");
    }

    #[test]
    fn test_font_spec() {
        let spec = FontSpec::new("Arial", 16.0).bold().italic();
        assert_eq!(spec.family, "Arial");
        assert_eq!(spec.size, 16.0);
        assert_eq!(spec.weight, FontWeight::Bold);
        assert_eq!(spec.style, FontStyle::Italic);
    }

    #[test]
    fn test_font_spec_default() {
        let spec = FontSpec::default();
        assert_eq!(spec.family, "sans-serif");
        assert_eq!(spec.size, 16.0);
        assert_eq!(spec.weight, FontWeight::Normal);
        assert_eq!(spec.style, FontStyle::Normal);
    }
}
