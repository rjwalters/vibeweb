//! Platform event types

/// Mouse button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button (primary)
    Left,
    /// Right mouse button (secondary)
    Right,
    /// Middle mouse button (wheel click)
    Middle,
    /// Other mouse button with numeric identifier
    Other(u16),
}

/// Button/key state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementState {
    /// Button/key is pressed
    Pressed,
    /// Button/key is released
    Released,
}

/// Keyboard key codes (subset of common keys)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    // Navigation
    /// Up arrow key
    ArrowUp,
    /// Down arrow key
    ArrowDown,
    /// Left arrow key
    ArrowLeft,
    /// Right arrow key
    ArrowRight,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,

    // Editing
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Enter/Return key
    Enter,
    /// Tab key
    Tab,
    /// Escape key
    Escape,

    // Letters (A-Z)
    /// A key
    KeyA,
    /// B key
    KeyB,
    /// C key
    KeyC,
    /// D key
    KeyD,
    /// E key
    KeyE,
    /// F key
    KeyF,
    /// G key
    KeyG,
    /// H key
    KeyH,
    /// I key
    KeyI,
    /// J key
    KeyJ,
    /// K key
    KeyK,
    /// L key
    KeyL,
    /// M key
    KeyM,
    /// N key
    KeyN,
    /// O key
    KeyO,
    /// P key
    KeyP,
    /// Q key
    KeyQ,
    /// R key
    KeyR,
    /// S key
    KeyS,
    /// T key
    KeyT,
    /// U key
    KeyU,
    /// V key
    KeyV,
    /// W key
    KeyW,
    /// X key
    KeyX,
    /// Y key
    KeyY,
    /// Z key
    KeyZ,

    // Numbers
    /// 0 key
    Digit0,
    /// 1 key
    Digit1,
    /// 2 key
    Digit2,
    /// 3 key
    Digit3,
    /// 4 key
    Digit4,
    /// 5 key
    Digit5,
    /// 6 key
    Digit6,
    /// 7 key
    Digit7,
    /// 8 key
    Digit8,
    /// 9 key
    Digit9,

    // Modifiers (for detection)
    /// Left Shift key
    ShiftLeft,
    /// Right Shift key
    ShiftRight,
    /// Left Control key
    ControlLeft,
    /// Right Control key
    ControlRight,
    /// Left Alt key
    AltLeft,
    /// Right Alt key
    AltRight,

    // Function keys
    /// F1 key
    F1,
    /// F2 key
    F2,
    /// F3 key
    F3,
    /// F4 key
    F4,
    /// F5 key
    F5,
    /// F6 key
    F6,
    /// F7 key
    F7,
    /// F8 key
    F8,
    /// F9 key
    F9,
    /// F10 key
    F10,
    /// F11 key
    F11,
    /// F12 key
    F12,

    // Other
    /// Space key
    Space,
    /// Unknown or unmapped key
    Unknown,
}

/// Modifier key state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Shift key is held
    pub shift: bool,
    /// Control key is held
    pub ctrl: bool,
    /// Alt key is held
    pub alt: bool,
}

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

    // Mouse events

    /// Mouse cursor moved within the window
    MouseMoved {
        /// X coordinate in physical pixels
        x: f64,
        /// Y coordinate in physical pixels
        y: f64,
    },

    /// Mouse button was pressed or released
    MouseButton {
        /// Which button
        button: MouseButton,
        /// Pressed or released
        state: ElementState,
        /// X coordinate at time of event
        x: f64,
        /// Y coordinate at time of event
        y: f64,
    },

    /// Mouse wheel was scrolled
    MouseWheel {
        /// Horizontal scroll amount (positive = right)
        delta_x: f64,
        /// Vertical scroll amount (positive = down)
        delta_y: f64,
    },

    // Keyboard events

    /// Keyboard key was pressed or released
    KeyboardInput {
        /// Which key
        key: KeyCode,
        /// Pressed or released
        state: ElementState,
        /// Modifier keys held at time of event
        modifiers: Modifiers,
    },

    /// Character input (after IME processing)
    CharacterInput {
        /// The character that was typed
        character: char,
    },
}
