//! Display list generation and rasterization
//!
//! This crate provides CPU-based software rendering primitives for the
//! Vibeweb browser. It includes a framebuffer, color types, display lists,
//! and basic drawing operations.
//!
//! # Display Lists
//!
//! The display list is the bridge between layout and rendering. It serializes
//! drawing commands that can be efficiently executed against a framebuffer.
//!
//! ```
//! use vw_gfx::{Color, DisplayList, DisplayCommand, Rect, Framebuffer};
//!
//! // Build a display list
//! let mut list = DisplayList::new();
//! list.push_solid_color(Rect::new(10, 10, 100, 50), Color::RED);
//!
//! // Paint to a framebuffer
//! let mut fb = Framebuffer::new(800, 600);
//! list.paint(&mut fb);
//! ```

pub mod color;
pub mod display_list;
pub mod framebuffer;
pub mod rect;

pub use color::Color;
pub use display_list::{BorderWidths, DisplayCommand, DisplayList};
pub use framebuffer::Framebuffer;
pub use rect::Rect;
