//! Display list for rendering layout trees.
//!
//! A display list is a serialized representation of drawing commands generated
//! from the layout tree. It enables:
//!
//! - **Separation of concerns**: Layout produces display commands; rendering consumes them
//! - **Efficient invalidation**: Only repaint changed regions (future optimization)
//! - **Painter's algorithm**: Correct stacking order for overlapping content
//! - **Future optimization**: Display list caching, GPU upload, incremental updates
//!
//! # Example
//!
//! ```
//! use vw_gfx::{Color, DisplayList, DisplayCommand, Rect, Framebuffer};
//!
//! // Build a display list
//! let mut list = DisplayList::new();
//! list.push(DisplayCommand::SolidColor {
//!     rect: Rect::new(10, 10, 100, 50),
//!     color: Color::RED,
//! });
//!
//! // Paint to a framebuffer
//! let mut fb = Framebuffer::new(800, 600);
//! list.paint(&mut fb);
//! ```

use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::rect::Rect;

/// Edge widths for border rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BorderWidths {
    /// Top border width in pixels
    pub top: u32,
    /// Right border width in pixels
    pub right: u32,
    /// Bottom border width in pixels
    pub bottom: u32,
    /// Left border width in pixels
    pub left: u32,
}

impl BorderWidths {
    /// Create uniform border widths.
    pub const fn uniform(width: u32) -> Self {
        Self {
            top: width,
            right: width,
            bottom: width,
            left: width,
        }
    }

