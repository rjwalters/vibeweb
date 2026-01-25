//! CSS Value Types
//!
//! Represents the various value types that can appear in CSS declarations,
//! including lengths, colors, keywords, and more.

use std::fmt;

/// A CSS value in a declaration
#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    /// A length value (e.g., 10px, 2em)
    Length(Length),
    /// A percentage value (e.g., 50%)
    Percentage(f64),
    /// A color value
    Color(Color),
    /// A keyword (e.g., inherit, auto, none)
    Keyword(String),
    /// A numeric value without unit
    Number(f64),
    /// A string value
    String(String),
    /// A URL reference
    Url(String),
    /// A function call with arguments (e.g., calc(...))
    Function(String, Vec<CssValue>),
    /// Multiple values (e.g., margin: 10px 20px)
    Multiple(Vec<CssValue>),
    /// Raw/unparsed value (fallback)
    Raw(String),
}

impl Default for CssValue {
    fn default() -> Self {
        CssValue::Keyword("initial".to_string())
    }
}

impl fmt::Display for CssValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CssValue::Length(len) => write!(f, "{}", len),
            CssValue::Percentage(p) => write!(f, "{}%", p),
            CssValue::Color(c) => write!(f, "{}", c),
            CssValue::Keyword(k) => write!(f, "{}", k),
            CssValue::Number(n) => write!(f, "{}", n),
            CssValue::String(s) => write!(f, "\"{}\"", s),
            CssValue::Url(u) => write!(f, "url({})", u),
            CssValue::Function(name, args) => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            CssValue::Multiple(values) => {
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", v)?;
                }
                Ok(())
            }
            CssValue::Raw(r) => write!(f, "{}", r),
        }
    }
}

/// A length value with unit
#[derive(Debug, Clone, PartialEq)]
pub struct Length {
    pub value: f64,
    pub unit: LengthUnit,
}

impl Length {
    /// Create a new length
    pub fn new(value: f64, unit: LengthUnit) -> Self {
        Length { value, unit }
    }

    /// Create a length in pixels
    pub fn px(value: f64) -> Self {
        Length::new(value, LengthUnit::Px)
    }

    /// Create a length in em
    pub fn em(value: f64) -> Self {
        Length::new(value, LengthUnit::Em)
    }

    /// Create a zero length
    pub fn zero() -> Self {
        Length::new(0.0, LengthUnit::Px)
    }

    /// Convert to pixels given a reference size for relative units
    pub fn to_px(&self, reference_px: f64) -> f64 {
        match self.unit {
            LengthUnit::Px => self.value,
            LengthUnit::Em | LengthUnit::Rem => self.value * reference_px,
            LengthUnit::Percent => self.value / 100.0 * reference_px,
            LengthUnit::Vw | LengthUnit::Vh => self.value / 100.0 * reference_px,
            LengthUnit::Pt => self.value * 96.0 / 72.0,
            LengthUnit::Cm => self.value * 96.0 / 2.54,
            LengthUnit::Mm => self.value * 96.0 / 25.4,
            LengthUnit::In => self.value * 96.0,
        }
    }
}

impl Default for Length {
    fn default() -> Self {
        Length::zero()
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.value, self.unit)
    }
}

/// Units for length values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LengthUnit {
    /// Pixels
    #[default]
    Px,
    /// Font size of the element
    Em,
    /// Font size of the root element
    Rem,
    /// Percentage
    Percent,
    /// Viewport width
    Vw,
    /// Viewport height
    Vh,
    /// Points
    Pt,
    /// Centimeters
    Cm,
    /// Millimeters
    Mm,
    /// Inches
    In,
}

impl LengthUnit {
    /// Parse a unit string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "px" => Some(LengthUnit::Px),
            "em" => Some(LengthUnit::Em),
            "rem" => Some(LengthUnit::Rem),
            "%" => Some(LengthUnit::Percent),
            "vw" => Some(LengthUnit::Vw),
            "vh" => Some(LengthUnit::Vh),
            "pt" => Some(LengthUnit::Pt),
            "cm" => Some(LengthUnit::Cm),
            "mm" => Some(LengthUnit::Mm),
            "in" => Some(LengthUnit::In),
            _ => None,
        }
    }
}


impl fmt::Display for LengthUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LengthUnit::Px => "px",
            LengthUnit::Em => "em",
            LengthUnit::Rem => "rem",
            LengthUnit::Percent => "%",
            LengthUnit::Vw => "vw",
            LengthUnit::Vh => "vh",
            LengthUnit::Pt => "pt",
            LengthUnit::Cm => "cm",
            LengthUnit::Mm => "mm",
            LengthUnit::In => "in",
        };
        write!(f, "{}", s)
    }
}

/// A color value
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,
}

impl Color {
    /// Create a new color
    pub fn new(r: u8, g: u8, b: u8, a: f64) -> Self {
        Color { r, g, b, a }
    }

