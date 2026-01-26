//! Parsed and styled document for rendering.
//!
//! This module provides the `Document` type which holds the result of
//! parsing HTML and CSS into a DOM tree with computed styles.

use crate::error::Result;
use crate::render_tree::RenderTree;
use crate::viewport::Viewport;
use vw_css::Stylesheet;
use vw_dom::Document as DomDocument;
use vw_dom::NodeId as DomNodeId;
use vw_layout::dom::{Document as LayoutDocument, NodeData as LayoutNodeData};
use vw_layout::style::{ComputedStyle as LayoutComputedStyle, Display, Length, StyleTree};
use vw_layout::NodeId as LayoutNodeId;
use vw_style::tree::NodeInfo;
use vw_style::ComputedStyle as VwComputedStyle;
use vw_style::Length as VwLength;

/// A parsed and styled document ready for rendering.
///
/// The Document holds the parsed DOM tree and computed styles from CSS.
/// It can generate a `RenderTree` for a given viewport, which can then
/// be painted to a framebuffer.
///
/// # Example
///
/// ```
/// use vw_renderer::Document;
///
/// let doc = Document::load("<html><body><p>Hello</p></body></html>", "")
///     .expect("Parse error");
/// ```
#[derive(Debug)]
pub struct Document {
    /// The parsed DOM tree
    dom: DomDocument,
    /// The parsed stylesheet (may be empty)
    stylesheet: Stylesheet,
}

impl Document {
    /// Parse HTML and CSS into a Document.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML source to parse
    /// * `css` - The CSS source to parse (can be empty)
    ///
    /// # Returns
    ///
    /// A Document ready for rendering, or an error if parsing fails.
    pub fn load(html: &str, css: &str) -> Result<Self> {
        // Parse HTML into DOM
        let dom = vw_html::parse(html);

        // Parse CSS into stylesheet
        let stylesheet = vw_css::parse(css);

        Ok(Document { dom, stylesheet })
    }

    /// Generate a render tree for the given viewport.
    ///
    /// This performs layout and prepares the document for painting.
    /// The render tree contains positioned boxes ready to be painted.
    ///
    /// # Arguments
    ///
    /// * `viewport` - The viewport dimensions for layout
    ///
    /// # Returns
    ///
    /// A RenderTree ready for painting.
    pub fn render_tree(&self, viewport: &Viewport) -> RenderTree {
        // Convert DOM to layout-compatible format
        let (layout_doc, node_mapping) = self.build_layout_document();

        // Compute styles
        let style_tree = self.compute_styles(&layout_doc, &node_mapping);

        // Perform layout
        let layout_viewport = viewport.to_layout_viewport();
        let layout_root = vw_layout::layout(&layout_doc, &style_tree, &layout_viewport);

        RenderTree::new(layout_root, viewport.width(), viewport.height())
    }

    /// Build a layout-compatible document from the DOM.
    ///
    /// Returns the layout document and a mapping from DOM node IDs to layout node IDs.
    fn build_layout_document(&self) -> (LayoutDocument, Vec<(DomNodeId, LayoutNodeId)>) {
        let mut layout_doc = LayoutDocument::new();
        let mut node_mapping = Vec::new();

        // Get the document element (usually <html>)
        if let Some(doc_element) = self.dom.document_element() {
            self.build_layout_subtree(&mut layout_doc, &mut node_mapping, doc_element, None);
        }

        (layout_doc, node_mapping)
    }

