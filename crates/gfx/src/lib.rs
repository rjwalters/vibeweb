//! Display list generation and rasterization
//!
//! This crate provides CPU-based software rendering primitives for the
//! Vibeweb browser. It includes a framebuffer, color types, and basic
//! drawing operations.

pub mod color;
pub mod framebuffer;
pub mod rect;

pub use color::Color;
pub use framebuffer::Framebuffer;
pub use rect::Rect;
