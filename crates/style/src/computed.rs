//! Computed style representation.
//!
//! This module defines the `ComputedStyle` struct which holds the final
//! computed values for all CSS properties that affect layout and rendering.

use crate::properties::*;
use crate::values::*;
use std::fmt;

/// The computed style for a DOM node.
///
/// This contains the final, resolved values of all CSS properties
/// after cascade, inheritance, and value resolution have been applied.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    // === Box Model ===
    /// The display property determines how the element generates boxes.
    pub display: Display,

    /// The position property determines the positioning scheme.
    pub position: Position,

    /// Width of the content box.
    pub width: LengthOrAuto,

    /// Height of the content box.
    pub height: LengthOrAuto,

    /// Minimum width of the content box.
    pub min_width: LengthOrAuto,

    /// Minimum height of the content box.
    pub min_height: LengthOrAuto,

    /// Maximum width of the content box.
    pub max_width: LengthOrAuto,

    /// Maximum height of the content box.
    pub max_height: LengthOrAuto,

    /// Margin (top, right, bottom, left).
    pub margin: Sides<LengthOrAuto>,

    /// Padding (top, right, bottom, left).
    pub padding: Sides<Length>,

    /// Border width (top, right, bottom, left).
    pub border_width: Sides<Length>,

    /// Box sizing model.
    pub box_sizing: BoxSizing,

    // === Colors ===
    /// Foreground (text) color.
    pub color: Color,

    /// Background color.
    pub background_color: Color,

    /// Border color.
    pub border_color: Color,

    // === Typography ===
    /// Font size in pixels (always resolved to px).
    pub font_size: f32,

    /// Font family names in preference order.
    pub font_family: Vec<String>,

    /// Font weight.
    pub font_weight: FontWeight,

    /// Line height.
    pub line_height: LineHeight,

    /// Text alignment.
    pub text_align: TextAlign,

    /// White space handling.
    pub white_space: WhiteSpace,

    // === Positioning ===
    /// Top offset for positioned elements.
    pub top: LengthOrAuto,

    /// Right offset for positioned elements.
    pub right: LengthOrAuto,

    /// Bottom offset for positioned elements.
    pub bottom: LengthOrAuto,

    /// Left offset for positioned elements.
    pub left: LengthOrAuto,

    /// Z-index for stacking context.
    pub z_index: Option<i32>,

    // === Visual ===
    /// Overflow handling.
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,

    /// Visibility.
    pub visibility: Visibility,

    /// Opacity (0.0 to 1.0).
    pub opacity: f32,

    // === Float ===
    /// Float property.
    pub float: Float,

    /// Clear property.
    pub clear: Clear,

    /// Vertical alignment for inline elements.
    pub vertical_align: VerticalAlign,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        ComputedStyle {
            // Box model
            display: Display::default(),
            position: Position::default(),
            width: LengthOrAuto::Auto,
            height: LengthOrAuto::Auto,
            min_width: LengthOrAuto::Auto,
            min_height: LengthOrAuto::Auto,
            max_width: LengthOrAuto::Auto,
            max_height: LengthOrAuto::Auto,
            margin: Sides::default(),
            padding: Sides::default(),
            border_width: Sides::default(),
            box_sizing: BoxSizing::default(),

            // Colors
            color: Color::BLACK,
            background_color: Color::TRANSPARENT,
            border_color: Color::BLACK,

            // Typography
            font_size: 16.0, // Default browser font size
            font_family: vec!["serif".to_string()],
            font_weight: FontWeight::default(),
            line_height: LineHeight::default(),
            text_align: TextAlign::default(),
            white_space: WhiteSpace::default(),

            // Positioning
            top: LengthOrAuto::Auto,
            right: LengthOrAuto::Auto,
            bottom: LengthOrAuto::Auto,
            left: LengthOrAuto::Auto,
            z_index: None,

            // Visual
            overflow_x: Overflow::default(),
            overflow_y: Overflow::default(),
            visibility: Visibility::default(),
            opacity: 1.0,

            // Float
            float: Float::default(),
            clear: Clear::default(),
            vertical_align: VerticalAlign::default(),
        }
    }
}

