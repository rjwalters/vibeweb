//! CSS border rendering primitives
//!
//! This module provides types and functions for rendering CSS borders with
//! support for different widths, colors, and styles per edge.
//!
//! # Border Styles
//!
//! The following border styles are supported:
//! - `None`: No border rendered
//! - `Solid`: A solid line
//! - `Dashed`: A dashed line pattern
//! - `Dotted`: A dotted line pattern
//! - `Double`: Two parallel solid lines
//!
//! # Example
//!
//! ```
//! use vw_gfx::{Color, Border, BorderEdge, BorderStyle};
//!
//! // Create a uniform 2px solid black border
//! let border = Border::uniform(2.0, Color::BLACK, BorderStyle::Solid);
//!
//! // Create a border with different edges
//! let custom_border = Border {
//!     top: BorderEdge::solid(1.0, Color::RED),
//!     right: BorderEdge::solid(2.0, Color::GREEN),
//!     bottom: BorderEdge::solid(1.0, Color::BLUE),
//!     left: BorderEdge::solid(2.0, Color::BLACK),
//! };
//! ```

use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::rect::Rect;

/// Border rendering style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyle {
    /// No border (invisible)
    #[default]
    None,
    /// Solid line border
    Solid,
    /// Dashed line border (pattern: 3px on, 3px off)
    Dashed,
    /// Dotted line border (circular dots)
    Dotted,
    /// Double line border (two parallel lines)
    Double,
}

/// Border specification for one edge
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderEdge {
    /// Width of the border in pixels
    pub width: f32,
    /// Color of the border
    pub color: Color,
    /// Style of the border
    pub style: BorderStyle,
}

impl Default for BorderEdge {
    fn default() -> Self {
        Self {
            width: 0.0,
            color: Color::TRANSPARENT,
            style: BorderStyle::None,
        }
    }
}

impl BorderEdge {
    /// Create a new border edge with the given properties
    pub const fn new(width: f32, color: Color, style: BorderStyle) -> Self {
        Self {
            width,
            color,
            style,
        }
    }

    /// Create a solid border edge
    pub const fn solid(width: f32, color: Color) -> Self {
        Self {
            width,
            color,
            style: BorderStyle::Solid,
        }
    }

    /// Create a dashed border edge
    pub const fn dashed(width: f32, color: Color) -> Self {
        Self {
            width,
            color,
            style: BorderStyle::Dashed,
        }
    }

    /// Create a dotted border edge
    pub const fn dotted(width: f32, color: Color) -> Self {
        Self {
            width,
            color,
            style: BorderStyle::Dotted,
        }
    }

    /// Create a double border edge
    pub const fn double(width: f32, color: Color) -> Self {
        Self {
            width,
            color,
            style: BorderStyle::Double,
        }
    }

    /// Check if this edge is effectively invisible
    pub fn is_none(&self) -> bool {
        self.style == BorderStyle::None || self.width <= 0.0 || self.color.is_transparent()
    }
}

/// Complete border specification with independent edges
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Border {
    /// Top border edge
    pub top: BorderEdge,
    /// Right border edge
    pub right: BorderEdge,
    /// Bottom border edge
    pub bottom: BorderEdge,
    /// Left border edge
    pub left: BorderEdge,
}

impl Border {
    /// Create a uniform border with the same width, color, and style on all sides
    pub const fn uniform(width: f32, color: Color, style: BorderStyle) -> Self {
        let edge = BorderEdge::new(width, color, style);
        Self {
            top: edge,
            right: edge,
            bottom: edge,
            left: edge,
        }
    }

    /// Create a uniform solid border
    pub const fn solid(width: f32, color: Color) -> Self {
        Self::uniform(width, color, BorderStyle::Solid)
    }

    /// Check if all borders are none/invisible
    pub fn is_none(&self) -> bool {
        self.top.is_none() && self.right.is_none() && self.bottom.is_none() && self.left.is_none()
    }

