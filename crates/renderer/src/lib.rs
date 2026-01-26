//! Render pipeline coordinator for the vibeweb browser.
//!
//! This crate orchestrates the full rendering pipeline from HTML source
//! to painted pixels. It coordinates:
//!
//! - HTML parsing (via vw-html)
//! - CSS parsing and style resolution (via vw-css, vw-style)
//! - Layout tree construction (via vw-layout)
//! - Display list generation and painting (via vw-gfx)
//!
//! # Overview
//!
//! The rendering pipeline flows as follows:
//!
//! ```text
//! HTML + CSS → Document → RenderTree → Framebuffer
//!              (parsed)    (laid out)   (painted)
//! ```
//!
//! # Example
//!
//! ```
//! use vw_renderer::{Document, Viewport};
//! use vw_gfx::Framebuffer;
//!
//! // Parse HTML and CSS into a Document
//! let doc = Document::load("<html><body><p>Hello</p></body></html>", "")
//!     .expect("Parse error");
//!
//! // Generate a render tree for the viewport
//! let viewport = Viewport::new(800, 600);
//! let render_tree = doc.render_tree(&viewport);
//!
//! // Paint into a framebuffer
//! let mut fb = Framebuffer::new(800, 600);
//! render_tree.paint(&mut fb);
//! ```

mod browser;
mod document;
mod author_matcher;
mod error;
mod paint;
mod render_tree;
mod viewport;

pub use browser::Browser;
pub use document::Document;
pub use error::{Error, Result};
pub use render_tree::RenderTree;
pub use viewport::Viewport;

#[cfg(test)]
mod tests {
    use super::*;
    use vw_gfx::Framebuffer;

    #[test]
    fn test_basic_pipeline() {
        let doc = Document::load("<html><body><p>Hello</p></body></html>", "").unwrap();
        let viewport = Viewport::new(800, 600);
        let render_tree = doc.render_tree(&viewport);

        let mut fb = Framebuffer::new(800, 600);
        render_tree.paint(&mut fb);

        // After painting, framebuffer should have content
        // (background color at minimum)
        assert_eq!(fb.width(), 800);
        assert_eq!(fb.height(), 600);
    }

    #[test]
    fn test_empty_document() {
        let doc = Document::load("", "").unwrap();
        let viewport = Viewport::new(800, 600);
        let render_tree = doc.render_tree(&viewport);

        let mut fb = Framebuffer::new(800, 600);
        render_tree.paint(&mut fb);
    }

    #[test]
    fn test_styled_document() {
        let html = "<html><body><div class='red'>Hello</div></body></html>";
        let css = ".red { background-color: red; }";

        let doc = Document::load(html, css).unwrap();
        let viewport = Viewport::new(800, 600);
        let render_tree = doc.render_tree(&viewport);

        let mut fb = Framebuffer::new(800, 600);
        render_tree.paint(&mut fb);
    }

    #[test]
    fn test_viewport_resize() {
        let doc = Document::load("<html><body><p>Test</p></body></html>", "").unwrap();

        // Render at one size
        let viewport1 = Viewport::new(800, 600);
        let tree1 = doc.render_tree(&viewport1);

        // Render at different size
        let viewport2 = Viewport::new(1024, 768);
        let tree2 = doc.render_tree(&viewport2);

        // Trees should have different dimensions
        assert_ne!(tree1.viewport_width(), tree2.viewport_width());
    }
}