impl ComputedStyle {
    /// Create a new computed style with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this element generates a box (i.e., not display: none).
    pub fn generates_box(&self) -> bool {
        self.display.generates_box()
    }

    /// Check if this element is visible.
    pub fn is_visible(&self) -> bool {
        self.visibility.is_visible() && self.opacity > 0.0
    }

    /// Check if this element is a block-level element.
    pub fn is_block(&self) -> bool {
        self.display.is_block_level()
    }

    /// Check if this element is inline-level.
    pub fn is_inline(&self) -> bool {
        self.display.is_inline_level()
    }

    /// Check if this element is positioned (not static).
    pub fn is_positioned(&self) -> bool {
        self.position.is_positioned()
    }

    /// Check if this element is out of the normal document flow.
    pub fn is_out_of_flow(&self) -> bool {
        self.position.is_out_of_flow() || self.float.is_floating()
    }

    /// Get the resolved line height in pixels.
    pub fn line_height_px(&self) -> f32 {
        self.line_height.to_px(self.font_size)
    }

    /// Create a style inheriting inheritable properties from a parent.
    pub fn inherit_from(parent: &ComputedStyle) -> Self {
        // Inherited properties (CSS spec defines which are inherited)
        ComputedStyle {
            color: parent.color,
            font_size: parent.font_size,
            font_family: parent.font_family.clone(),
            font_weight: parent.font_weight,
            line_height: parent.line_height,
            text_align: parent.text_align,
            white_space: parent.white_space,
            visibility: parent.visibility,
            ..Default::default()
        }
    }

    /// Apply a display value computed from HTML element semantics.
    pub fn with_display(mut self, display: Display) -> Self {
        self.display = display;
        self
    }
}

impl fmt::Display for ComputedStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ComputedStyle {{")?;
        writeln!(f, "  display: {}", self.display)?;
        writeln!(f, "  position: {}", self.position)?;
        writeln!(f, "  width: {}", self.width)?;
        writeln!(f, "  height: {}", self.height)?;
        writeln!(f, "  color: {}", self.color)?;
        writeln!(f, "  background-color: {}", self.background_color)?;
        writeln!(f, "  font-size: {}px", self.font_size)?;
        writeln!(f, "  font-family: {:?}", self.font_family)?;
        writeln!(f, "  font-weight: {}", self.font_weight)?;
        writeln!(f, "  line-height: {}", self.line_height)?;
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_style() {
        let style = ComputedStyle::default();
        assert_eq!(style.display, Display::Block);
        assert_eq!(style.position, Position::Static);
        assert_eq!(style.font_size, 16.0);
        assert!(style.generates_box());
    }

    #[test]
    fn test_inheritance() {
        let parent = ComputedStyle {
            color: Color::rgb(255, 0, 0),
            font_size: 20.0,
            font_family: vec!["Arial".to_string()],
            ..ComputedStyle::default()
        };

        let child = ComputedStyle::inherit_from(&parent);

        // Inherited properties should match parent
        assert_eq!(child.color, parent.color);
        assert_eq!(child.font_size, parent.font_size);
        assert_eq!(child.font_family, parent.font_family);

        // Non-inherited properties should be default
        assert_eq!(child.background_color, Color::TRANSPARENT);
        assert_eq!(child.margin, Sides::default());
    }

    #[test]
    fn test_visibility_checks() {
        let mut style = ComputedStyle::default();
        assert!(style.is_visible());

        style.visibility = Visibility::Hidden;
        assert!(!style.is_visible());

        style.visibility = Visibility::Visible;
        style.opacity = 0.0;
        assert!(!style.is_visible());
    }

    #[test]
    fn test_flow_checks() {
        let mut style = ComputedStyle::default();
        assert!(!style.is_out_of_flow());

        style.position = Position::Absolute;
        assert!(style.is_out_of_flow());

        style.position = Position::Static;
        style.float = Float::Left;
        assert!(style.is_out_of_flow());
    }
}