    /// Get the total border widths as a tuple (top, right, bottom, left)
    pub fn widths(&self) -> (f32, f32, f32, f32) {
        (
            if self.top.is_none() { 0.0 } else { self.top.width },
            if self.right.is_none() { 0.0 } else { self.right.width },
            if self.bottom.is_none() { 0.0 } else { self.bottom.width },
            if self.left.is_none() { 0.0 } else { self.left.width },
        )
    }
}

/// Which side of the border is being rendered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSide {
    Top,
    Right,
    Bottom,
    Left,
}

/// Draw a border around a content rectangle.
///
/// The border is drawn INSIDE the given border box rectangle, following CSS box model.
pub fn draw_border(framebuffer: &mut Framebuffer, border_box: Rect, border: &Border) {
    if border.is_none() || border_box.is_empty() {
        return;
    }

    // Top border
    if !border.top.is_none() && border.top.width > 0.0 {
        draw_border_edge(
            framebuffer,
            border_box.x,
            border_box.y,
            border_box.width as f32,
            border.top.width,
            &border.top,
            EdgeSide::Top,
        );
    }

    // Bottom border
    if !border.bottom.is_none() && border.bottom.width > 0.0 {
        let bottom_y = border_box.y + border_box.height as i32 - border.bottom.width as i32;
        draw_border_edge(
            framebuffer,
            border_box.x,
            bottom_y,
            border_box.width as f32,
            border.bottom.width,
            &border.bottom,
            EdgeSide::Bottom,
        );
    }

    // Left border (between top and bottom)
    if !border.left.is_none() && border.left.width > 0.0 {
        let left_y = border_box.y + border.top.width as i32;
        let left_height = (border_box.height as f32 - border.top.width - border.bottom.width).max(0.0);
        if left_height > 0.0 {
            draw_border_edge(
                framebuffer,
                border_box.x,
                left_y,
                border.left.width,
                left_height,
                &border.left,
                EdgeSide::Left,
            );
        }
    }

    // Right border (between top and bottom)
    if !border.right.is_none() && border.right.width > 0.0 {
        let right_x = border_box.x + border_box.width as i32 - border.right.width as i32;
        let right_y = border_box.y + border.top.width as i32;
        let right_height = (border_box.height as f32 - border.top.width - border.bottom.width).max(0.0);
        if right_height > 0.0 {
            draw_border_edge(
                framebuffer,
                right_x,
                right_y,
                border.right.width,
                right_height,
                &border.right,
                EdgeSide::Right,
            );
        }
    }
}

/// Draw a single border edge
fn draw_border_edge(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: f32,
    height: f32,
    edge: &BorderEdge,
    side: EdgeSide,
) {
    match edge.style {
        BorderStyle::None => {}
        BorderStyle::Solid => {
            draw_solid_edge(framebuffer, x, y, width, height, edge.color);
        }
        BorderStyle::Dashed => {
            draw_dashed_edge(framebuffer, x, y, width, height, edge, side);
        }
        BorderStyle::Dotted => {
            draw_dotted_edge(framebuffer, x, y, width, height, edge, side);
        }
        BorderStyle::Double => {
            draw_double_edge(framebuffer, x, y, width, height, edge, side);
        }
    }
}

/// Draw a solid border edge
fn draw_solid_edge(framebuffer: &mut Framebuffer, x: i32, y: i32, width: f32, height: f32, color: Color) {
    let rect = Rect::new(x, y, width.round() as u32, height.round() as u32);
    framebuffer.fill_rect(rect, color);
}

