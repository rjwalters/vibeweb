//! Window abstraction over winit

use crate::error::PlatformError;
use crate::event::Event;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window as WinitWindow, WindowAttributes, WindowId};

/// Window configuration for creating a new window
pub struct WindowConfig {
    /// Window title
    pub title: String,
    /// Initial window width in pixels
    pub width: u32,
    /// Initial window height in pixels
    pub height: u32,
}

impl WindowConfig {
    /// Create a new window configuration
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Vibeweb Browser".to_string(),
            width: 800,
            height: 600,
        }
    }
}

/// A platform window that can run an event loop
pub struct Window {
    event_loop: EventLoop<()>,
    config: WindowConfig,
}

impl Window {
    /// Create a new window with the given configuration
    pub fn new(config: WindowConfig) -> Result<Self, PlatformError> {
        let event_loop = EventLoop::new()
            .map_err(|e| PlatformError::EventLoopCreation(e.to_string()))?;

        Ok(Self { event_loop, config })
    }

    /// Run the window event loop with the given callback
    ///
    /// The callback is invoked for each event. The callback receives a mutable
    /// reference to a WindowContext for requesting redraws and a reference to
    /// a softbuffer Surface for presenting pixels.
    pub fn run<F>(self, callback: F) -> Result<(), PlatformError>
    where
        F: FnMut(&WindowContext, Event) + 'static,
    {
        let config = self.config;
        let mut app = App {
            config,
            window: None,
            callback: Box::new(callback),
        };

        self.event_loop
            .run_app(&mut app)
            .map_err(|e| PlatformError::EventLoopCreation(e.to_string()))?;

        Ok(())
    }
}

/// Context provided to the event callback for window operations
pub struct WindowContext {
    window: Arc<WinitWindow>,
}

impl WindowContext {
    /// Request a redraw of the window
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// Get the current window size
    pub fn size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    /// Get the raw window handle for graphics integration
    pub fn window(&self) -> &Arc<WinitWindow> {
        &self.window
    }
}

struct App<F>
where
    F: FnMut(&WindowContext, Event),
{
    config: WindowConfig,
    window: Option<Arc<WinitWindow>>,
    callback: Box<F>,
}

impl<F> ApplicationHandler for App<F>
where
    F: FnMut(&WindowContext, Event),
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let size = PhysicalSize::new(self.config.width, self.config.height);
            let attrs = WindowAttributes::default()
                .with_title(&self.config.title)
                .with_inner_size(size);

            match event_loop.create_window(attrs) {
                Ok(window) => {
                    self.window = Some(Arc::new(window));
                }
                Err(e) => {
                    eprintln!("Failed to create window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else {
            return;
        };

        let ctx = WindowContext {
            window: Arc::clone(window),
        };

        match event {
            WindowEvent::CloseRequested => {
                (self.callback)(&ctx, Event::CloseRequested);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                // Ensure minimum size
                let width = NonZeroU32::new(size.width).map(|n| n.get()).unwrap_or(1);
                let height = NonZeroU32::new(size.height).map(|n| n.get()).unwrap_or(1);
                (self.callback)(&ctx, Event::Resize { width, height });
            }
            WindowEvent::RedrawRequested => {
                (self.callback)(&ctx, Event::Redraw);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "Vibeweb Browser");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
    }

    #[test]
    fn window_config_new() {
        let config = WindowConfig::new("Test Window", 1024, 768);
        assert_eq!(config.title, "Test Window");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
    }
}
