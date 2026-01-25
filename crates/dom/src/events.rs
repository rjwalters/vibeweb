//! DOM Event Propagation System
//!
//! This module provides event handling infrastructure for the vibeweb browser,
//! enabling the routing of platform-level input events to DOM elements. It
//! implements a subset of the W3C DOM Events specification, focused on the
//! needs of M5 (Navigation UX).
//!
//! # Overview
//!
//! The event system provides:
//! - **Hit testing**: Finding the DOM element at screen coordinates
//! - **Event propagation**: Bubbling events up the DOM tree
//! - **Focus management**: Tracking which element receives keyboard events
//!
//! # Event Flow
//!
//! When a platform event (e.g., mouse click) occurs:
//!
//! ```text
//! Platform Event → Hit Test → Create DomEvent → Bubble Up Tree
//!      ↓              ↓            ↓                   ↓
//! MouseButton    Find NodeId   DomEvent::Click   Parent → Parent → Root
//! ```
//!
//! # Example
//!
//! ```
//! use vw_dom::events::{DomEvent, FocusManager};
//! use vw_dom::Document;
//!
//! // Create document and focus manager
//! let mut doc = Document::new();
//! let mut focus_manager = FocusManager::new();
//!
//! // Create an element in the document
//! let div = doc.create_element("div");
//! doc.append_child(doc.root(), div);
//!
//! // Create a click event targeting the element
//! let event = DomEvent::click(div, 100.0, 200.0);
//! assert_eq!(event.target, div);
//!
//! // Focus management
//! focus_manager.focus(div);
//! assert!(focus_manager.is_focused(div));
//! ```

use crate::node::NodeId;
use crate::Document;

/// The type of DOM event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomEventType {
    /// Mouse click event (mousedown + mouseup on same element)
    Click,
    /// Mouse button pressed
    MouseDown,
    /// Mouse button released
    MouseUp,
    /// Mouse cursor entered an element
    MouseEnter,
    /// Mouse cursor left an element
    MouseLeave,
    /// Mouse cursor moved over an element
    MouseMove,
    /// Keyboard key pressed
    KeyDown,
    /// Keyboard key released
    KeyUp,
    /// Element received focus
    Focus,
    /// Element lost focus
    Blur,
    /// Mouse wheel scrolled
    Scroll,
}

impl DomEventType {
    /// Returns true if this event type bubbles up the DOM tree.
    ///
    /// Most events bubble, but focus/blur and mouseenter/mouseleave do not.
    pub fn bubbles(&self) -> bool {
        match self {
            DomEventType::Click
            | DomEventType::MouseDown
            | DomEventType::MouseUp
            | DomEventType::MouseMove
            | DomEventType::KeyDown
            | DomEventType::KeyUp
            | DomEventType::Scroll => true,
            // These events don't bubble per W3C spec
            DomEventType::MouseEnter
            | DomEventType::MouseLeave
            | DomEventType::Focus
            | DomEventType::Blur => false,
        }
    }

    /// Returns true if this event has a default action that can be prevented.
    pub fn cancelable(&self) -> bool {
        match self {
            DomEventType::Click
            | DomEventType::MouseDown
            | DomEventType::MouseUp
            | DomEventType::KeyDown
            | DomEventType::KeyUp
            | DomEventType::Scroll => true,
            DomEventType::MouseEnter
            | DomEventType::MouseLeave
            | DomEventType::MouseMove
            | DomEventType::Focus
            | DomEventType::Blur => false,
        }
    }
}