/// Draw a dashed border edge
///
/// Uses a pattern of 3x the border width on, 3x the border width off.
fn draw_dashed_edge(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: f32,
    height: f32,
    edge: &BorderEdge,
    side: EdgeSide,
) {
    let dash_length = (edge.width * 3.0).max(3.0);
    let gap_length = dash_length;

    match side {
        EdgeSide::Top | EdgeSide::Bottom => {
            // Horizontal dashes
            let mut current_x = x as f32;
            let end_x = x as f32 + width;
            let mut drawing = true;

            while current_x < end_x {
                if drawing {
                    let dash_end = (current_x + dash_length).min(end_x);
                    let dash_width = dash_end - current_x;
                    if dash_width > 0.0 {
                        draw_solid_edge(framebuffer, current_x as i32, y, dash_width, height, edge.color);
                    }
                    current_x += dash_length;
                } else {
                    current_x += gap_length;
                }
                drawing = !drawing;
            }
        }
        EdgeSide::Left | EdgeSide::Right => {
            // Vertical dashes
            let mut current_y = y as f32;
            let end_y = y as f32 + height;
            let mut drawing = true;

            while current_y < end_y {
                if drawing {
                    let dash_end = (current_y + dash_length).min(end_y);
                    let dash_height = dash_end - current_y;
                    if dash_height > 0.0 {
                        draw_solid_edge(framebuffer, x, current_y as i32, width, dash_height, edge.color);
                    }
                    current_y += dash_length;
                } else {
                    current_y += gap_length;
                }
                drawing = !drawing;
            }
        }
    }
}

/// Draw a dotted border edge
///
/// Draws square dots with spacing equal to the border width.
fn draw_dotted_edge(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: f32,
    height: f32,
    edge: &BorderEdge,
    side: EdgeSide,
) {
    let dot_size = edge.width.max(1.0);
    let spacing = dot_size;

    match side {
        EdgeSide::Top | EdgeSide::Bottom => {
            // Horizontal dots
            let mut current_x = x as f32;
            let end_x = x as f32 + width;
            // Center dots vertically within the border height
            let dot_y = y as f32 + (height - dot_size) / 2.0;

            while current_x + dot_size <= end_x {
                draw_solid_edge(framebuffer, current_x as i32, dot_y as i32, dot_size, dot_size, edge.color);
                current_x += dot_size + spacing;
            }
        }
        EdgeSide::Left | EdgeSide::Right => {
            // Vertical dots
            let mut current_y = y as f32;
            let end_y = y as f32 + height;
            // Center dots horizontally within the border width
            let dot_x = x as f32 + (width - dot_size) / 2.0;

            while current_y + dot_size <= end_y {
                draw_solid_edge(framebuffer, dot_x as i32, current_y as i32, dot_size, dot_size, edge.color);
                current_y += dot_size + spacing;
            }
        }
    }
}

