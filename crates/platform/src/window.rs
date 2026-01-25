//! Window abstraction over winit

use crate::error::PlatformError;
use crate::event::{ElementState, Event, KeyCode, Modifiers, MouseButton};
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
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
        let event_loop =
            EventLoop::new().map_err(|e| PlatformError::EventLoopCreation(e.to_string()))?;

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
            cursor_position: (0.0, 0.0),
            modifiers: Modifiers::default(),
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
    /// Track last known cursor position for mouse button events
    cursor_position: (f64, f64),
    /// Track modifier key state
    modifiers: Modifiers,
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

            // Mouse events
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = (position.x, position.y);
                (self.callback)(
                    &ctx,
                    Event::MouseMoved {
                        x: position.x,
                        y: position.y,
                    },
                );
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = map_mouse_button(button);
                let state = map_element_state(state);
                let (x, y) = self.cursor_position;
                (self.callback)(&ctx, Event::MouseButton { button, state, x, y });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        // Line delta: multiply by a reasonable pixels-per-line factor
                        (x as f64 * 20.0, y as f64 * 20.0)
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x, pos.y),
                };
                (self.callback)(&ctx, Event::MouseWheel { delta_x, delta_y });
            }

            // Keyboard events
            WindowEvent::ModifiersChanged(new_modifiers) => {
                let state = new_modifiers.state();
                self.modifiers = Modifiers {
                    shift: state.shift_key(),
                    ctrl: state.control_key(),
                    alt: state.alt_key(),
                };
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let key = map_keycode(&event.physical_key);
                let state = map_element_state(event.state);
                (self.callback)(
                    &ctx,
                    Event::KeyboardInput {
                        key,
                        state,
                        modifiers: self.modifiers,
                    },
                );

                // Also emit CharacterInput for text typed (only on press, with text)
                if event.state == winit::event::ElementState::Pressed {
                    if let Some(text) = &event.text {
                        for c in text.chars() {
                            // Filter out control characters
                            if !c.is_control() {
                                (self.callback)(&ctx, Event::CharacterInput { character: c });
                            }
                        }
                    }
                }
            }

            _ => {}
        }
    }
}

/// Map winit mouse button to our MouseButton type
fn map_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Other(n) => MouseButton::Other(n),
        winit::event::MouseButton::Back => MouseButton::Other(4),
        winit::event::MouseButton::Forward => MouseButton::Other(5),
    }
}

/// Map winit element state to our ElementState type
fn map_element_state(state: winit::event::ElementState) -> ElementState {
    match state {
        winit::event::ElementState::Pressed => ElementState::Pressed,
        winit::event::ElementState::Released => ElementState::Released,
    }
}

