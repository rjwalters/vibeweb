//! Color type for graphics rendering
//!
//! This is the canonical Color type used throughout the browser. All crates
//! should use this type for color values to ensure consistency.

use std::fmt;

/// A 32-bit RGBA color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255, 255 = fully opaque)
    pub a: u8,
}

impl Color {
    /// White color
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    /// Black color
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    /// Red color
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    /// Green color
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };

    /// Blue color
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };

    /// Transparent (fully transparent black)
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    /// Create a new color from RGBA components
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new opaque color from RGB components
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Convert to 32-bit ARGB format (used by softbuffer)
    #[inline]
    pub const fn to_argb(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Create a color from 32-bit ARGB format
    pub const fn from_argb(argb: u32) -> Self {
        Self {
            a: ((argb >> 24) & 0xFF) as u8,
            r: ((argb >> 16) & 0xFF) as u8,
            g: ((argb >> 8) & 0xFF) as u8,
            b: (argb & 0xFF) as u8,
        }
    }

    /// Create an opaque color from a hex value (e.g., 0xFF0000 for red).
    ///
    /// This is useful for CSS-style hex colors without alpha.
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    /// Check if this color is fully transparent.
    pub const fn is_transparent(&self) -> bool {
        self.a == 0
    }

    /// Create a new color from RGBA components (alias for `new`)
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::BLACK
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "rgba({}, {}, {}, {:.2})",
                self.r,
                self.g,
                self.b,
                self.a as f32 / 255.0
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_constants() {
        assert_eq!(Color::WHITE.to_argb(), 0xFFFFFFFF);
        assert_eq!(Color::BLACK.to_argb(), 0xFF000000);
        assert_eq!(Color::RED.to_argb(), 0xFFFF0000);
        assert_eq!(Color::GREEN.to_argb(), 0xFF00FF00);
        assert_eq!(Color::BLUE.to_argb(), 0xFF0000FF);
        assert_eq!(Color::TRANSPARENT.to_argb(), 0x00000000);
    }

    #[test]
    fn color_rgb() {
        let color = Color::rgb(128, 64, 32);
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 64);
        assert_eq!(color.b, 32);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn color_roundtrip() {
        let original = Color::new(100, 150, 200, 128);
        let argb = original.to_argb();
        let restored = Color::from_argb(argb);
        assert_eq!(original, restored);
    }
}