/// A DOM event with target and metadata.
///
/// DOM events are higher-level than platform events and include
/// information about the target element and propagation state.
#[derive(Debug, Clone)]
pub struct DomEvent {
    /// The type of event.
    pub event_type: DomEventType,
    /// The element that is the target of the event.
    pub target: NodeId,
    /// The element currently processing the event (changes during bubbling).
    pub current_target: NodeId,
    /// X coordinate relative to the viewport (for mouse events).
    pub client_x: f32,
    /// Y coordinate relative to the viewport (for mouse events).
    pub client_y: f32,
    /// Key code (for keyboard events).
    pub key: Option<String>,
    /// Whether the default action has been prevented.
    pub default_prevented: bool,
    /// Whether propagation has been stopped.
    pub propagation_stopped: bool,
    /// Scroll delta X (for scroll events).
    pub delta_x: f32,
    /// Scroll delta Y (for scroll events).
    pub delta_y: f32,
}

impl DomEvent {
    /// Create a new click event.
    pub fn click(target: NodeId, x: f32, y: f32) -> Self {
        Self {
            event_type: DomEventType::Click,
            target,
            current_target: target,
            client_x: x,
            client_y: y,
            key: None,
            default_prevented: false,
            propagation_stopped: false,
            delta_x: 0.0,
            delta_y: 0.0,
        }
    }

    /// Create a new mouse down event.
    pub fn mouse_down(target: NodeId, x: f32, y: f32) -> Self {
        Self {
            event_type: DomEventType::MouseDown,
            target,
            current_target: target,
            client_x: x,
            client_y: y,
            key: None,
            default_prevented: false,
            propagation_stopped: false,
            delta_x: 0.0,
            delta_y: 0.0,
        }
    }

    /// Create a new mouse up event.
    pub fn mouse_up(target: NodeId, x: f32, y: f32) -> Self {
        Self {
            event_type: DomEventType::MouseUp,
            target,
            current_target: target,
            client_x: x,
            client_y: y,
            key: None,
            default_prevented: false,
            propagation_stopped: false,
            delta_x: 0.0,
            delta_y: 0.0,
        }
    }

    /// Create a new mouse move event.
    pub fn mouse_move(target: NodeId, x: f32, y: f32) -> Self {
        Self {
            event_type: DomEventType::MouseMove,
            target,
            current_target: target,
            client_x: x,
            client_y: y,
            key: None,
            default_prevented: false,
            propagation_stopped: false,
            delta_x: 0.0,
            delta_y: 0.0,
        }
    }

    /// Create a new key down event.
    pub fn key_down(target: NodeId, key: String) -> Self {
        Self {
            event_type: DomEventType::KeyDown,
            target,
            current_target: target,
            client_x: 0.0,
            client_y: 0.0,
            key: Some(key),
            default_prevented: false,
            propagation_stopped: false,
            delta_x: 0.0,
            delta_y: 0.0,
        }
    }

    /// Create a new key up event.
    pub fn key_up(target: NodeId, key: String) -> Self {
        Self {
            event_type: DomEventType::KeyUp,
            target,
            current_target: target,
            client_x: 0.0,
            client_y: 0.0,
            key: Some(key),
            default_prevented: false,
            propagation_stopped: false,
            delta_x: 0.0,
            delta_y: 0.0,
        }
    }

    /// Create a new focus event.
    pub fn focus(target: NodeId) -> Self {
        Self {
            event_type: DomEventType::Focus,
            target,
            current_target: target,
            client_x: 0.0,
            client_y: 0.0,
            key: None,
            default_prevented: false,
            propagation_stopped: false,
            delta_x: 0.0,
            delta_y: 0.0,
        }
    }

    /// Create a new blur event.
    pub fn blur(target: NodeId) -> Self {
        Self {
            event_type: DomEventType::Blur,
            target,
            current_target: target,
            client_x: 0.0,
            client_y: 0.0,
            key: None,
            default_prevented: false,
            propagation_stopped: false,
            delta_x: 0.0,
            delta_y: 0.0,
        }
    }

    /// Create a new scroll event.
    pub fn scroll(target: NodeId, delta_x: f32, delta_y: f32) -> Self {
        Self {
            event_type: DomEventType::Scroll,
            target,
            current_target: target,
            client_x: 0.0,
            client_y: 0.0,
            key: None,
            default_prevented: false,
            propagation_stopped: false,
            delta_x,
            delta_y,
        }
    }

