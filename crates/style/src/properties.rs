//! CSS property types and enumerations.
//!
//! This module defines the property types used in computed styles,
//! focusing on layout-critical properties for M2/M3.

use std::fmt;

/// Display property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    #[default]
    Block,
    Inline,
    InlineBlock,
    None,
    // Flex and Grid can be added later
}

impl Display {
    /// Check if this display value generates a box.
    pub fn generates_box(&self) -> bool {
        !matches!(self, Display::None)
    }

    /// Check if this display value establishes a block formatting context.
    pub fn is_block_level(&self) -> bool {
        matches!(self, Display::Block | Display::InlineBlock)
    }

    /// Check if this display value is inline-level.
    pub fn is_inline_level(&self) -> bool {
        matches!(self, Display::Inline | Display::InlineBlock)
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Display::Block => write!(f, "block"),
            Display::Inline => write!(f, "inline"),
            Display::InlineBlock => write!(f, "inline-block"),
            Display::None => write!(f, "none"),
        }
    }
}

/// Position property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
    // Sticky can be added later
}

impl Position {
    /// Check if this position creates a positioned element.
    pub fn is_positioned(&self) -> bool {
        !matches!(self, Position::Static)
    }

    /// Check if this position removes the element from normal flow.
    pub fn is_out_of_flow(&self) -> bool {
        matches!(self, Position::Absolute | Position::Fixed)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Position::Static => write!(f, "static"),
            Position::Relative => write!(f, "relative"),
            Position::Absolute => write!(f, "absolute"),
            Position::Fixed => write!(f, "fixed"),
        }
    }
}

/// Font weight values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontWeight(pub u16);

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::NORMAL
    }
}

impl FontWeight {
    pub const NORMAL: FontWeight = FontWeight(400);
    pub const BOLD: FontWeight = FontWeight(700);
    pub const THIN: FontWeight = FontWeight(100);
    pub const LIGHT: FontWeight = FontWeight(300);
    pub const MEDIUM: FontWeight = FontWeight(500);
    pub const SEMIBOLD: FontWeight = FontWeight(600);
    pub const EXTRABOLD: FontWeight = FontWeight(800);
    pub const BLACK: FontWeight = FontWeight(900);

    /// Create a font weight from a numeric value.
    /// Values are clamped to the valid range [1, 1000].
    pub fn new(weight: u16) -> Self {
        FontWeight(weight.clamp(1, 1000))
    }

    /// Get the numeric weight value.
    pub fn value(&self) -> u16 {
        self.0
    }

    /// Check if this is a bold weight (>= 700).
    pub fn is_bold(&self) -> bool {
        self.0 >= 700
    }
}

impl fmt::Display for FontWeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            400 => write!(f, "normal"),
            700 => write!(f, "bold"),
            _ => write!(f, "{}", self.0),
        }
    }
}

/// Line height values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineHeight {
    /// Normal line height (typically 1.2)
    #[default]
    Normal,
    /// A unitless multiplier of font-size
    Number(f32),
    /// An explicit length
    Length(f32),
}

impl LineHeight {
    /// Resolve line height to pixels.
    pub fn to_px(&self, font_size: f32) -> f32 {
        match self {
            LineHeight::Normal => font_size * 1.2,
            LineHeight::Number(n) => font_size * n,
            LineHeight::Length(l) => *l,
        }
    }
}

impl fmt::Display for LineHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineHeight::Normal => write!(f, "normal"),
            LineHeight::Number(n) => write!(f, "{}", n),
            LineHeight::Length(l) => write!(f, "{}px", l),
        }
    }
}

/// Text alignment values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

impl fmt::Display for TextAlign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextAlign::Start => write!(f, "start"),
            TextAlign::End => write!(f, "end"),
            TextAlign::Left => write!(f, "left"),
            TextAlign::Right => write!(f, "right"),
            TextAlign::Center => write!(f, "center"),
            TextAlign::Justify => write!(f, "justify"),
        }
    }
}

/// Overflow property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
}

impl fmt::Display for Overflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Overflow::Visible => write!(f, "visible"),
            Overflow::Hidden => write!(f, "hidden"),
            Overflow::Scroll => write!(f, "scroll"),
            Overflow::Auto => write!(f, "auto"),
        }
    }
}

/// Visibility property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    Collapse,
}