    /// Recursively build the layout document from a DOM subtree.
    fn build_layout_subtree(
        &self,
        layout_doc: &mut LayoutDocument,
        node_mapping: &mut Vec<(DomNodeId, LayoutNodeId)>,
        dom_id: DomNodeId,
        parent_layout_id: Option<LayoutNodeId>,
    ) -> Option<LayoutNodeId> {
        let node = self.dom.get(dom_id)?;

        // Create layout node based on DOM node type
        let layout_node = match &node.data {
            vw_dom::NodeData::Element(elem) => {
                LayoutNodeData::element(LayoutNodeId(0), &elem.tag_name)
            }
            vw_dom::NodeData::Text(text) => {
                // Skip whitespace-only text nodes
                if text.trim().is_empty() {
                    return None;
                }
                LayoutNodeData::text(LayoutNodeId(0), text)
            }
            vw_dom::NodeData::Document => LayoutNodeData::document(LayoutNodeId(0)),
            _ => return None, // Skip comments, doctype, etc.
        };

        // Add node to layout document
        let layout_id = layout_doc.add_node(layout_node);
        node_mapping.push((dom_id, layout_id));

        // Set as root if no parent
        if parent_layout_id.is_none() {
            layout_doc.set_root(layout_id);
        }

        // Link to parent
        if let Some(parent_id) = parent_layout_id {
            if let Some(parent) = layout_doc.get_node_mut(parent_id) {
                parent.children.push(layout_id);
            }
        }

        // Process children
        let mut child_id = node.first_child;
        while let Some(id) = child_id {
            self.build_layout_subtree(layout_doc, node_mapping, id, Some(layout_id));
            child_id = self.dom.get(id).and_then(|n| n.next_sibling);
        }

        Some(layout_id)
    }

    /// Compute styles for all nodes in the layout document.
    fn compute_styles(
        &self,
        layout_doc: &LayoutDocument,
        node_mapping: &[(DomNodeId, LayoutNodeId)],
    ) -> StyleTree {
        use crate::author_matcher::AuthorStyleMatcher;
        use vw_style::tree::StyleTreeBuilder;

        // Build NodeInfo list for vw-style
        let nodes: Vec<NodeInfo> = layout_doc
            .nodes()
            .enumerate()
            .map(|(idx, node)| {
                let parent = if idx > 0 {
                    // Find parent - search node_mapping or use heuristics
                    Some(0) // Simplification: assume root is parent for now
                } else {
                    None
                };

                if node.is_element() {
                    NodeInfo::element(idx, parent, node.tag_name.as_deref().unwrap_or(""))
                } else {
                    NodeInfo::text(idx, parent)
                }
            })
            .collect();

        // Create node mapping in the format expected by AuthorStyleMatcher
        let author_node_mapping: Vec<(DomNodeId, usize)> = node_mapping
            .iter()
            .map(|(dom_id, layout_id)| (*dom_id, layout_id.0))
            .collect();

        // Build the author style matcher and compute styles
        let author_matcher = AuthorStyleMatcher::new(&self.stylesheet, &self.dom, &author_node_mapping);
        let builder = StyleTreeBuilder::new(&author_matcher);
        let vw_style_tree = builder.build(&nodes);

        // Convert vw-style's StyleTree to vw-layout's StyleTree
        let mut layout_style_tree = StyleTree::new();

        for node in layout_doc.nodes() {
            let idx = node.id.0;
            if let Some(vw_style) = vw_style_tree.get(idx) {
                let layout_style = convert_style(vw_style);
                layout_style_tree.insert(node.id, layout_style);
            } else {
                // Use default style
                layout_style_tree.insert(node.id, LayoutComputedStyle::default());
            }
        }

        layout_style_tree
    }

    /// Get a reference to the underlying DOM document.
    pub fn dom(&self) -> &DomDocument {
        &self.dom
    }

    /// Get a reference to the parsed stylesheet.
    pub fn stylesheet(&self) -> &Stylesheet {
        &self.stylesheet
    }
}