    /// Create border widths with individual values.
    pub const fn new(top: u32, right: u32, bottom: u32, left: u32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Check if all border widths are zero.
    pub const fn is_zero(&self) -> bool {
        self.top == 0 && self.right == 0 && self.bottom == 0 && self.left == 0
    }
}

/// A drawing command in the display list.
///
/// Commands are rendered in order, implementing the painter's algorithm
/// where later commands paint over earlier ones.
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayCommand {
    /// Fill a rectangle with a solid color.
    SolidColor {
        /// The rectangle to fill
        rect: Rect,
        /// The fill color
        color: Color,
    },

    /// Draw a border around a rectangle.
    ///
    /// The border is drawn inside the given rectangle, not outside.
    Border {
        /// The outer rectangle (border box)
        rect: Rect,
        /// Border widths for each edge
        widths: BorderWidths,
        /// The border color
        color: Color,
    },

    /// Draw text at a position.
    ///
    /// Note: Text rendering is not yet implemented in the M4 foundation.
    /// This command is included for API completeness and will be functional
    /// once font rendering is integrated.
    Text {
        /// X position of the text baseline
        x: i32,
        /// Y position of the text baseline
        y: i32,
        /// The text content to render
        text: String,
        /// The text color
        color: Color,
        /// Font size in pixels
        font_size: f32,
    },

    /// Push a clipping rectangle onto the clip stack.
    ///
    /// All subsequent commands will be clipped to this rectangle
    /// until a matching `PopClip` command is encountered.
    PushClip(Rect),

    /// Pop the current clipping rectangle from the clip stack.
    PopClip,
}

/// A display list is an ordered collection of drawing commands.
///
/// Commands are stored in paint order: first command is painted first
/// (at the back), last command is painted last (at the front).
#[derive(Debug, Clone, Default)]
pub struct DisplayList {
    commands: Vec<DisplayCommand>,
}

impl DisplayList {
    /// Create a new empty display list.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Create a display list with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            commands: Vec::with_capacity(capacity),
        }
    }

    /// Add a command to the display list.
    pub fn push(&mut self, command: DisplayCommand) {
        self.commands.push(command);
    }

    /// Add a solid color fill command.
    pub fn push_solid_color(&mut self, rect: Rect, color: Color) {
        self.commands
            .push(DisplayCommand::SolidColor { rect, color });
    }

    /// Add a border command.
    pub fn push_border(&mut self, rect: Rect, widths: BorderWidths, color: Color) {
        if !widths.is_zero() {
            self.commands.push(DisplayCommand::Border {
                rect,
                widths,
                color,
            });
        }
    }

    /// Add a text command.
    pub fn push_text(&mut self, x: i32, y: i32, text: String, color: Color, font_size: f32) {
        if !text.is_empty() {
            self.commands.push(DisplayCommand::Text {
                x,
                y,
                text,
                color,
                font_size,
            });
        }
    }

    /// Push a clip rectangle.
    pub fn push_clip(&mut self, rect: Rect) {
        self.commands.push(DisplayCommand::PushClip(rect));
    }

    /// Pop the current clip rectangle.
    pub fn pop_clip(&mut self) {
        self.commands.push(DisplayCommand::PopClip);
    }

    /// Get the number of commands in the display list.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the display list is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Iterate over the commands.
    pub fn iter(&self) -> impl Iterator<Item = &DisplayCommand> {
        self.commands.iter()
    }

    /// Clear all commands from the display list.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Append another display list to this one.
    pub fn append(&mut self, other: &mut DisplayList) {
        self.commands.append(&mut other.commands);
    }

    /// Paint the display list to a framebuffer.
    ///
    /// Commands are executed in order. Clipping is supported through
    /// `PushClip` and `PopClip` commands.
    pub fn paint(&self, framebuffer: &mut Framebuffer) {
        let mut clip_stack: Vec<Rect> = Vec::new();

        for command in &self.commands {
            match command {
                DisplayCommand::SolidColor { rect, color } => {
                    let clipped = clip_rect(*rect, &clip_stack);
                    if let Some(r) = clipped {
                        framebuffer.fill_rect(r, *color);
                    }
                }

                DisplayCommand::Border {
                    rect,
                    widths,
                    color,
                } => {
                    // Draw border as four rectangles (top, right, bottom, left)
                    self.paint_border(framebuffer, *rect, *widths, *color, &clip_stack);
                }

                DisplayCommand::Text {
                    x: _,
                    y: _,
                    text: _,
                    color: _,
                    font_size: _,
                } => {
                    // Text rendering is not yet implemented.
                    // This will be functional once font rendering is integrated.
                    // For now, this is a no-op placeholder.
                }

                DisplayCommand::PushClip(rect) => {
                    // Intersect with current clip if there is one
                    let effective_clip = if let Some(current) = clip_stack.last() {
                        intersect_rects(*current, *rect)
                    } else {
                        Some(*rect)
                    };
                    if let Some(clip) = effective_clip {
                        clip_stack.push(clip);
                    } else {
                        // Empty intersection - push an empty rect
                        clip_stack.push(Rect::new(0, 0, 0, 0));
                    }
                }

                DisplayCommand::PopClip => {
                    clip_stack.pop();
                }
            }
        }
    }

    /// Paint a border as four edge rectangles.
    fn paint_border(
        &self,
        framebuffer: &mut Framebuffer,
        rect: Rect,
        widths: BorderWidths,
        color: Color,
        clip_stack: &[Rect],
    ) {
        // Top border
        if widths.top > 0 {
            let top_rect = Rect::new(rect.x, rect.y, rect.width, widths.top);
            if let Some(clipped) = clip_rect(top_rect, clip_stack) {
                framebuffer.fill_rect(clipped, color);
            }
        }

        // Bottom border
        if widths.bottom > 0 {
            let bottom_y = rect.y + rect.height as i32 - widths.bottom as i32;
            let bottom_rect = Rect::new(rect.x, bottom_y, rect.width, widths.bottom);
            if let Some(clipped) = clip_rect(bottom_rect, clip_stack) {
                framebuffer.fill_rect(clipped, color);
            }
        }

        // Left border (between top and bottom)
        if widths.left > 0 {
            let left_y = rect.y + widths.top as i32;
            let left_height = rect
                .height
                .saturating_sub(widths.top)
                .saturating_sub(widths.bottom);
            if left_height > 0 {
                let left_rect = Rect::new(rect.x, left_y, widths.left, left_height);
                if let Some(clipped) = clip_rect(left_rect, clip_stack) {
                    framebuffer.fill_rect(clipped, color);
                }
            }
        }

        // Right border (between top and bottom)
        if widths.right > 0 {
            let right_x = rect.x + rect.width as i32 - widths.right as i32;
            let right_y = rect.y + widths.top as i32;
            let right_height = rect
                .height
                .saturating_sub(widths.top)
                .saturating_sub(widths.bottom);
            if right_height > 0 {
                let right_rect = Rect::new(right_x, right_y, widths.right, right_height);
                if let Some(clipped) = clip_rect(right_rect, clip_stack) {
                    framebuffer.fill_rect(clipped, color);
                }
            }
        }
    }
}