    /// Prevent the default action for this event.
    ///
    /// This is used to stop native behaviors like link navigation.
    pub fn prevent_default(&mut self) {
        if self.event_type.cancelable() {
            self.default_prevented = true;
        }
    }

    /// Stop the event from bubbling to parent elements.
    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    /// Returns true if the event's default action was prevented.
    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented
    }

    /// Returns true if propagation was stopped.
    pub fn is_propagation_stopped(&self) -> bool {
        self.propagation_stopped
    }
}

/// Result of handling an event on a specific element.
#[derive(Debug, Clone)]
pub enum EventAction {
    /// No special action needed.
    None,
    /// Navigate to a URL (from clicking a link).
    Navigate(String),
    /// Submit a form.
    SubmitForm,
    /// Focus an element.
    FocusElement(NodeId),
}

/// Manages focus state for keyboard event routing.
///
/// The focus manager tracks which element currently has focus and
/// provides methods for moving focus between focusable elements.
#[derive(Debug, Clone)]
pub struct FocusManager {
    /// The currently focused element, if any.
    focused: Option<NodeId>,
    /// The element that was last focused before current (for blur events).
    previous_focused: Option<NodeId>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Create a new focus manager with no focused element.
    pub fn new() -> Self {
        Self {
            focused: None,
            previous_focused: None,
        }
    }

    /// Get the currently focused element.
    pub fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    /// Get the previously focused element.
    pub fn previous_focused(&self) -> Option<NodeId> {
        self.previous_focused
    }

    /// Set focus to an element.
    ///
    /// Returns the previously focused element if any.
    pub fn focus(&mut self, node: NodeId) -> Option<NodeId> {
        let previous = self.focused;
        self.previous_focused = previous;
        self.focused = Some(node);
        previous
    }

    /// Remove focus from all elements.
    ///
    /// Returns the previously focused element if any.
    pub fn blur(&mut self) -> Option<NodeId> {
        let previous = self.focused;
        self.previous_focused = previous;
        self.focused = None;
        previous
    }

    /// Check if an element is focused.
    pub fn is_focused(&self, node: NodeId) -> bool {
        self.focused == Some(node)
    }

    /// Check if any element has focus.
    pub fn has_focus(&self) -> bool {
        self.focused.is_some()
    }
}

/// Checks if an element is focusable.
///
/// Focusable elements include:
/// - Links (`<a>` with href)
/// - Form controls (`<input>`, `<button>`, `<select>`, `<textarea>`)
/// - Elements with tabindex attribute
pub fn is_focusable(doc: &Document, node: NodeId) -> bool {
    let Some(n) = doc.get(node) else {
        return false;
    };

    let Some(element) = n.as_element() else {
        return false;
    };

    // Check for tabindex attribute
    if element.get_attribute("tabindex").is_some() {
        return true;
    }

    // Check for natively focusable elements
    let tag = element.tag_name.to_ascii_lowercase();
    match tag.as_str() {
        "a" => element.get_attribute("href").is_some(),
        "button" | "input" | "select" | "textarea" => true,
        _ => false,
    }
}

/// Checks if an element is interactive (responds to clicks).
///
/// Interactive elements include links, buttons, and form controls.
pub fn is_interactive(doc: &Document, node: NodeId) -> bool {
    let Some(n) = doc.get(node) else {
        return false;
    };

    let Some(element) = n.as_element() else {
        return false;
    };

    let tag = element.tag_name.to_ascii_lowercase();
    match tag.as_str() {
        "a" => element.get_attribute("href").is_some(),
        "button" | "input" | "select" | "textarea" | "summary" => true,
        _ => false,
    }
}