/// Map winit physical key to our KeyCode type
fn map_keycode(key: &PhysicalKey) -> KeyCode {
    use winit::keyboard::KeyCode as WinitKeyCode;

    match key {
        PhysicalKey::Code(code) => match code {
            // Navigation
            WinitKeyCode::ArrowUp => KeyCode::ArrowUp,
            WinitKeyCode::ArrowDown => KeyCode::ArrowDown,
            WinitKeyCode::ArrowLeft => KeyCode::ArrowLeft,
            WinitKeyCode::ArrowRight => KeyCode::ArrowRight,
            WinitKeyCode::Home => KeyCode::Home,
            WinitKeyCode::End => KeyCode::End,
            WinitKeyCode::PageUp => KeyCode::PageUp,
            WinitKeyCode::PageDown => KeyCode::PageDown,

            // Editing
            WinitKeyCode::Backspace => KeyCode::Backspace,
            WinitKeyCode::Delete => KeyCode::Delete,
            WinitKeyCode::Enter => KeyCode::Enter,
            WinitKeyCode::Tab => KeyCode::Tab,
            WinitKeyCode::Escape => KeyCode::Escape,

            // Letters
            WinitKeyCode::KeyA => KeyCode::KeyA,
            WinitKeyCode::KeyB => KeyCode::KeyB,
            WinitKeyCode::KeyC => KeyCode::KeyC,
            WinitKeyCode::KeyD => KeyCode::KeyD,
            WinitKeyCode::KeyE => KeyCode::KeyE,
            WinitKeyCode::KeyF => KeyCode::KeyF,
            WinitKeyCode::KeyG => KeyCode::KeyG,
            WinitKeyCode::KeyH => KeyCode::KeyH,
            WinitKeyCode::KeyI => KeyCode::KeyI,
            WinitKeyCode::KeyJ => KeyCode::KeyJ,
            WinitKeyCode::KeyK => KeyCode::KeyK,
            WinitKeyCode::KeyL => KeyCode::KeyL,
            WinitKeyCode::KeyM => KeyCode::KeyM,
            WinitKeyCode::KeyN => KeyCode::KeyN,
            WinitKeyCode::KeyO => KeyCode::KeyO,
            WinitKeyCode::KeyP => KeyCode::KeyP,
            WinitKeyCode::KeyQ => KeyCode::KeyQ,
            WinitKeyCode::KeyR => KeyCode::KeyR,
            WinitKeyCode::KeyS => KeyCode::KeyS,
            WinitKeyCode::KeyT => KeyCode::KeyT,
            WinitKeyCode::KeyU => KeyCode::KeyU,
            WinitKeyCode::KeyV => KeyCode::KeyV,
            WinitKeyCode::KeyW => KeyCode::KeyW,
            WinitKeyCode::KeyX => KeyCode::KeyX,
            WinitKeyCode::KeyY => KeyCode::KeyY,
            WinitKeyCode::KeyZ => KeyCode::KeyZ,

            // Numbers
            WinitKeyCode::Digit0 => KeyCode::Digit0,
            WinitKeyCode::Digit1 => KeyCode::Digit1,
            WinitKeyCode::Digit2 => KeyCode::Digit2,
            WinitKeyCode::Digit3 => KeyCode::Digit3,
            WinitKeyCode::Digit4 => KeyCode::Digit4,
            WinitKeyCode::Digit5 => KeyCode::Digit5,
            WinitKeyCode::Digit6 => KeyCode::Digit6,
            WinitKeyCode::Digit7 => KeyCode::Digit7,
            WinitKeyCode::Digit8 => KeyCode::Digit8,
            WinitKeyCode::Digit9 => KeyCode::Digit9,

            // Modifiers
            WinitKeyCode::ShiftLeft => KeyCode::ShiftLeft,
            WinitKeyCode::ShiftRight => KeyCode::ShiftRight,
            WinitKeyCode::ControlLeft => KeyCode::ControlLeft,
            WinitKeyCode::ControlRight => KeyCode::ControlRight,
            WinitKeyCode::AltLeft => KeyCode::AltLeft,
            WinitKeyCode::AltRight => KeyCode::AltRight,

            // Function keys
            WinitKeyCode::F1 => KeyCode::F1,
            WinitKeyCode::F2 => KeyCode::F2,
            WinitKeyCode::F3 => KeyCode::F3,
            WinitKeyCode::F4 => KeyCode::F4,
            WinitKeyCode::F5 => KeyCode::F5,
            WinitKeyCode::F6 => KeyCode::F6,
            WinitKeyCode::F7 => KeyCode::F7,
            WinitKeyCode::F8 => KeyCode::F8,
            WinitKeyCode::F9 => KeyCode::F9,
            WinitKeyCode::F10 => KeyCode::F10,
            WinitKeyCode::F11 => KeyCode::F11,
            WinitKeyCode::F12 => KeyCode::F12,

            // Other
            WinitKeyCode::Space => KeyCode::Space,

            _ => KeyCode::Unknown,
        },
        PhysicalKey::Unidentified(_) => KeyCode::Unknown,
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
