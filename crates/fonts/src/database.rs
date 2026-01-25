//! Font database for font discovery and loading

use fontdb::{Database, ID};

use crate::font::Font;

/// Font database for font discovery and loading
pub struct FontDb {
    db: Database,
}

impl FontDb {
    /// Create a new font database with system fonts
    pub fn new() -> Self {
        let mut db = Database::new();
        db.load_system_fonts();
        FontDb { db }
    }

    /// Create an empty font database (for testing)
    pub fn empty() -> Self {
        FontDb {
            db: Database::new(),
        }
    }

    /// Get the number of fonts in the database
    pub fn len(&self) -> usize {
        self.db.len()
    }

    /// Check if the database is empty
    pub fn is_empty(&self) -> bool {
        self.db.is_empty()
    }

    /// Find a font by family name
    pub fn find_font(&self, family: &str, weight: FontWeight, style: FontStyle) -> Option<FontId> {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            weight: weight.into(),
            style: style.into(),
            ..Default::default()
        };
        self.db.query(&query).map(FontId)
    }

    /// Find a font by generic family (sans-serif, serif, monospace)
    pub fn find_generic(&self, family: GenericFamily, weight: FontWeight, style: FontStyle) -> Option<FontId> {
        let fontdb_family = match family {
            GenericFamily::Serif => fontdb::Family::Serif,
            GenericFamily::SansSerif => fontdb::Family::SansSerif,
            GenericFamily::Monospace => fontdb::Family::Monospace,
            GenericFamily::Cursive => fontdb::Family::Cursive,
            GenericFamily::Fantasy => fontdb::Family::Fantasy,
        };

        let query = fontdb::Query {
            families: &[fontdb_family],
            weight: weight.into(),
            style: style.into(),
            ..Default::default()
        };
        self.db.query(&query).map(FontId)
    }

    /// Get fallback font (system default sans-serif)
    pub fn fallback(&self) -> Option<FontId> {
        // Try sans-serif first, then any available font
        self.find_generic(GenericFamily::SansSerif, FontWeight::Normal, FontStyle::Normal)
            .or_else(|| self.db.faces().next().map(|f| FontId(f.id)))
    }

    /// Load font data for a given ID
    pub fn load_font(&self, id: FontId) -> Option<Font> {
        self.db
            .with_face_data(id.0, Font::from_data)
            .flatten()
    }

    /// Get the font family name for a given ID
    pub fn font_family(&self, id: FontId) -> Option<String> {
        self.db
            .face(id.0)
            .and_then(|face| face.families.first().map(|(name, _)| name.clone()))
    }
}

impl Default for FontDb {
    fn default() -> Self {
        Self::new()
    }
}

/// Opaque font identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(pub(crate) ID);

/// Generic font families
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericFamily {
    Serif,
    SansSerif,
    Monospace,
    Cursive,
    Fantasy,
}

/// Font weight (100-900)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontWeight {
    Thin,       // 100
    ExtraLight, // 200
    Light,      // 300
    #[default]
    Normal,     // 400
    Medium,     // 500
    SemiBold,   // 600
    Bold,       // 700
    ExtraBold,  // 800
    Black,      // 900
}

impl FontWeight {
    /// Create a FontWeight from a numeric value (100-900)
    pub fn from_number(n: u16) -> Self {
        match n {
            0..=149 => FontWeight::Thin,
            150..=249 => FontWeight::ExtraLight,
            250..=349 => FontWeight::Light,
            350..=449 => FontWeight::Normal,
            450..=549 => FontWeight::Medium,
            550..=649 => FontWeight::SemiBold,
            650..=749 => FontWeight::Bold,
            750..=849 => FontWeight::ExtraBold,
            _ => FontWeight::Black,
        }
    }

    /// Get the numeric value of this weight
    pub fn to_number(self) -> u16 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
        }
    }
}

impl From<FontWeight> for fontdb::Weight {
    fn from(weight: FontWeight) -> Self {
        fontdb::Weight(weight.to_number())
    }
}

/// Font style (normal, italic, oblique)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl From<FontStyle> for fontdb::Style {
    fn from(style: FontStyle) -> Self {
        match style {
            FontStyle::Normal => fontdb::Style::Normal,
            FontStyle::Italic => fontdb::Style::Italic,
            FontStyle::Oblique => fontdb::Style::Oblique,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_weight_from_number() {
        assert_eq!(FontWeight::from_number(100), FontWeight::Thin);
        assert_eq!(FontWeight::from_number(400), FontWeight::Normal);
        assert_eq!(FontWeight::from_number(700), FontWeight::Bold);
        assert_eq!(FontWeight::from_number(900), FontWeight::Black);
    }

    #[test]
    fn test_font_weight_to_number() {
        assert_eq!(FontWeight::Thin.to_number(), 100);
        assert_eq!(FontWeight::Normal.to_number(), 400);
        assert_eq!(FontWeight::Bold.to_number(), 700);
        assert_eq!(FontWeight::Black.to_number(), 900);
    }

    #[test]
    fn test_empty_database() {
        let db = FontDb::empty();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_system_fonts_loaded() {
        let db = FontDb::new();
        // System should have at least some fonts
        // This might fail in minimal environments, but should work on typical systems
        assert!(!db.is_empty(), "System should have fonts available");
    }

    #[test]
    fn test_fallback_font() {
        let db = FontDb::new();
        let fallback = db.fallback();
        // Should be able to get a fallback font on any system with fonts
        assert!(fallback.is_some(), "Should have a fallback font available");
    }

    #[test]
    fn test_load_fallback_font() {
        let db = FontDb::new();
        if let Some(id) = db.fallback() {
            let font = db.load_font(id);
            assert!(font.is_some(), "Should be able to load fallback font");
        }
    }

    #[test]
    fn test_find_generic_sans_serif() {
        let db = FontDb::new();
        let font = db.find_generic(GenericFamily::SansSerif, FontWeight::Normal, FontStyle::Normal);
        // Most systems have a sans-serif font
        assert!(font.is_some(), "Should find a sans-serif font");
    }
}