    /// Create an opaque color
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::new(r, g, b, 1.0)
    }

    /// Create a color with alpha
    pub fn rgba(r: u8, g: u8, b: u8, a: f64) -> Self {
        Color::new(r, g, b, a)
    }

    /// Parse a hex color string
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            3 => {
                // Short form: #RGB -> #RRGGBB
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Color::rgb(r, g, b))
            }
            4 => {
                // Short form with alpha: #RGBA
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
                Some(Color::rgba(r, g, b, a as f64 / 255.0))
            }
            6 => {
                // Full form: #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::rgb(r, g, b))
            }
            8 => {
                // Full form with alpha: #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::rgba(r, g, b, a as f64 / 255.0))
            }
            _ => None,
        }
    }

    /// Get a named color
    pub fn from_name(name: &str) -> Option<Self> {
        // Common named colors (CSS Level 1 + some CSS Level 2)
        match name.to_lowercase().as_str() {
            // CSS Level 1
            "black" => Some(Color::rgb(0, 0, 0)),
            "silver" => Some(Color::rgb(192, 192, 192)),
            "gray" | "grey" => Some(Color::rgb(128, 128, 128)),
            "white" => Some(Color::rgb(255, 255, 255)),
            "maroon" => Some(Color::rgb(128, 0, 0)),
            "red" => Some(Color::rgb(255, 0, 0)),
            "purple" => Some(Color::rgb(128, 0, 128)),
            "fuchsia" | "magenta" => Some(Color::rgb(255, 0, 255)),
            "green" => Some(Color::rgb(0, 128, 0)),
            "lime" => Some(Color::rgb(0, 255, 0)),
            "olive" => Some(Color::rgb(128, 128, 0)),
            "yellow" => Some(Color::rgb(255, 255, 0)),
            "navy" => Some(Color::rgb(0, 0, 128)),
            "blue" => Some(Color::rgb(0, 0, 255)),
            "teal" => Some(Color::rgb(0, 128, 128)),
            "aqua" | "cyan" => Some(Color::rgb(0, 255, 255)),
            // CSS Level 2/3 common colors
            "orange" => Some(Color::rgb(255, 165, 0)),
            "pink" => Some(Color::rgb(255, 192, 203)),
            "brown" => Some(Color::rgb(165, 42, 42)),
            "coral" => Some(Color::rgb(255, 127, 80)),
            "crimson" => Some(Color::rgb(220, 20, 60)),
            "darkblue" => Some(Color::rgb(0, 0, 139)),
            "darkgray" | "darkgrey" => Some(Color::rgb(169, 169, 169)),
            "darkgreen" => Some(Color::rgb(0, 100, 0)),
            "darkred" => Some(Color::rgb(139, 0, 0)),
            "gold" => Some(Color::rgb(255, 215, 0)),
            "indigo" => Some(Color::rgb(75, 0, 130)),
            "lightblue" => Some(Color::rgb(173, 216, 230)),
            "lightgray" | "lightgrey" => Some(Color::rgb(211, 211, 211)),
            "lightgreen" => Some(Color::rgb(144, 238, 144)),
            "lightyellow" => Some(Color::rgb(255, 255, 224)),
            "violet" => Some(Color::rgb(238, 130, 238)),
            // Special values
            "transparent" => Some(Color::rgba(0, 0, 0, 0.0)),
            "currentcolor" => None, // Needs special handling
            _ => None,
        }
    }

    /// Predefined colors
    pub fn black() -> Self {
        Color::rgb(0, 0, 0)
    }

    pub fn white() -> Self {
        Color::rgb(255, 255, 255)
    }

    pub fn transparent() -> Self {
        Color::rgba(0, 0, 0, 0.0)
    }

    /// Convert to CSS hex string
    pub fn to_hex(&self) -> String {
        if (self.a - 1.0).abs() < f64::EPSILON {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                self.r,
                self.g,
                self.b,
                (self.a * 255.0) as u8
            )
        }
    }

    /// Convert to [r, g, b, a] array
    pub fn to_array(&self) -> [f64; 4] {
        [
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
            self.a,
        ]
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::black()
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if (self.a - 1.0).abs() < f64::EPSILON {
            write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_short() {
        let color = Color::from_hex("#f00").unwrap();
        assert_eq!(color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn parse_hex_long() {
        let color = Color::from_hex("#ff0000").unwrap();
        assert_eq!(color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn parse_hex_with_alpha() {
        let color = Color::from_hex("#ff000080").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert!((color.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn parse_named_color() {
        assert_eq!(Color::from_name("red"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(Color::from_name("blue"), Some(Color::rgb(0, 0, 255)));
        assert_eq!(Color::from_name("transparent"), Some(Color::rgba(0, 0, 0, 0.0)));
    }

    #[test]
    fn length_to_px() {
        let px = Length::px(100.0);
        assert_eq!(px.to_px(16.0), 100.0);

        let em = Length::em(2.0);
        assert_eq!(em.to_px(16.0), 32.0);
    }

    #[test]
    fn parse_length_unit() {
        assert_eq!(LengthUnit::parse("px"), Some(LengthUnit::Px));
        assert_eq!(LengthUnit::parse("em"), Some(LengthUnit::Em));
        assert_eq!(LengthUnit::parse("rem"), Some(LengthUnit::Rem));
        assert_eq!(LengthUnit::parse("vw"), Some(LengthUnit::Vw));
        assert_eq!(LengthUnit::parse("invalid"), None);
    }

    #[test]
    fn color_to_hex() {
        let color = Color::rgb(255, 0, 0);
        assert_eq!(color.to_hex(), "#ff0000");

        let color_alpha = Color::rgba(255, 0, 0, 0.5);
        assert_eq!(color_alpha.to_hex(), "#ff00007f");
    }

    #[test]
    fn css_value_display() {
        assert_eq!(format!("{}", CssValue::Length(Length::px(10.0))), "10px");
        assert_eq!(format!("{}", CssValue::Percentage(50.0)), "50%");
        assert_eq!(format!("{}", CssValue::Keyword("auto".to_string())), "auto");
    }
}