/// Determines the default action for clicking an element.
///
/// Returns an EventAction describing what should happen when this
/// element is clicked (e.g., navigation for links).
pub fn get_click_action(doc: &Document, node: NodeId) -> EventAction {
    let Some(n) = doc.get(node) else {
        return EventAction::None;
    };

    let Some(element) = n.as_element() else {
        return EventAction::None;
    };

    let tag = element.tag_name.to_ascii_lowercase();
    match tag.as_str() {
        "a" => {
            if let Some(href) = element.get_attribute("href") {
                EventAction::Navigate(href.to_string())
            } else {
                EventAction::None
            }
        }
        "button" => {
            // Check if button is in a form (submit action)
            // For now, just return None - form handling comes later
            EventAction::None
        }
        "input" => {
            // Focus the input
            EventAction::FocusElement(node)
        }
        _ => EventAction::None,
    }
}

/// Propagates an event through the DOM tree.
///
/// This implements the bubbling phase of DOM event propagation.
/// The event starts at the target and bubbles up to ancestor elements.
///
/// # Arguments
///
/// * `doc` - The DOM document
/// * `event` - The event to propagate (will be mutated during propagation)
/// * `handler` - A callback invoked for each element in the propagation path
///
/// # Returns
///
/// The final event state after propagation completes.
pub fn propagate_event<F>(doc: &Document, event: &mut DomEvent, mut handler: F)
where
    F: FnMut(&Document, NodeId, &mut DomEvent),
{
    // First, invoke handler on target
    handler(doc, event.target, event);

    // If event doesn't bubble, we're done
    if !event.event_type.bubbles() || event.propagation_stopped {
        return;
    }

    // Bubble up through ancestors
    for ancestor in doc.ancestors(event.target) {
        event.current_target = ancestor;
        handler(doc, ancestor, event);

        if event.propagation_stopped {
            break;
        }
    }
}

/// Finds the innermost element at the given coordinates in the layout tree.
///
/// This performs hit testing by traversing the layout tree in reverse paint
/// order (back to front), finding the topmost element that contains the point.
///
/// # Arguments
///
/// * `layout_root` - The root of the layout tree
/// * `x` - X coordinate in viewport space
/// * `y` - Y coordinate in viewport space
///
/// # Returns
///
/// The NodeId of the element at the coordinates, or None if no element found.
pub fn hit_test(layout_root: &vw_layout::LayoutBox, x: f32, y: f32) -> Option<NodeId> {
    hit_test_recursive(layout_root, x, y)
}

fn hit_test_recursive(layout_box: &vw_layout::LayoutBox, x: f32, y: f32) -> Option<NodeId> {
    let border_box = layout_box.border_box();

    // Check if point is within this box's border box
    if !point_in_rect(x, y, &border_box) {
        return None;
    }

    // Check children in reverse order (last child is painted on top)
    for child in layout_box.children.iter().rev() {
        if let Some(hit) = hit_test_recursive(child, x, y) {
            return Some(hit);
        }
    }

    // If no child was hit, return this box if it has a node_id
    layout_box.node_id.map(|id| NodeId(id.0))
}

/// Check if a point is inside a rectangle.
fn point_in_rect(x: f32, y: f32, rect: &vw_layout::Rect) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// Finds all elements at the given coordinates, from innermost to outermost.
///
/// Unlike `hit_test`, this returns the full chain of elements at the point,
/// which is useful for determining event delegation targets.
pub fn hit_test_all(layout_root: &vw_layout::LayoutBox, x: f32, y: f32) -> Vec<NodeId> {
    let mut result = Vec::new();
    hit_test_all_recursive(layout_root, x, y, &mut result);
    result
}

fn hit_test_all_recursive(
    layout_box: &vw_layout::LayoutBox,
    x: f32,
    y: f32,
    result: &mut Vec<NodeId>,
) {
    let border_box = layout_box.border_box();

    // Check if point is within this box's border box
    if !point_in_rect(x, y, &border_box) {
        return;
    }

    // First, recurse into children (innermost first in result)
    for child in layout_box.children.iter().rev() {
        hit_test_all_recursive(child, x, y, result);
    }

    // Add this node if it has an ID
    if let Some(id) = layout_box.node_id {
        result.push(NodeId(id.0));
    }
}

