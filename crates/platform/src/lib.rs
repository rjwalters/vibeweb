//! Window, input, timers, clipboard - thin OS abstraction
//!
//! This crate provides a platform abstraction layer for windowing and events.
//! It wraps winit to provide a simple, focused API for the Vibeweb browser.

pub mod error;
pub mod event;
pub mod window;

pub use error::PlatformError;
pub use event::Event;
pub use window::{Window, WindowConfig, WindowContext};

/// Re-export winit window type for softbuffer integration
pub use winit::window::Window as WinitWindow;
