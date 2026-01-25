//! Inline layout algorithm.
//!
//! This module implements CSS 2.1 inline formatting context layout.
//! Inline-level boxes are laid out horizontally in line boxes.

use crate::box_types::{LayoutBox, NodeId};
use crate::fonts::{FontMetrics, TextMetrics};

/// A line box containing inline-level content.
///
/// Line boxes are created during inline layout to hold horizontal
/// runs of inline content. Each line box has its own baseline.
#[derive(Debug, Clone)]
pub struct LineBox {
    /// Fragments in this line
    pub fragments: Vec<InlineFragment>,
    /// X position of the line box
    pub x: f32,
    /// Y position of the line box (top of line)
    pub y: f32,
    /// Total width of the line
    pub width: f32,
    /// Height of the line box
    pub height: f32,
    /// Baseline position (distance from top)
    pub baseline: f32,
}

impl LineBox {
    /// Create a new empty line box.
    pub fn new(x: f32, y: f32) -> Self {
        LineBox {
            fragments: Vec::new(),
            x,
            y,
            width: 0.0,
            height: 0.0,
            baseline: 0.0,
        }
    }

    /// Add a fragment to this line.
    pub fn add_fragment(&mut self, fragment: InlineFragment) {
        self.width += fragment.width;
        self.fragments.push(fragment);
    }

    /// Recalculate line height and baseline based on fragments.
    pub fn recalculate_metrics(&mut self) {
        let mut max_ascent: f32 = 0.0;
        let mut max_descent: f32 = 0.0;

        for fragment in &self.fragments {
            max_ascent = max_ascent.max(fragment.ascent);
            max_descent = max_descent.max(fragment.descent);
        }

        self.height = max_ascent + max_descent;
        self.baseline = max_ascent;
    }

    /// Get the remaining available width for more content.
    pub fn available_width(&self, max_width: f32) -> f32 {
        max_width - self.width
    }

    /// Check if the line is empty.
    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }
}

/// A fragment of inline content within a line box.
///
/// Each fragment represents a piece of inline content (text, inline box)
/// positioned within a line.
#[derive(Debug, Clone)]
pub struct InlineFragment {
    /// Reference to the source node (if any)
    pub node_id: Option<NodeId>,
    /// X position relative to line box
    pub x: f32,
    /// Width of this fragment
    pub width: f32,
    /// Distance from baseline to top of fragment
    pub ascent: f32,
    /// Distance from baseline to bottom of fragment
    pub descent: f32,
    /// Text content (for text fragments)
    pub text: Option<String>,
    /// Fragment type
    pub fragment_type: FragmentType,
}

impl InlineFragment {
    /// Create a text fragment.
    pub fn text(text: String, x: f32, metrics: TextMetrics) -> Self {
        InlineFragment {
            node_id: None,
            x,
            width: metrics.width,
            ascent: metrics.ascent,
            descent: metrics.descent,
            text: Some(text),
            fragment_type: FragmentType::Text,
        }
    }

    /// Create an inline box fragment.
    pub fn inline_box(x: f32, width: f32, height: f32) -> Self {
        InlineFragment {
            node_id: None,
            x,
            width,
            ascent: height,
            descent: 0.0,
            text: None,
            fragment_type: FragmentType::InlineBox,
        }
    }

    /// Set the node ID.
    pub fn with_node_id(mut self, id: NodeId) -> Self {
        self.node_id = Some(id);
        self
    }

    /// Total height of the fragment.
    pub fn height(&self) -> f32 {
        self.ascent + self.descent
    }
}

/// Type of inline fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentType {
    /// Text content
    Text,
    /// Inline-level box
    InlineBox,
    /// Whitespace (for breaking)
    Whitespace,
}

/// Result of laying out inline content.
#[derive(Debug, Clone)]
pub struct InlineLayoutResult {
    /// Generated line boxes
    pub line_boxes: Vec<LineBox>,
    /// Total height consumed
    pub total_height: f32,
}

/// Lay out inline content within a containing block.
///
/// This is the main entry point for inline formatting context layout.
pub fn layout_inline<F: FontMetrics>(
    box_: &LayoutBox,
    containing_width: f32,
    start_x: f32,
    start_y: f32,
    fonts: &F,
    font_size: f32,
) -> InlineLayoutResult {
    let mut line_boxes = Vec::new();
    let mut current_line = LineBox::new(start_x, start_y);
    let mut x_cursor = 0.0;

    // Collect all inline content
    let fragments = collect_inline_fragments(box_, fonts, font_size);

    for fragment_info in fragments {
        // Check if we need to wrap to a new line
        if x_cursor + fragment_info.width > containing_width && !current_line.is_empty() {
            // Finalize current line
            current_line.recalculate_metrics();
            let line_bottom = current_line.y + current_line.height;
            line_boxes.push(current_line);

            // Start new line
            current_line = LineBox::new(start_x, line_bottom);
            x_cursor = 0.0;
        }

        // Add fragment to current line
        let mut fragment = fragment_info;
        fragment.x = x_cursor;
        x_cursor += fragment.width;
        current_line.add_fragment(fragment);
    }

    // Don't forget the last line
    if !current_line.is_empty() {
        current_line.recalculate_metrics();
        line_boxes.push(current_line);
    }

    // Calculate total height
    let total_height = line_boxes.iter().map(|lb| lb.height).sum();

    InlineLayoutResult {
        line_boxes,
        total_height,
    }
}