/// Finds the nearest interactive element at the given coordinates.
///
/// This is useful for determining click targets - it finds the first
/// interactive element (link, button, etc.) in the hit test chain.
pub fn find_interactive_at(
    doc: &Document,
    layout_root: &vw_layout::LayoutBox,
    x: f32,
    y: f32,
) -> Option<NodeId> {
    let elements = hit_test_all(layout_root, x, y);
    elements.into_iter().find(|&id| is_interactive(doc, id))
}

/// Finds the nearest focusable element at the given coordinates.
pub fn find_focusable_at(
    doc: &Document,
    layout_root: &vw_layout::LayoutBox,
    x: f32,
    y: f32,
) -> Option<NodeId> {
    let elements = hit_test_all(layout_root, x, y);
    elements.into_iter().find(|&id| is_focusable(doc, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vw_layout::{BoxType, Dimensions, LayoutBox as LBox, NodeId as LayoutNodeId, Rect};

    // Helper to create a layout box at specific position
    fn make_box(id: usize, x: f32, y: f32, width: f32, height: f32) -> LBox {
        let mut b = LBox::new(BoxType::Block);
        b.node_id = Some(LayoutNodeId(id));
        b.dimensions = Dimensions {
            content: Rect::new(x, y, width, height),
            ..Default::default()
        };
        b
    }

    #[test]
    fn test_dom_event_creation() {
        let target = NodeId(1);
        let event = DomEvent::click(target, 100.0, 200.0);

        assert_eq!(event.event_type, DomEventType::Click);
        assert_eq!(event.target, target);
        assert_eq!(event.client_x, 100.0);
        assert_eq!(event.client_y, 200.0);
        assert!(!event.default_prevented);
        assert!(!event.propagation_stopped);
    }

    #[test]
    fn test_event_prevent_default() {
        let mut event = DomEvent::click(NodeId(1), 0.0, 0.0);
        assert!(!event.is_default_prevented());

        event.prevent_default();
        assert!(event.is_default_prevented());
    }

    #[test]
    fn test_event_stop_propagation() {
        let mut event = DomEvent::click(NodeId(1), 0.0, 0.0);
        assert!(!event.is_propagation_stopped());

        event.stop_propagation();
        assert!(event.is_propagation_stopped());
    }

    #[test]
    fn test_event_type_bubbles() {
        assert!(DomEventType::Click.bubbles());
        assert!(DomEventType::MouseDown.bubbles());
        assert!(DomEventType::KeyDown.bubbles());
        assert!(!DomEventType::Focus.bubbles());
        assert!(!DomEventType::Blur.bubbles());
        assert!(!DomEventType::MouseEnter.bubbles());
    }

    #[test]
    fn test_focus_manager() {
        let mut fm = FocusManager::new();
        assert!(!fm.has_focus());

        let node1 = NodeId(1);
        let node2 = NodeId(2);

        // Focus node1
        assert_eq!(fm.focus(node1), None);
        assert!(fm.has_focus());
        assert!(fm.is_focused(node1));
        assert!(!fm.is_focused(node2));

        // Focus node2, should return previous
        assert_eq!(fm.focus(node2), Some(node1));
        assert!(fm.is_focused(node2));
        assert!(!fm.is_focused(node1));
        assert_eq!(fm.previous_focused(), Some(node1));

        // Blur
        assert_eq!(fm.blur(), Some(node2));
        assert!(!fm.has_focus());
    }

    #[test]
    fn test_hit_test_simple() {
        // Create a simple layout tree:
        // root (0,0, 800x600)
        //   -> child (100, 100, 200x100)
        let mut root = make_box(0, 0.0, 0.0, 800.0, 600.0);
        let child = make_box(1, 100.0, 100.0, 200.0, 100.0);
        root.children.push(child);

        // Hit the child
        let result = hit_test(&root, 150.0, 150.0);
        assert_eq!(result, Some(NodeId(1)));

        // Hit outside child but in root
        let result = hit_test(&root, 50.0, 50.0);
        assert_eq!(result, Some(NodeId(0)));

        // Miss completely
        let result = hit_test(&root, 1000.0, 1000.0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_hit_test_nested() {
        // root (0,0, 800x600)
        //   -> parent (50, 50, 400x300)
        //     -> child (100, 100, 100x50)
        let mut root = make_box(0, 0.0, 0.0, 800.0, 600.0);
        let mut parent = make_box(1, 50.0, 50.0, 400.0, 300.0);
        let child = make_box(2, 100.0, 100.0, 100.0, 50.0);
        parent.children.push(child);
        root.children.push(parent);

        // Hit innermost
        let result = hit_test(&root, 120.0, 120.0);
        assert_eq!(result, Some(NodeId(2)));

        // Hit parent but not child
        let result = hit_test(&root, 60.0, 60.0);
        assert_eq!(result, Some(NodeId(1)));
    }

    #[test]
    fn test_hit_test_overlapping_siblings() {
        // Later siblings are painted on top, so last child wins
        // root (0,0, 800x600)
        //   -> child1 (100, 100, 200x200)
        //   -> child2 (150, 150, 200x200) - overlaps child1, painted on top
        let mut root = make_box(0, 0.0, 0.0, 800.0, 600.0);
        let child1 = make_box(1, 100.0, 100.0, 200.0, 200.0);
        let child2 = make_box(2, 150.0, 150.0, 200.0, 200.0);
        root.children.push(child1);
        root.children.push(child2);

        // In overlap region, child2 (last) should win
        let result = hit_test(&root, 200.0, 200.0);
        assert_eq!(result, Some(NodeId(2)));

        // Only in child1
        let result = hit_test(&root, 110.0, 110.0);
        assert_eq!(result, Some(NodeId(1)));
    }

    #[test]
    fn test_hit_test_all() {
        let mut root = make_box(0, 0.0, 0.0, 800.0, 600.0);
        let mut parent = make_box(1, 50.0, 50.0, 400.0, 300.0);
        let child = make_box(2, 100.0, 100.0, 100.0, 50.0);
        parent.children.push(child);
        root.children.push(parent);

        // Hit innermost, should get all three
        let result = hit_test_all(&root, 120.0, 120.0);
        assert_eq!(result, vec![NodeId(2), NodeId(1), NodeId(0)]);
    }

    #[test]
    fn test_is_focusable() {
        let mut doc = Document::new();

        // Create a link with href
        let link = doc.create_element_with_attributes("a", vec![("href".into(), "#".into())]);
        doc.append_child(doc.root(), link);
        assert!(is_focusable(&doc, link));

        // Create a link without href
        let link_no_href = doc.create_element("a");
        doc.append_child(doc.root(), link_no_href);
        assert!(!is_focusable(&doc, link_no_href));

        // Create form controls
        let button = doc.create_element("button");
        doc.append_child(doc.root(), button);
        assert!(is_focusable(&doc, button));

        let input = doc.create_element("input");
        doc.append_child(doc.root(), input);
        assert!(is_focusable(&doc, input));

        // Create div with tabindex
        let div_tab =
            doc.create_element_with_attributes("div", vec![("tabindex".into(), "0".into())]);
        doc.append_child(doc.root(), div_tab);
        assert!(is_focusable(&doc, div_tab));

        // Plain div
        let div = doc.create_element("div");
        doc.append_child(doc.root(), div);
        assert!(!is_focusable(&doc, div));
    }

    #[test]
    fn test_is_interactive() {
        let mut doc = Document::new();

        let link = doc.create_element_with_attributes("a", vec![("href".into(), "#".into())]);
        doc.append_child(doc.root(), link);
        assert!(is_interactive(&doc, link));

        let button = doc.create_element("button");
        doc.append_child(doc.root(), button);
        assert!(is_interactive(&doc, button));

        let div = doc.create_element("div");
        doc.append_child(doc.root(), div);
        assert!(!is_interactive(&doc, div));
    }

    #[test]
    fn test_get_click_action() {
        let mut doc = Document::new();

        // Link navigation
        let link = doc.create_element_with_attributes(
            "a",
            vec![("href".into(), "https://example.com".into())],
        );
        doc.append_child(doc.root(), link);
        match get_click_action(&doc, link) {
            EventAction::Navigate(url) => assert_eq!(url, "https://example.com"),
            _ => panic!("Expected Navigate action"),
        }

        // Input focus
        let input = doc.create_element("input");
        doc.append_child(doc.root(), input);
        match get_click_action(&doc, input) {
            EventAction::FocusElement(id) => assert_eq!(id, input),
            _ => panic!("Expected FocusElement action"),
        }

        // Plain div
        let div = doc.create_element("div");
        doc.append_child(doc.root(), div);
        assert!(matches!(get_click_action(&doc, div), EventAction::None));
    }

    #[test]
    fn test_propagate_event() {
        let mut doc = Document::new();
        // Build: root -> html -> body -> div
        let html = doc.create_element("html");
        let body = doc.create_element("body");
        let div = doc.create_element("div");

        doc.append_child(doc.root(), html);
        doc.append_child(html, body);
        doc.append_child(body, div);

        let mut event = DomEvent::click(div, 0.0, 0.0);
        let mut visited = Vec::new();

        propagate_event(&doc, &mut event, |_doc, node_id, _event| {
            visited.push(node_id);
        });

        // Should visit: div -> body -> html -> root
        assert_eq!(visited, vec![div, body, html, doc.root()]);
    }

    #[test]
    fn test_propagate_event_stop() {
        let mut doc = Document::new();
        let html = doc.create_element("html");
        let body = doc.create_element("body");
        let div = doc.create_element("div");

        doc.append_child(doc.root(), html);
        doc.append_child(html, body);
        doc.append_child(body, div);

        let mut event = DomEvent::click(div, 0.0, 0.0);
        let mut visited = Vec::new();

        propagate_event(&doc, &mut event, |_doc, node_id, event| {
            visited.push(node_id);
            // Stop at body
            if node_id == body {
                event.stop_propagation();
            }
        });

        // Should stop at body
        assert_eq!(visited, vec![div, body]);
    }

    #[test]
    fn test_non_bubbling_event() {
        let mut doc = Document::new();
        let html = doc.create_element("html");
        let body = doc.create_element("body");

        doc.append_child(doc.root(), html);
        doc.append_child(html, body);

        // Focus doesn't bubble
        let mut event = DomEvent::focus(body);
        let mut visited = Vec::new();

        propagate_event(&doc, &mut event, |_doc, node_id, _event| {
            visited.push(node_id);
        });

        // Should only visit target
        assert_eq!(visited, vec![body]);
    }

    #[test]
    fn test_point_in_rect() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);

        // Inside
        assert!(point_in_rect(50.0, 40.0, &rect));

        // On edges
        assert!(point_in_rect(10.0, 20.0, &rect)); // top-left
        assert!(!point_in_rect(110.0, 70.0, &rect)); // bottom-right (exclusive)

        // Outside
        assert!(!point_in_rect(5.0, 40.0, &rect)); // left
        assert!(!point_in_rect(120.0, 40.0, &rect)); // right
        assert!(!point_in_rect(50.0, 15.0, &rect)); // above
        assert!(!point_in_rect(50.0, 80.0, &rect)); // below
    }
}
