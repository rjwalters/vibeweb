//! CSS value types for computed styles.
//!
//! This module defines the fundamental value types used throughout the style system,
//! including lengths, colors, and box model properties.

use std::fmt;

/// A length value with an associated unit.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Length {
    /// Pixels (absolute)
    Px(f32),
    /// Ems (relative to font-size)
    Em(f32),
    /// Rems (relative to root font-size)
    Rem(f32),
    /// Percentage (relative to containing block)
    Percent(f32),
    /// Zero length
    #[default]
    Zero,
}

impl Length {
    /// Create a length in pixels.
    pub fn px(value: f32) -> Self {
        if value == 0.0 {
            Length::Zero
        } else {
            Length::Px(value)
        }
    }

    /// Create a length in ems.
    pub fn em(value: f32) -> Self {
        if value == 0.0 {
            Length::Zero
        } else {
            Length::Em(value)
        }
    }

    /// Create a length in rems.
    pub fn rem(value: f32) -> Self {
        if value == 0.0 {
            Length::Zero
        } else {
            Length::Rem(value)
        }
    }

    /// Create a percentage length.
    pub fn percent(value: f32) -> Self {
        Length::Percent(value)
    }

    /// Resolve this length to pixels.
    ///
    /// # Arguments
    /// * `font_size` - The current element's font-size in pixels (for em units)
    /// * `root_font_size` - The root element's font-size in pixels (for rem units)
    /// * `containing_size` - The containing block's size in pixels (for percentages)
    pub fn to_px(&self, font_size: f32, root_font_size: f32, containing_size: Option<f32>) -> f32 {
        match self {
            Length::Px(v) => *v,
            Length::Em(v) => *v * font_size,
            Length::Rem(v) => *v * root_font_size,
            Length::Percent(v) => containing_size.map(|s| s * v / 100.0).unwrap_or(0.0),
            Length::Zero => 0.0,
        }
    }

    /// Check if this is a zero length.
    pub fn is_zero(&self) -> bool {
        matches!(self, Length::Zero)
            || matches!(self, Length::Px(v) if *v == 0.0)
            || matches!(self, Length::Em(v) if *v == 0.0)
            || matches!(self, Length::Rem(v) if *v == 0.0)
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Length::Px(v) => write!(f, "{}px", v),
            Length::Em(v) => write!(f, "{}em", v),
            Length::Rem(v) => write!(f, "{}rem", v),
            Length::Percent(v) => write!(f, "{}%", v),
            Length::Zero => write!(f, "0"),
        }
    }
}

/// A length value that can also be `auto`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LengthOrAuto {
    Length(Length),
    #[default]
    Auto,
}

impl LengthOrAuto {
    /// Create from a length.
    pub fn length(l: Length) -> Self {
        LengthOrAuto::Length(l)
    }

    /// Create an auto value.
    pub fn auto() -> Self {
        LengthOrAuto::Auto
    }

    /// Check if this is auto.
    pub fn is_auto(&self) -> bool {
        matches!(self, LengthOrAuto::Auto)
    }

    /// Try to resolve to pixels, returning None for auto.
    pub fn to_px(
        &self,
        font_size: f32,
        root_font_size: f32,
        containing_size: Option<f32>,
    ) -> Option<f32> {
        match self {
            LengthOrAuto::Length(l) => Some(l.to_px(font_size, root_font_size, containing_size)),
            LengthOrAuto::Auto => None,
        }
    }
}

impl fmt::Display for LengthOrAuto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LengthOrAuto::Length(l) => write!(f, "{}", l),
            LengthOrAuto::Auto => write!(f, "auto"),
        }
    }
}

impl From<Length> for LengthOrAuto {
    fn from(l: Length) -> Self {
        LengthOrAuto::Length(l)
    }
}

/// Four-sided values (top, right, bottom, left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Default> Default for Sides<T> {
    fn default() -> Self {
        Sides {
            top: T::default(),
            right: T::default(),
            bottom: T::default(),
            left: T::default(),
        }
    }
}

impl<T: Clone> Sides<T> {
    /// Create sides with all values the same.
    pub fn all(value: T) -> Self {
        Sides {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    /// Create sides from vertical and horizontal values.
    pub fn symmetric(vertical: T, horizontal: T) -> Self {
        Sides {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create sides with explicit values for each side.
    pub fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Sides {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl Sides<Length> {
    /// Resolve all sides to pixels.
    pub fn to_px(
        &self,
        font_size: f32,
        root_font_size: f32,
        containing_size: Option<f32>,
    ) -> Sides<f32> {
        Sides {
            top: self.top.to_px(font_size, root_font_size, containing_size),
            right: self.right.to_px(font_size, root_font_size, containing_size),
            bottom: self
                .bottom
                .to_px(font_size, root_font_size, containing_size),
            left: self.left.to_px(font_size, root_font_size, containing_size),
        }
    }
}

impl Sides<LengthOrAuto> {
    /// Resolve all sides to pixels, using a default for auto values.
    pub fn to_px_or(
        &self,
        font_size: f32,
        root_font_size: f32,
        containing_size: Option<f32>,
        default: f32,
    ) -> Sides<f32> {
        Sides {
            top: self
                .top
                .to_px(font_size, root_font_size, containing_size)
                .unwrap_or(default),
            right: self
                .right
                .to_px(font_size, root_font_size, containing_size)
                .unwrap_or(default),
            bottom: self
                .bottom
                .to_px(font_size, root_font_size, containing_size)
                .unwrap_or(default),
            left: self
                .left
                .to_px(font_size, root_font_size, containing_size)
                .unwrap_or(default),
        }
    }
}

// Re-export Color from vw_gfx to consolidate the type system.
// This ensures all crates use the same Color type for rendering.
pub use vw_gfx::color::Color;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_to_px() {
        let font_size = 16.0;
        let root_font_size = 16.0;

        assert_eq!(
            Length::px(10.0).to_px(font_size, root_font_size, None),
            10.0
        );
        assert_eq!(Length::em(2.0).to_px(font_size, root_font_size, None), 32.0);
        assert_eq!(
            Length::rem(1.5).to_px(font_size, root_font_size, None),
            24.0
        );
        assert_eq!(
            Length::percent(50.0).to_px(font_size, root_font_size, Some(200.0)),
            100.0
        );
        assert_eq!(Length::Zero.to_px(font_size, root_font_size, None), 0.0);
    }

    #[test]
    fn test_length_or_auto() {
        let auto = LengthOrAuto::auto();
        let length = LengthOrAuto::length(Length::px(10.0));

        assert!(auto.is_auto());
        assert!(!length.is_auto());
        assert_eq!(auto.to_px(16.0, 16.0, None), None);
        assert_eq!(length.to_px(16.0, 16.0, None), Some(10.0));
    }

    #[test]
    fn test_sides() {
        let sides = Sides::all(Length::px(10.0));
        assert_eq!(sides.top, Length::px(10.0));
        assert_eq!(sides.right, Length::px(10.0));

        let symmetric = Sides::symmetric(Length::px(5.0), Length::px(10.0));
        assert_eq!(symmetric.top, Length::px(5.0));
        assert_eq!(symmetric.right, Length::px(10.0));
    }

    #[test]
    fn test_color() {
        let red = Color::from_hex(0xFF0000);
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);

        assert!(Color::TRANSPARENT.is_transparent());
        assert!(!Color::BLACK.is_transparent());
    }
}
