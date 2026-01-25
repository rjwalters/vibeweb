//! Layout tree construction.
//!
//! This module builds a layout tree from DOM nodes and computed styles.
//! It handles:
//! - Creating layout boxes for each element
//! - Determining box types from display property
//! - Creating anonymous boxes for mixed content
//! - Skipping display:none elements

use crate::box_types::{BoxType, LayoutBox};
use crate::dom::{Document, NodeData, NodeType};
use crate::style::{Display, StyleTree};

/// Build a layout tree from a DOM document and style tree.
///
/// This creates the box tree that will be used for layout. Elements with
/// `display: none` are excluded, and anonymous boxes are created for
/// mixed block/inline content.
pub fn build_layout_tree(document: &Document, styles: &StyleTree) -> Option<LayoutBox> {
    let root_id = document.root()?;
    let root_node = document.get_node(root_id)?;
    build_layout_box(document, styles, root_node)
}

/// Build a layout box for a single node.
fn build_layout_box(
    document: &Document,
    styles: &StyleTree,
    node: &NodeData,
) -> Option<LayoutBox> {
    match node.node_type {
        NodeType::Element => build_element_box(document, styles, node),
        NodeType::Text => build_text_box(node),
        NodeType::Document => build_document_box(document, styles, node),
        NodeType::Comment => None, // Comments don't generate boxes
    }
}

/// Build a layout box for an element node.
fn build_element_box(
    document: &Document,
    styles: &StyleTree,
    node: &NodeData,
) -> Option<LayoutBox> {
    // Get computed style (or default)
    let style = styles.get(node.id).cloned().unwrap_or_default();

    // Skip display: none elements
    if style.display.is_none() {
        return None;
    }

    // Determine box type from display property
    let box_type = match style.display {
        Display::Block => BoxType::Block,
        Display::Inline => BoxType::Inline,
        Display::InlineBlock => BoxType::InlineBlock,
        Display::None => return None,
    };

    let mut layout_box = LayoutBox::new(box_type).with_node_id(node.id);

    // Build child boxes
    let mut child_boxes = Vec::new();
    for child_id in &node.children {
        if let Some(child_node) = document.get_node(*child_id) {
            if let Some(child_box) = build_layout_box(document, styles, child_node) {
                child_boxes.push(child_box);
            }
        }
    }

    // Handle mixed content (create anonymous boxes if needed)
    layout_box.children = wrap_mixed_content(child_boxes, box_type);

    Some(layout_box)
}

/// Build a layout box for a text node.
fn build_text_box(node: &NodeData) -> Option<LayoutBox> {
    let text = node.text_content.as_ref()?;

    // Skip empty or whitespace-only text nodes
    // (A more sophisticated implementation would preserve some whitespace)
    if text.trim().is_empty() {
        return None;
    }

    Some(
        LayoutBox::new(BoxType::Inline)
            .with_node_id(node.id)
            .with_text(text.clone()),
    )
}

/// Build a layout box for the document root.
fn build_document_box(
    document: &Document,
    styles: &StyleTree,
    node: &NodeData,
) -> Option<LayoutBox> {
    let mut layout_box = LayoutBox::block().with_node_id(node.id);

    // Build child boxes (typically the <html> element)
    for child_id in &node.children {
        if let Some(child_node) = document.get_node(*child_id) {
            if let Some(child_box) = build_layout_box(document, styles, child_node) {
                layout_box.add_child(child_box);
            }
        }
    }

    Some(layout_box)
}

/// Wrap mixed inline/block content in anonymous boxes.
///
/// According to CSS, if a block container has both block and inline children,
/// the inline children must be wrapped in anonymous block boxes.
fn wrap_mixed_content(children: Vec<LayoutBox>, parent_type: BoxType) -> Vec<LayoutBox> {
    if children.is_empty() {
        return children;
    }

    // Check if we have mixed content
    let has_block = children.iter().any(|c| c.box_type.is_block_level());
    let has_inline = children.iter().any(|c| c.box_type.is_inline_level());

    if !has_block || !has_inline {
        // No mixing, return as-is
        return children;
    }

    // Only wrap if parent is block-level
    if !parent_type.is_block_level() {
        return children;
    }

    // Wrap consecutive inline boxes in anonymous block boxes
    let mut result = Vec::new();
    let mut inline_buffer: Vec<LayoutBox> = Vec::new();

    for child in children {
        if child.box_type.is_block_level() {
            // Flush inline buffer
            if !inline_buffer.is_empty() {
                let mut anonymous = LayoutBox::anonymous_block();
                anonymous.children = std::mem::take(&mut inline_buffer);
                result.push(anonymous);
            }
            result.push(child);
        } else {
            inline_buffer.push(child);
        }
    }

    // Flush remaining inline content
    if !inline_buffer.is_empty() {
        let mut anonymous = LayoutBox::anonymous_block();
        anonymous.children = inline_buffer;
        result.push(anonymous);
    }

    result
}