/// Clip a rectangle against the current clip stack.
///
/// Returns None if the rectangle is completely clipped out.
fn clip_rect(rect: Rect, clip_stack: &[Rect]) -> Option<Rect> {
    if clip_stack.is_empty() {
        return Some(rect);
    }

    // Use the top of the clip stack
    let clip = clip_stack.last()?;
    intersect_rects(*clip, rect)
}

/// Compute the intersection of two rectangles.
///
/// Returns None if the rectangles don't overlap.
fn intersect_rects(a: Rect, b: Rect) -> Option<Rect> {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = a.right().min(b.right());
    let y2 = a.bottom().min(b.bottom());

    if x1 < x2 && y1 < y2 {
        Some(Rect::new(x1, y1, (x2 - x1) as u32, (y2 - y1) as u32))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_list_new() {
        let list = DisplayList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_display_list_push() {
        let mut list = DisplayList::new();
        list.push_solid_color(Rect::new(0, 0, 100, 100), Color::RED);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_display_list_push_methods() {
        let mut list = DisplayList::new();

        list.push_solid_color(Rect::new(0, 0, 100, 100), Color::RED);
        list.push_border(
            Rect::new(10, 10, 80, 80),
            BorderWidths::uniform(2),
            Color::BLACK,
        );
        list.push_text(20, 50, "Hello".to_string(), Color::BLACK, 16.0);

        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_border_widths_zero() {
        assert!(BorderWidths::default().is_zero());
        assert!(!BorderWidths::uniform(1).is_zero());
        assert!(BorderWidths::uniform(0).is_zero());
    }

    #[test]
    fn test_empty_text_not_added() {
        let mut list = DisplayList::new();
        list.push_text(0, 0, "".to_string(), Color::BLACK, 16.0);
        assert!(list.is_empty());
    }

    #[test]
    fn test_zero_border_not_added() {
        let mut list = DisplayList::new();
        list.push_border(
            Rect::new(0, 0, 100, 100),
            BorderWidths::default(),
            Color::BLACK,
        );
        assert!(list.is_empty());
    }

    #[test]
    fn test_display_list_clear() {
        let mut list = DisplayList::new();
        list.push_solid_color(Rect::new(0, 0, 100, 100), Color::RED);
        assert!(!list.is_empty());

        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn test_display_list_append() {
        let mut list1 = DisplayList::new();
        list1.push_solid_color(Rect::new(0, 0, 50, 50), Color::RED);

        let mut list2 = DisplayList::new();
        list2.push_solid_color(Rect::new(50, 50, 50, 50), Color::BLUE);

        list1.append(&mut list2);
        assert_eq!(list1.len(), 2);
        assert!(list2.is_empty());
    }

    #[test]
    fn test_paint_solid_color() {
        let mut list = DisplayList::new();
        list.push_solid_color(Rect::new(10, 10, 5, 5), Color::RED);

        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);
        list.paint(&mut fb);

        // Check that the rectangle was painted
        assert_eq!(fb.get_pixel(12, 12), Color::RED);
        // Check that outside the rectangle is still white
        assert_eq!(fb.get_pixel(5, 5), Color::WHITE);
    }

    #[test]
    fn test_paint_border() {
        let mut list = DisplayList::new();
        list.push_border(
            Rect::new(10, 10, 20, 20),
            BorderWidths::uniform(2),
            Color::BLACK,
        );

        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);
        list.paint(&mut fb);

        // Top border
        assert_eq!(fb.get_pixel(15, 10), Color::BLACK);
        assert_eq!(fb.get_pixel(15, 11), Color::BLACK);
        // Inside (should be white, not filled by border)
        assert_eq!(fb.get_pixel(15, 15), Color::WHITE);
        // Bottom border
        assert_eq!(fb.get_pixel(15, 28), Color::BLACK);
        assert_eq!(fb.get_pixel(15, 29), Color::BLACK);
    }

    #[test]
    fn test_paint_with_clip() {
        let mut list = DisplayList::new();
        list.push_clip(Rect::new(5, 5, 10, 10)); // Clip region: 5,5 to 15,15
        list.push_solid_color(Rect::new(0, 0, 20, 20), Color::RED);
        list.pop_clip();

        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);
        list.paint(&mut fb);

        // Inside clip region
        assert_eq!(fb.get_pixel(10, 10), Color::RED);
        // Outside clip region (but inside the rect)
        assert_eq!(fb.get_pixel(2, 2), Color::WHITE);
        assert_eq!(fb.get_pixel(18, 18), Color::WHITE);
    }

    #[test]
    fn test_nested_clips() {
        let mut list = DisplayList::new();
        list.push_clip(Rect::new(0, 0, 20, 20)); // First clip: 0,0 to 20,20
        list.push_clip(Rect::new(10, 10, 20, 20)); // Second clip: 10,10 to 30,30
                                                   // Effective clip should be intersection: 10,10 to 20,20
        list.push_solid_color(Rect::new(0, 0, 40, 40), Color::RED);
        list.pop_clip();
        list.pop_clip();

        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);
        list.paint(&mut fb);

        // Inside intersection
        assert_eq!(fb.get_pixel(15, 15), Color::RED);
        // Outside intersection but inside first clip
        assert_eq!(fb.get_pixel(5, 5), Color::WHITE);
        // Outside intersection but inside second clip
        assert_eq!(fb.get_pixel(25, 25), Color::WHITE);
    }

    #[test]
    fn test_intersect_rects() {
        // Overlapping rectangles
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        let result = intersect_rects(a, b);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.x, 5);
        assert_eq!(r.y, 5);
        assert_eq!(r.width, 5);
        assert_eq!(r.height, 5);

        // Non-overlapping rectangles
        let c = Rect::new(0, 0, 5, 5);
        let d = Rect::new(10, 10, 5, 5);
        assert!(intersect_rects(c, d).is_none());
    }

    #[test]
    fn test_display_list_iter() {
        let mut list = DisplayList::new();
        list.push_solid_color(Rect::new(0, 0, 10, 10), Color::RED);
        list.push_solid_color(Rect::new(10, 10, 10, 10), Color::BLUE);

        let commands: Vec<_> = list.iter().collect();
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_painters_algorithm() {
        // Later commands should paint over earlier ones
        let mut list = DisplayList::new();
        list.push_solid_color(Rect::new(0, 0, 20, 20), Color::RED);
        list.push_solid_color(Rect::new(5, 5, 10, 10), Color::BLUE);

        let mut fb = Framebuffer::new(100, 100);
        fb.clear(Color::WHITE);
        list.paint(&mut fb);

        // The overlapping region should be blue (second command)
        assert_eq!(fb.get_pixel(10, 10), Color::BLUE);
        // The non-overlapping region of the first rect should be red
        assert_eq!(fb.get_pixel(2, 2), Color::RED);
        // Outside both should be white
        assert_eq!(fb.get_pixel(25, 25), Color::WHITE);
    }
}