/// Draw a double border edge
///
/// Draws two parallel lines with a gap of 1/3 the total width.
fn draw_double_edge(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: f32,
    height: f32,
    edge: &BorderEdge,
    side: EdgeSide,
) {
    // For double borders, we need at least 3px to see the effect
    if edge.width < 3.0 {
        // Fall back to solid for very thin borders
        draw_solid_edge(framebuffer, x, y, width, height, edge.color);
        return;
    }

    // Divide into thirds: line, gap, line
    let line_width = (edge.width / 3.0).floor().max(1.0);
    let gap_width = edge.width - (line_width * 2.0);

    match side {
        EdgeSide::Top | EdgeSide::Bottom => {
            // Top/outer line
            draw_solid_edge(framebuffer, x, y, width, line_width, edge.color);
            // Bottom/inner line
            let inner_y = y + (line_width + gap_width) as i32;
            draw_solid_edge(framebuffer, x, inner_y, width, line_width, edge.color);
        }
        EdgeSide::Left => {
            // Outer line (leftmost)
            draw_solid_edge(framebuffer, x, y, line_width, height, edge.color);
            // Inner line
            let inner_x = x + (line_width + gap_width) as i32;
            draw_solid_edge(framebuffer, inner_x, y, line_width, height, edge.color);
        }
        EdgeSide::Right => {
            // Inner line (leftmost in the right border)
            draw_solid_edge(framebuffer, x, y, line_width, height, edge.color);
            // Outer line (rightmost)
            let outer_x = x + (line_width + gap_width) as i32;
            draw_solid_edge(framebuffer, outer_x, y, line_width, height, edge.color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_style_default() {
        assert_eq!(BorderStyle::default(), BorderStyle::None);
    }

    #[test]
    fn test_border_edge_default() {
        let edge = BorderEdge::default();
        assert_eq!(edge.width, 0.0);
        assert_eq!(edge.style, BorderStyle::None);
        assert!(edge.is_none());
    }

    #[test]
    fn test_border_edge_solid() {
        let edge = BorderEdge::solid(2.0, Color::RED);
        assert_eq!(edge.width, 2.0);
        assert_eq!(edge.color, Color::RED);
        assert_eq!(edge.style, BorderStyle::Solid);
        assert!(!edge.is_none());
    }

    #[test]
    fn test_border_edge_dashed() {
        let edge = BorderEdge::dashed(3.0, Color::BLUE);
        assert_eq!(edge.style, BorderStyle::Dashed);
    }

    #[test]
    fn test_border_edge_dotted() {
        let edge = BorderEdge::dotted(1.0, Color::GREEN);
        assert_eq!(edge.style, BorderStyle::Dotted);
    }

    #[test]
    fn test_border_edge_double() {
        let edge = BorderEdge::double(4.0, Color::BLACK);
        assert_eq!(edge.style, BorderStyle::Double);
    }

    #[test]
    fn test_border_edge_is_none_zero_width() {
        let edge = BorderEdge::solid(0.0, Color::RED);
        assert!(edge.is_none());
    }

    #[test]
    fn test_border_edge_is_none_transparent() {
        let edge = BorderEdge::solid(2.0, Color::TRANSPARENT);
        assert!(edge.is_none());
    }

    #[test]
    fn test_border_uniform() {
        let border = Border::uniform(2.0, Color::BLACK, BorderStyle::Solid);
        assert_eq!(border.top.width, 2.0);
        assert_eq!(border.right.width, 2.0);
        assert_eq!(border.bottom.width, 2.0);
        assert_eq!(border.left.width, 2.0);
        assert!(!border.is_none());
    }

    #[test]
    fn test_border_solid() {
        let border = Border::solid(3.0, Color::RED);
        assert_eq!(border.top.style, BorderStyle::Solid);
        assert_eq!(border.top.color, Color::RED);
    }

    #[test]
    fn test_border_is_none() {
        assert!(Border::default().is_none());
        assert!(!Border::solid(1.0, Color::BLACK).is_none());
    }

    #[test]
    fn test_border_widths() {
        let border = Border {
            top: BorderEdge::solid(1.0, Color::BLACK),
            right: BorderEdge::solid(2.0, Color::BLACK),
            bottom: BorderEdge::solid(3.0, Color::BLACK),
            left: BorderEdge::solid(4.0, Color::BLACK),
        };
        assert_eq!(border.widths(), (1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn test_draw_solid_border() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);

        let border = Border::solid(2.0, Color::BLACK);
        draw_border(&mut fb, Rect::new(10, 10, 20, 20), &border);

        // Top border
        assert_eq!(fb.get_pixel(15, 10), Color::BLACK);
        assert_eq!(fb.get_pixel(15, 11), Color::BLACK);
        // Inside (should be white, not filled)
        assert_eq!(fb.get_pixel(15, 15), Color::WHITE);
        // Bottom border
        assert_eq!(fb.get_pixel(15, 28), Color::BLACK);
        assert_eq!(fb.get_pixel(15, 29), Color::BLACK);
    }

    #[test]
    fn test_draw_border_different_colors() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);

        let border = Border {
            top: BorderEdge::solid(2.0, Color::RED),
            right: BorderEdge::solid(2.0, Color::GREEN),
            bottom: BorderEdge::solid(2.0, Color::BLUE),
            left: BorderEdge::solid(2.0, Color::BLACK),
        };
        draw_border(&mut fb, Rect::new(10, 10, 20, 20), &border);

        // Check each edge has correct color
        assert_eq!(fb.get_pixel(15, 10), Color::RED);    // top
        assert_eq!(fb.get_pixel(28, 15), Color::GREEN);  // right
        assert_eq!(fb.get_pixel(15, 29), Color::BLUE);   // bottom
        assert_eq!(fb.get_pixel(10, 15), Color::BLACK);  // left
    }

    #[test]
    fn test_draw_border_different_widths() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);

        let border = Border {
            top: BorderEdge::solid(1.0, Color::BLACK),
            right: BorderEdge::solid(2.0, Color::BLACK),
            bottom: BorderEdge::solid(3.0, Color::BLACK),
            left: BorderEdge::solid(4.0, Color::BLACK),
        };
        draw_border(&mut fb, Rect::new(10, 10, 30, 30), &border);

        // Top: 1px
        assert_eq!(fb.get_pixel(20, 10), Color::BLACK);
        assert_eq!(fb.get_pixel(20, 11), Color::WHITE); // Just below 1px top border

        // Left: 4px wide
        assert_eq!(fb.get_pixel(10, 20), Color::BLACK);
        assert_eq!(fb.get_pixel(13, 20), Color::BLACK);
        assert_eq!(fb.get_pixel(14, 20), Color::WHITE); // Just after 4px left border
    }

    #[test]
    fn test_draw_dashed_border() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);

        let border = Border::uniform(2.0, Color::BLACK, BorderStyle::Dashed);
        draw_border(&mut fb, Rect::new(10, 10, 50, 50), &border);

        // Should have some black (dash) and some white (gap) along top
        let mut found_black = false;
        let mut found_white = false;
        for x in 10..60 {
            let pixel = fb.get_pixel(x, 10);
            if pixel == Color::BLACK {
                found_black = true;
            }
            if pixel == Color::WHITE {
                found_white = true;
            }
        }
        assert!(found_black, "Dashed border should have black pixels");
        assert!(found_white, "Dashed border should have gaps");
    }

    #[test]
    fn test_draw_dotted_border() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);

        let border = Border::uniform(2.0, Color::BLACK, BorderStyle::Dotted);
        draw_border(&mut fb, Rect::new(10, 10, 50, 50), &border);

        // Should have some black (dots) and some white (spacing) along top
        let mut found_black = false;
        let mut found_white = false;
        for x in 10..60 {
            let pixel = fb.get_pixel(x, 10);
            if pixel == Color::BLACK {
                found_black = true;
            }
            if pixel == Color::WHITE {
                found_white = true;
            }
        }
        assert!(found_black, "Dotted border should have black pixels");
        assert!(found_white, "Dotted border should have spacing");
    }

    #[test]
    fn test_draw_double_border() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);

        // Need at least 3px for double effect
        let border = Border::uniform(6.0, Color::BLACK, BorderStyle::Double);
        draw_border(&mut fb, Rect::new(10, 10, 50, 50), &border);

        // Top border should have black, gap, black pattern vertically
        let top_outer = fb.get_pixel(30, 10); // First line
        let top_gap = fb.get_pixel(30, 12);   // Gap area
        let top_inner = fb.get_pixel(30, 14); // Second line

        assert_eq!(top_outer, Color::BLACK, "Outer line should be black");
        assert_eq!(top_gap, Color::WHITE, "Gap should be white");
        assert_eq!(top_inner, Color::BLACK, "Inner line should be black");
    }

    #[test]
    fn test_draw_border_empty_rect() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);

        let border = Border::solid(2.0, Color::BLACK);
        draw_border(&mut fb, Rect::new(10, 10, 0, 0), &border);

        // Nothing should change
        assert_eq!(fb.get_pixel(10, 10), Color::WHITE);
    }

    #[test]
    fn test_draw_border_none_style() {
        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);

        let border = Border::uniform(2.0, Color::BLACK, BorderStyle::None);
        draw_border(&mut fb, Rect::new(10, 10, 20, 20), &border);

        // Nothing should be drawn
        assert_eq!(fb.get_pixel(10, 10), Color::WHITE);
        assert_eq!(fb.get_pixel(15, 10), Color::WHITE);
    }
}
