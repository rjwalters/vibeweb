//! Platform event types

/// Events that the window can dispatch to the application
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Window needs to be redrawn
    Redraw,

    /// Window was resized
    Resize {
        /// New width in pixels
        width: u32,
        /// New height in pixels
        height: u32,
    },

    /// User requested the window to close
    CloseRequested,
}