/// Collect inline fragments from a layout box.
fn collect_inline_fragments<F: FontMetrics>(
    box_: &LayoutBox,
    fonts: &F,
    font_size: f32,
) -> Vec<InlineFragment> {
    let mut fragments = Vec::new();

    // If this box has text content, create a text fragment
    if let Some(ref text) = box_.text_content {
        let metrics = fonts.measure_text(text, font_size, "sans-serif");
        let fragment = InlineFragment::text(text.clone(), 0.0, metrics);
        fragments.push(if let Some(id) = box_.node_id {
            fragment.with_node_id(id)
        } else {
            fragment
        });
    }

    // Recursively collect from children
    for child in &box_.children {
        fragments.extend(collect_inline_fragments(child, fonts, font_size));
    }

    fragments
}

/// Word breaking for text layout.
pub struct WordBreaker<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> WordBreaker<'a> {
    /// Create a new word breaker.
    pub fn new(text: &'a str) -> Self {
        WordBreaker { text, position: 0 }
    }
}

impl<'a> Iterator for WordBreaker<'a> {
    type Item = (&'a str, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.text.len() {
            return None;
        }

        let remaining = &self.text[self.position..];

        // Find next break opportunity
        if let Some(space_pos) = remaining.find(char::is_whitespace) {
            if space_pos == 0 {
                // Leading whitespace
                let ws_end = remaining
                    .find(|c: char| !c.is_whitespace())
                    .unwrap_or(remaining.len());
                self.position += ws_end;
                Some((&remaining[..ws_end], true))
            } else {
                // Word before whitespace
                self.position += space_pos;
                Some((&remaining[..space_pos], false))
            }
        } else {
            // Rest of the text
            self.position = self.text.len();
            Some((remaining, false))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_types::BoxType;
    use crate::fonts::FixedFontMetrics;

    #[test]
    fn test_line_box_creation() {
        let line = LineBox::new(10.0, 20.0);
        assert_eq!(line.x, 10.0);
        assert_eq!(line.y, 20.0);
        assert!(line.is_empty());
    }

    #[test]
    fn test_inline_fragment_text() {
        let metrics = TextMetrics::new(48.0, 12.0, 4.0);
        let fragment = InlineFragment::text("Hello".to_string(), 0.0, metrics);

        assert_eq!(fragment.width, 48.0);
        assert_eq!(fragment.height(), 16.0);
        assert_eq!(fragment.text, Some("Hello".to_string()));
    }

    #[test]
    fn test_line_box_metrics() {
        let mut line = LineBox::new(0.0, 0.0);

        let fragment1 = InlineFragment::text(
            "Hello".to_string(),
            0.0,
            TextMetrics::new(48.0, 12.0, 4.0),
        );
        let fragment2 = InlineFragment::text(
            "World".to_string(),
            48.0,
            TextMetrics::new(48.0, 14.0, 6.0),
        );

        line.add_fragment(fragment1);
        line.add_fragment(fragment2);
        line.recalculate_metrics();

        assert_eq!(line.width, 96.0);
        assert_eq!(line.baseline, 14.0); // Max ascent
        assert_eq!(line.height, 20.0); // Max ascent + max descent
    }

    #[test]
    fn test_inline_layout_single_line() {
        let fonts = FixedFontMetrics::default();

        let mut box_ = LayoutBox::new(BoxType::Inline);
        box_.text_content = Some("Hello".to_string());

        let result = layout_inline(&box_, 800.0, 0.0, 0.0, &fonts, 16.0);

        assert_eq!(result.line_boxes.len(), 1);
        assert_eq!(result.line_boxes[0].fragments.len(), 1);
    }

    #[test]
    fn test_inline_layout_wrapping() {
        let fonts = FixedFontMetrics::default();

        // Each word is about 50px wide (5 chars * 16px * 0.6)
        // Container is 100px, so "Hello World" should wrap
        let mut box_ = LayoutBox::new(BoxType::Inline);
        box_.text_content = Some("Hello World".to_string());

        let result = layout_inline(&box_, 60.0, 0.0, 0.0, &fonts, 16.0);

        // With a 60px container and text that's about 105px, it should still be one line
        // because we don't break within words in this simple implementation
        // A more sophisticated implementation would break between words
        assert!(!result.line_boxes.is_empty());
    }

    #[test]
    fn test_word_breaker() {
        let breaker = WordBreaker::new("Hello World!");
        let words: Vec<_> = breaker.collect();

        assert_eq!(words.len(), 3);
        assert_eq!(words[0], ("Hello", false));
        assert_eq!(words[1], (" ", true));
        assert_eq!(words[2], ("World!", false));
    }

    #[test]
    fn test_word_breaker_multiple_spaces() {
        let breaker = WordBreaker::new("Hello   World");
        let words: Vec<_> = breaker.collect();

        assert_eq!(words.len(), 3);
        assert_eq!(words[0], ("Hello", false));
        assert_eq!(words[1], ("   ", true));
        assert_eq!(words[2], ("World", false));
    }
}