/// Check if a box has only inline children.
pub fn has_only_inline_children(layout_box: &LayoutBox) -> bool {
    layout_box
        .children
        .iter()
        .all(|c| c.box_type.is_inline_level())
}

/// Check if a box has any block children.
pub fn has_block_children(layout_box: &LayoutBox) -> bool {
    layout_box
        .children
        .iter()
        .any(|c| c.box_type.is_block_level())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_types::NodeId;
    use crate::dom::NodeData;
    use crate::style::ComputedStyle;

    fn setup_simple_document() -> (Document, StyleTree) {
        let mut document = Document::new();
        let mut styles = StyleTree::new();

        // Create: <div><p>Hello</p></div>
        let doc_id = document.add_node(NodeData::document(NodeId(0)));
        document.set_root(doc_id);

        let div_id = document.add_node(NodeData::element(NodeId(0), "div"));
        let p_id = document.add_node(NodeData::element(NodeId(0), "p"));
        let text_id = document.add_node(NodeData::text(NodeId(0), "Hello"));

        // Link nodes
        document.get_node_mut(doc_id).unwrap().children.push(div_id);
        document.get_node_mut(div_id).unwrap().children.push(p_id);
        document.get_node_mut(p_id).unwrap().children.push(text_id);

        // Set styles
        styles.insert(div_id, ComputedStyle::default());
        styles.insert(
            p_id,
            ComputedStyle {
                display: Display::Block,
                ..Default::default()
            },
        );

        (document, styles)
    }

    #[test]
    fn test_build_layout_tree() {
        let (document, styles) = setup_simple_document();
        let layout_tree = build_layout_tree(&document, &styles);

        assert!(layout_tree.is_some());
        let root = layout_tree.unwrap();
        assert_eq!(root.box_type, BoxType::Block);
        assert_eq!(root.children.len(), 1); // div
    }

    #[test]
    fn test_skip_display_none() {
        let mut document = Document::new();
        let mut styles = StyleTree::new();

        let doc_id = document.add_node(NodeData::document(NodeId(0)));
        document.set_root(doc_id);

        let div_id = document.add_node(NodeData::element(NodeId(0), "div"));
        let hidden_id = document.add_node(NodeData::element(NodeId(0), "span"));

        document.get_node_mut(doc_id).unwrap().children.push(div_id);
        document.get_node_mut(div_id).unwrap().children.push(hidden_id);

        styles.insert(div_id, ComputedStyle::default());
        styles.insert(
            hidden_id,
            ComputedStyle {
                display: Display::None,
                ..Default::default()
            },
        );

        let layout_tree = build_layout_tree(&document, &styles).unwrap();
        let div_box = &layout_tree.children[0];
        assert!(div_box.children.is_empty()); // Hidden element not included
    }

    #[test]
    fn test_text_node_box() {
        let node = NodeData::text(NodeId(42), "Hello, World!");
        let layout_box = build_text_box(&node);

        assert!(layout_box.is_some());
        let box_ = layout_box.unwrap();
        assert_eq!(box_.box_type, BoxType::Inline);
        assert_eq!(box_.text_content, Some("Hello, World!".to_string()));
        assert_eq!(box_.node_id, Some(NodeId(42)));
    }

    #[test]
    fn test_skip_empty_text() {
        let node = NodeData::text(NodeId(1), "   ");
        let layout_box = build_text_box(&node);
        assert!(layout_box.is_none());
    }

    #[test]
    fn test_wrap_mixed_content() {
        let inline1 = LayoutBox::inline();
        let block = LayoutBox::block();
        let inline2 = LayoutBox::inline();

        let children = vec![inline1, block, inline2];
        let wrapped = wrap_mixed_content(children, BoxType::Block);

        // Should have: anonymous(inline1), block, anonymous(inline2)
        assert_eq!(wrapped.len(), 3);
        assert_eq!(wrapped[0].box_type, BoxType::Anonymous);
        assert_eq!(wrapped[0].children.len(), 1);
        assert_eq!(wrapped[1].box_type, BoxType::Block);
        assert_eq!(wrapped[2].box_type, BoxType::Anonymous);
        assert_eq!(wrapped[2].children.len(), 1);
    }

    #[test]
    fn test_no_wrap_when_all_same_type() {
        let inline1 = LayoutBox::inline();
        let inline2 = LayoutBox::inline();

        let children = vec![inline1, inline2];
        let wrapped = wrap_mixed_content(children, BoxType::Block);

        // Should not wrap when all same type
        assert_eq!(wrapped.len(), 2);
        assert_eq!(wrapped[0].box_type, BoxType::Inline);
        assert_eq!(wrapped[1].box_type, BoxType::Inline);
    }
}