impl Visibility {
    /// Check if the element should be painted.
    pub fn is_visible(&self) -> bool {
        matches!(self, Visibility::Visible)
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Visible => write!(f, "visible"),
            Visibility::Hidden => write!(f, "hidden"),
            Visibility::Collapse => write!(f, "collapse"),
        }
    }
}

/// Box-sizing property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxSizing {
    #[default]
    ContentBox,
    BorderBox,
}

impl fmt::Display for BoxSizing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoxSizing::ContentBox => write!(f, "content-box"),
            BoxSizing::BorderBox => write!(f, "border-box"),
        }
    }
}

/// White-space property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhiteSpace {
    #[default]
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
}

impl WhiteSpace {
    /// Check if whitespace should be preserved.
    pub fn preserves_whitespace(&self) -> bool {
        matches!(self, WhiteSpace::Pre | WhiteSpace::PreWrap)
    }

    /// Check if line breaks should be preserved.
    pub fn preserves_newlines(&self) -> bool {
        matches!(self, WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::PreLine)
    }

    /// Check if text should wrap.
    pub fn wraps(&self) -> bool {
        !matches!(self, WhiteSpace::NoWrap | WhiteSpace::Pre)
    }
}

impl fmt::Display for WhiteSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhiteSpace::Normal => write!(f, "normal"),
            WhiteSpace::NoWrap => write!(f, "nowrap"),
            WhiteSpace::Pre => write!(f, "pre"),
            WhiteSpace::PreWrap => write!(f, "pre-wrap"),
            WhiteSpace::PreLine => write!(f, "pre-line"),
        }
    }
}

/// Float property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Float {
    #[default]
    None,
    Left,
    Right,
}

impl Float {
    /// Check if this float value takes the element out of normal flow.
    pub fn is_floating(&self) -> bool {
        !matches!(self, Float::None)
    }
}

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Float::None => write!(f, "none"),
            Float::Left => write!(f, "left"),
            Float::Right => write!(f, "right"),
        }
    }
}

/// Clear property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Clear {
    #[default]
    None,
    Left,
    Right,
    Both,
}

impl fmt::Display for Clear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Clear::None => write!(f, "none"),
            Clear::Left => write!(f, "left"),
            Clear::Right => write!(f, "right"),
            Clear::Both => write!(f, "both"),
        }
    }
}

/// Vertical-align property values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Sub,
    Super,
    Top,
    TextTop,
    Middle,
    Bottom,
    TextBottom,
    Length(f32),
}

impl fmt::Display for VerticalAlign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerticalAlign::Baseline => write!(f, "baseline"),
            VerticalAlign::Sub => write!(f, "sub"),
            VerticalAlign::Super => write!(f, "super"),
            VerticalAlign::Top => write!(f, "top"),
            VerticalAlign::TextTop => write!(f, "text-top"),
            VerticalAlign::Middle => write!(f, "middle"),
            VerticalAlign::Bottom => write!(f, "bottom"),
            VerticalAlign::TextBottom => write!(f, "text-bottom"),
            VerticalAlign::Length(l) => write!(f, "{}px", l),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert!(Display::Block.generates_box());
        assert!(!Display::None.generates_box());
        assert!(Display::Block.is_block_level());
        assert!(Display::Inline.is_inline_level());
    }

    #[test]
    fn test_position() {
        assert!(!Position::Static.is_positioned());
        assert!(Position::Relative.is_positioned());
        assert!(Position::Absolute.is_out_of_flow());
        assert!(!Position::Relative.is_out_of_flow());
    }

    #[test]
    fn test_font_weight() {
        assert_eq!(FontWeight::NORMAL.value(), 400);
        assert!(FontWeight::BOLD.is_bold());
        assert!(!FontWeight::NORMAL.is_bold());
    }

    #[test]
    fn test_line_height() {
        assert_eq!(LineHeight::Normal.to_px(16.0), 19.2);
        assert_eq!(LineHeight::Number(1.5).to_px(16.0), 24.0);
        assert_eq!(LineHeight::Length(20.0).to_px(16.0), 20.0);
    }

    #[test]
    fn test_white_space() {
        assert!(!WhiteSpace::Normal.preserves_whitespace());
        assert!(WhiteSpace::Pre.preserves_whitespace());
        assert!(WhiteSpace::Normal.wraps());
        assert!(!WhiteSpace::NoWrap.wraps());
    }
}
