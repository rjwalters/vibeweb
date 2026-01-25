//! Display list generation and rasterization
//!
//! This crate provides CPU-based software rendering primitives for the
//! Vibeweb browser. It includes a framebuffer, color types, display lists,
//! text rendering, and basic drawing operations.
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
//!
//! # Text Rendering
//!
//! Text can be rendered using the `TextRenderer` or via display list commands:
//!
//! ```no_run
//! use vw_gfx::{Color, Framebuffer, TextRenderer};
//! use vw_fonts::FontCache;
//!
//! let mut fb = Framebuffer::new(800, 600);
//! fb.clear(Color::WHITE);
//!
//! let mut fonts = FontCache::new();
//! if let Some(font) = fonts.get_fallback() {
//!     let mut renderer = TextRenderer::new(&mut fb);
//!     // Use font.as_ab_glyph() to get the underlying ab_glyph font
//!     renderer.draw_text(font.as_ab_glyph(), 16.0, 50.0, 100.0, "Hello, World!", Color::BLACK);
//! }
//! ```
//!
//! # CSS Border Rendering
//!
//! The crate supports CSS-style borders with different widths, colors, and
//! styles per edge:
//!
//! ```
//! use vw_gfx::{Border, BorderEdge, BorderStyle, Color, Framebuffer, Rect};
//! use vw_gfx::border::draw_border;
//!
//! let mut fb = Framebuffer::new(800, 600);
//!
//! // Create a border with different colors per edge
//! let border = Border {
//!     top: BorderEdge::solid(2.0, Color::RED),
//!     right: BorderEdge::dashed(2.0, Color::GREEN),
//!     bottom: BorderEdge::dotted(2.0, Color::BLUE),
//!     left: BorderEdge::double(4.0, Color::BLACK),
//! };
//!
//! draw_border(&mut fb, Rect::new(10, 10, 100, 50), &border);
//! ```

pub mod border;
pub mod color;
pub mod display_list;
pub mod framebuffer;
pub mod rect;
pub mod text;

pub use border::{Border, BorderEdge, BorderStyle};
pub use color::Color;
pub use display_list::{BorderWidths, DisplayCommand, DisplayList};
pub use framebuffer::Framebuffer;
pub use rect::Rect;
pub use text::TextRenderer;