/// Convert a vw-style ComputedStyle to a vw-layout ComputedStyle.
fn convert_style(vw_style: &VwComputedStyle) -> LayoutComputedStyle {
    use vw_layout::style::LineHeight as LayoutLineHeight;

    // Default font context for length resolution
    let font_size = vw_style.font_size;
    let root_font_size = 16.0; // Default root font size

    let display = match vw_style.display {
        vw_style::Display::Block => Display::Block,
        vw_style::Display::Inline => Display::Inline,
        vw_style::Display::InlineBlock => Display::InlineBlock,
        vw_style::Display::None => Display::None,
    };

    let convert_length = |len: &VwLength| -> Length {
        match len {
            VwLength::Px(px) => Length::Px(*px),
            VwLength::Percent(pct) => Length::Percent(*pct),
            VwLength::Em(em) => Length::Px(em * font_size),
            VwLength::Rem(rem) => Length::Px(rem * root_font_size),
            VwLength::Zero => Length::Px(0.0),
        }
    };

    let convert_length_or_auto = |len: &vw_style::LengthOrAuto| -> Length {
        match len {
            vw_style::LengthOrAuto::Auto => Length::Auto,
            vw_style::LengthOrAuto::Length(l) => convert_length(l),
        }
    };

    // Handle margin sides
    let margin_top = convert_length_or_auto(&vw_style.margin.top);
    let margin_right = convert_length_or_auto(&vw_style.margin.right);
    let margin_bottom = convert_length_or_auto(&vw_style.margin.bottom);
    let margin_left = convert_length_or_auto(&vw_style.margin.left);

    // Handle padding sides (resolve to px, default 0)
    let padding_top = length_to_px(&vw_style.padding.top, font_size, root_font_size);
    let padding_right = length_to_px(&vw_style.padding.right, font_size, root_font_size);
    let padding_bottom = length_to_px(&vw_style.padding.bottom, font_size, root_font_size);
    let padding_left = length_to_px(&vw_style.padding.left, font_size, root_font_size);

    // Handle border widths
    let border_top_width = length_to_px(&vw_style.border_width.top, font_size, root_font_size);
    let border_right_width = length_to_px(&vw_style.border_width.right, font_size, root_font_size);
    let border_bottom_width =
        length_to_px(&vw_style.border_width.bottom, font_size, root_font_size);
    let border_left_width = length_to_px(&vw_style.border_width.left, font_size, root_font_size);

    // Handle line height (vw_style::LineHeight::Length already contains px value)
    let line_height = match vw_style.line_height {
        vw_style::LineHeight::Normal => LayoutLineHeight::Normal,
        vw_style::LineHeight::Number(n) => LayoutLineHeight::Number(n),
        vw_style::LineHeight::Length(px) => LayoutLineHeight::Px(px),
    };

    LayoutComputedStyle {
        display,
        width: convert_length_or_auto(&vw_style.width),
        height: convert_length_or_auto(&vw_style.height),
        margin_top,
        margin_right,
        margin_bottom,
        margin_left,
        padding_top,
        padding_right,
        padding_bottom,
        padding_left,
        border_top_width,
        border_right_width,
        border_bottom_width,
        border_left_width,
        font_size: vw_style.font_size,
        line_height,
    }
}

/// Convert a vw-style Length to pixels.
fn length_to_px(len: &VwLength, font_size: f32, root_font_size: f32) -> f32 {
    len.to_px(font_size, root_font_size, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_simple_document() {
        let doc = Document::load("<html><body><p>Hello</p></body></html>", "").unwrap();

        assert!(doc.dom().document_element().is_some());
    }

    #[test]
    fn test_load_empty_document() {
        let doc = Document::load("", "").unwrap();
        // Empty HTML still creates a document
        assert!(doc.dom().node_count() > 0);
    }

    #[test]
    fn test_load_with_css() {
        let html = "<html><body><div class='red'>Test</div></body></html>";
        let css = ".red { color: red; }";

        let doc = Document::load(html, css).unwrap();

        assert!(!doc.stylesheet().rules.is_empty());
    }

    #[test]
    fn test_render_tree_generation() {
        let doc = Document::load("<html><body><p>Hello</p></body></html>", "").unwrap();
        let viewport = Viewport::new(800, 600);

        let render_tree = doc.render_tree(&viewport);

        assert_eq!(render_tree.viewport_width(), 800);
        assert_eq!(render_tree.viewport_height(), 600);
    }
}

    #[test]
    fn test_author_stylesheet_applied() {
        // Create a document with an author stylesheet
        let html = r#"<html><body><div class="red">Hello</div></body></html>"#;
        let css = ".red { color: red; }";

        let doc = Document::load(html, css).unwrap();

        // Verify the stylesheet was parsed
        assert!(!doc.stylesheet().rules.is_empty(), "Stylesheet should have rules");
        assert_eq!(doc.stylesheet().rules.len(), 1, "Should have exactly one rule");

        // Verify the rule selector
        let rule = &doc.stylesheet().rules[0];
        assert_eq!(rule.selectors.len(), 1, "Rule should have one selector");

        // Verify the declaration
        assert_eq!(rule.declarations.len(), 1, "Rule should have one declaration");
        assert_eq!(rule.declarations[0].property, "color", "Declaration should be for color property");
    }
