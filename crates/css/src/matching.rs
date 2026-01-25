//! CSS selector matching engine
//!
//! This module implements bottom-up selector matching, which is the standard
//! approach used by browser CSS engines. We start from the element being tested
//! and work our way up/back through the selector components.

use crate::selector::{
    AttributeMatcher, AttributeSelector, Combinator, CompoundSelector, PseudoClass, Selector,
    SelectorComponent, TypeSelector,
};
use vw_dom::{Document, NodeData, NodeId};

/// Context for pseudo-class matching.
///
/// Some pseudo-classes (like :hover, :focus) depend on runtime state that
/// the DOM alone cannot provide. This trait allows the caller to supply
/// that information.
pub trait PseudoClassContext {
    /// Returns true if the element is currently hovered.
    fn is_hovered(&self, _element: NodeId) -> bool {
        false
    }

    /// Returns true if the element is currently active (being clicked).
    fn is_active(&self, _element: NodeId) -> bool {
        false
    }

    /// Returns true if the element has focus.
    fn is_focused(&self, _element: NodeId) -> bool {
        false
    }

    /// Returns true if the element is a visited link.
    fn is_visited(&self, _element: NodeId) -> bool {
        false
    }

    /// Returns true if the element is an unvisited link.
    fn is_link(&self, element: NodeId) -> bool;

    /// Returns true if the element is enabled (for form elements).
    fn is_enabled(&self, _element: NodeId) -> bool {
        true
    }

    /// Returns true if the element is disabled (for form elements).
    fn is_disabled(&self, _element: NodeId) -> bool {
        false
    }

    /// Returns true if the element is checked (for checkboxes/radios).
    fn is_checked(&self, _element: NodeId) -> bool {
        false
    }
}

/// Default pseudo-class context with no runtime state.
pub struct NoPseudoClassContext;

impl PseudoClassContext for NoPseudoClassContext {
    fn is_link(&self, _element: NodeId) -> bool {
        false
    }
}

/// Checks if a selector matches an element in the document.
///
/// This uses bottom-up matching: we start with the rightmost compound
/// selector and work our way left, matching combinators against the
/// DOM tree structure.
pub fn matches(selector: &Selector, element: NodeId, document: &Document) -> bool {
    matches_with_context(selector, element, document, &NoPseudoClassContext)
}

/// Checks if a selector matches an element, with a custom pseudo-class context.
pub fn matches_with_context<C: PseudoClassContext>(
    selector: &Selector,
    element: NodeId,
    document: &Document,
    context: &C,
) -> bool {
    // Empty selector never matches
    if selector.components.is_empty() {
        return false;
    }

    // Start from the rightmost component and work backwards
    let components = &selector.components;
    matches_from_index(components, components.len() - 1, element, document, context)
}

/// Recursively matches selector components starting from a given index.
fn matches_from_index<C: PseudoClassContext>(
    components: &[SelectorComponent],
    index: usize,
    element: NodeId,
    document: &Document,
    context: &C,
) -> bool {
    match &components[index] {
        SelectorComponent::Compound(compound) => {
            // First, check if this compound selector matches the element
            if !matches_compound(compound, element, document, context) {
                return false;
            }

            // If this is the leftmost component, we're done
            if index == 0 {
                return true;
            }

            // Otherwise, get the combinator and continue matching
            // The combinator should be at index - 1
            if index < 2 {
                // There's something before us but not a combinator - malformed
                return false;
            }

            match &components[index - 1] {
                SelectorComponent::Combinator(combinator) => {
                    // Match the combinator against the DOM structure
                    match_combinator(
                        *combinator,
                        components,
                        index - 2,
                        element,
                        document,
                        context,
                    )
                }
                SelectorComponent::Compound(_) => {
                    // Two compounds in a row - malformed selector
                    false
                }
            }
        }
        SelectorComponent::Combinator(_) => {
            // Selector ends with a combinator - malformed
            false
        }
    }
}

/// Matches a combinator by finding an appropriate related element.
fn match_combinator<C: PseudoClassContext>(
    combinator: Combinator,
    components: &[SelectorComponent],
    target_index: usize,
    element: NodeId,
    document: &Document,
    context: &C,
) -> bool {
    match combinator {
        Combinator::Descendant => {
            // Walk up the ancestor chain looking for a match
            for ancestor in document.ancestors(element) {
                // Only try element nodes
                if let Some(node) = document.get(ancestor) {
                    if node.is_element() {
                        if matches_from_index(components, target_index, ancestor, document, context)
                        {
                            return true;
                        }
                    }
                }
            }
            false
        }
        Combinator::Child => {
            // Check only the direct parent
            if let Some(node) = document.get(element) {
                if let Some(parent) = node.parent {
                    if let Some(parent_node) = document.get(parent) {
                        if parent_node.is_element() {
                            return matches_from_index(
                                components,
                                target_index,
                                parent,
                                document,
                                context,
                            );
                        }
                    }
                }
            }
            false
        }
        Combinator::AdjacentSibling => {
            // Check the immediately preceding sibling
            if let Some(node) = document.get(element) {
                if let Some(prev) = node.prev_sibling {
                    // Skip non-element nodes
                    let mut current = Some(prev);
                    while let Some(id) = current {
                        if let Some(sib_node) = document.get(id) {
                            if sib_node.is_element() {
                                return matches_from_index(
                                    components,
                                    target_index,
                                    id,
                                    document,
                                    context,
                                );
                            }
                            current = sib_node.prev_sibling;
                        } else {
                            break;
                        }
                    }
                }
            }
            false
        }
        Combinator::GeneralSibling => {
            // Check all preceding siblings
            if let Some(node) = document.get(element) {
                let mut current = node.prev_sibling;
                while let Some(sib_id) = current {
                    if let Some(sib_node) = document.get(sib_id) {
                        if sib_node.is_element() {
                            if matches_from_index(
                                components,
                                target_index,
                                sib_id,
                                document,
                                context,
                            ) {
                                return true;
                            }
                        }
                        current = sib_node.prev_sibling;
                    } else {
                        break;
                    }
                }
            }
            false
        }
    }
}

/// Checks if a compound selector matches an element.
pub fn matches_compound<C: PseudoClassContext>(
    compound: &CompoundSelector,
    element: NodeId,
    document: &Document,
    context: &C,
) -> bool {
    let node = match document.get(element) {
        Some(n) => n,
        None => return false,
    };

    let elem_data = match node.as_element() {
        Some(e) => e,
        None => return false, // Can only match element nodes
    };

    // Check type selector
    if let Some(type_sel) = &compound.type_selector {
        match type_sel {
            TypeSelector::Universal => {
                // Universal always matches
            }
            TypeSelector::Tag(tag) => {
                // Case-insensitive comparison for HTML
                if !elem_data.tag_name.eq_ignore_ascii_case(tag) {
                    return false;
                }
            }
        }
    }

    // Check ID selector
    if let Some(id) = &compound.id {
        match elem_data.get_attribute("id") {
            Some(elem_id) if elem_id == id => {}
            _ => return false,
        }
    }

    // Check class selectors
    for class in &compound.classes {
        if !has_class(elem_data.get_attribute("class"), class) {
            return false;
        }
    }

    // Check attribute selectors
    for attr_sel in &compound.attributes {
        if !matches_attribute(attr_sel, elem_data) {
            return false;
        }
    }

    // Check pseudo-classes
    for pseudo in &compound.pseudo_classes {
        if !matches_pseudo_class(pseudo, element, document, context) {
            return false;
        }
    }

    true
}

/// Checks if an element has a specific class in its class attribute.
fn has_class(class_attr: Option<&str>, class_name: &str) -> bool {
    match class_attr {
        Some(classes) => classes.split_whitespace().any(|c| c == class_name),
        None => false,
    }
}

/// Checks if an attribute selector matches an element's attributes.
fn matches_attribute(selector: &AttributeSelector, elem_data: &vw_dom::ElementData) -> bool {
    let value = elem_data.get_attribute(&selector.name);

    match (&selector.matcher, value) {
        // Presence selector: just needs the attribute to exist
        (None, Some(_)) => true,
        (None, None) => false,

        // Value matchers need the attribute to exist
        (Some(_), None) => false,

        (Some(matcher), Some(value)) => match matcher {
            AttributeMatcher::Exact(expected) => value == expected,
            AttributeMatcher::Word(expected) => {
                value.split_whitespace().any(|word| word == expected)
            }
            AttributeMatcher::Prefix(expected) => value.starts_with(expected),
            AttributeMatcher::Suffix(expected) => value.ends_with(expected),
            AttributeMatcher::Substring(expected) => value.contains(expected),
            AttributeMatcher::HyphenPrefix(expected) => {
                value == expected || value.starts_with(&format!("{}-", expected))
            }
        },
    }
}

/// Checks if a pseudo-class matches an element.
fn matches_pseudo_class<C: PseudoClassContext>(
    pseudo: &PseudoClass,
    element: NodeId,
    document: &Document,
    context: &C,
) -> bool {
    match pseudo {
        // Dynamic pseudo-classes - delegate to context
        PseudoClass::Hover => context.is_hovered(element),
        PseudoClass::Active => context.is_active(element),
        PseudoClass::Focus => context.is_focused(element),
        PseudoClass::Visited => context.is_visited(element),
        PseudoClass::Link => context.is_link(element),
        PseudoClass::Enabled => context.is_enabled(element),
        PseudoClass::Disabled => context.is_disabled(element),
        PseudoClass::Checked => context.is_checked(element),

        // Structural pseudo-classes - computed from DOM
        PseudoClass::Root => is_root(element, document),
        PseudoClass::Empty => is_empty(element, document),
        PseudoClass::FirstChild => is_first_child(element, document),
        PseudoClass::LastChild => is_last_child(element, document),
        PseudoClass::OnlyChild => {
            is_first_child(element, document) && is_last_child(element, document)
        }
        PseudoClass::FirstOfType => is_first_of_type(element, document),
        PseudoClass::LastOfType => is_last_of_type(element, document),
        PseudoClass::OnlyOfType => {
            is_first_of_type(element, document) && is_last_of_type(element, document)
        }

        // Nth pseudo-classes
        PseudoClass::NthChild(formula) => {
            let pos = child_index(element, document);
            formula.matches(pos)
        }
        PseudoClass::NthLastChild(formula) => {
            let pos = child_index_from_end(element, document);
            formula.matches(pos)
        }
        PseudoClass::NthOfType(formula) => {
            let pos = type_index(element, document);
            formula.matches(pos)
        }
        PseudoClass::NthLastOfType(formula) => {
            let pos = type_index_from_end(element, document);
            formula.matches(pos)
        }

        // Negation pseudo-class
        PseudoClass::Not(inner) => !matches_compound(inner, element, document, context),
    }
}

/// Returns true if the element is the root element (usually <html>).
fn is_root(element: NodeId, document: &Document) -> bool {
    if let Some(node) = document.get(element) {
        if let Some(parent) = node.parent {
            if let Some(parent_node) = document.get(parent) {
                return parent_node.is_document();
            }
        }
    }
    false
}

/// Returns true if the element has no children (element or text nodes).
fn is_empty(element: NodeId, document: &Document) -> bool {
    for child_id in document.children(element) {
        if let Some(child) = document.get(child_id) {
            match &child.data {
                NodeData::Element(_) => return false,
                NodeData::Text(text) => {
                    // Non-whitespace text means not empty
                    if !text.trim().is_empty() {
                        return false;
                    }
                }
                _ => {}
            }
        }
    }
    true
}

/// Returns true if the element is the first element child of its parent.
fn is_first_child(element: NodeId, document: &Document) -> bool {
    child_index(element, document) == 1
}

/// Returns true if the element is the last element child of its parent.
fn is_last_child(element: NodeId, document: &Document) -> bool {
    child_index_from_end(element, document) == 1
}

/// Returns the 1-indexed position among element siblings.
fn child_index(element: NodeId, document: &Document) -> i32 {
    let node = match document.get(element) {
        Some(n) => n,
        None => return 0,
    };

    let parent = match node.parent {
        Some(p) => p,
        None => return 0,
    };

    let mut index = 0;
    for child_id in document.children(parent) {
        if let Some(child) = document.get(child_id) {
            if child.is_element() {
                index += 1;
                if child_id == element {
                    return index;
                }
            }
        }
    }
    0
}

/// Returns the 1-indexed position from the end among element siblings.
fn child_index_from_end(element: NodeId, document: &Document) -> i32 {
    let node = match document.get(element) {
        Some(n) => n,
        None => return 0,
    };

    let parent = match node.parent {
        Some(p) => p,
        None => return 0,
    };

    // Count total element children and find our position
    let mut total = 0;
    let mut our_index = 0;
    for child_id in document.children(parent) {
        if let Some(child) = document.get(child_id) {
            if child.is_element() {
                total += 1;
                if child_id == element {
                    our_index = total;
                }
            }
        }
    }

    if our_index == 0 {
        0
    } else {
        total - our_index + 1
    }
}

/// Returns true if the element is the first of its type among siblings.
fn is_first_of_type(element: NodeId, document: &Document) -> bool {
    type_index(element, document) == 1
}

/// Returns true if the element is the last of its type among siblings.
fn is_last_of_type(element: NodeId, document: &Document) -> bool {
    type_index_from_end(element, document) == 1
}

/// Returns the 1-indexed position among siblings of the same type.
fn type_index(element: NodeId, document: &Document) -> i32 {
    let tag_name = match document.get(element).and_then(|n| n.as_element()) {
        Some(e) => &e.tag_name,
        None => return 0,
    };

    let parent = match document.get(element).and_then(|n| n.parent) {
        Some(p) => p,
        None => return 0,
    };

    let mut index = 0;
    for child_id in document.children(parent) {
        if let Some(child) = document.get(child_id) {
            if let Some(child_elem) = child.as_element() {
                if child_elem.tag_name.eq_ignore_ascii_case(tag_name) {
                    index += 1;
                    if child_id == element {
                        return index;
                    }
                }
            }
        }
    }
    0
}

/// Returns the 1-indexed position from the end among siblings of the same type.
fn type_index_from_end(element: NodeId, document: &Document) -> i32 {
    let tag_name = match document.get(element).and_then(|n| n.as_element()) {
        Some(e) => &e.tag_name,
        None => return 0,
    };

    let parent = match document.get(element).and_then(|n| n.parent) {
        Some(p) => p,
        None => return 0,
    };

    let mut total = 0;
    let mut our_index = 0;
    for child_id in document.children(parent) {
        if let Some(child) = document.get(child_id) {
            if let Some(child_elem) = child.as_element() {
                if child_elem.tag_name.eq_ignore_ascii_case(tag_name) {
                    total += 1;
                    if child_id == element {
                        our_index = total;
                    }
                }
            }
        }
    }

    if our_index == 0 {
        0
    } else {
        total - our_index + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selector::{NthFormula, Selector, SelectorComponent};

    fn build_test_doc() -> (Document, NodeId, NodeId, NodeId, NodeId) {
        // Structure:
        // document
        //   html
        //     head
        //     body.container#main
        //       div.item
        //       div.item.active
        //       p
        let mut doc = Document::new();
        let html = doc.create_element("html");
        let head = doc.create_element("head");
        let body = doc.create_element_with_attributes(
            "body",
            vec![
                ("class".to_string(), "container".to_string()),
                ("id".to_string(), "main".to_string()),
            ],
        );
        let div1 = doc
            .create_element_with_attributes("div", vec![("class".to_string(), "item".to_string())]);
        let div2 = doc.create_element_with_attributes(
            "div",
            vec![("class".to_string(), "item active".to_string())],
        );
        let p = doc.create_element("p");

        doc.append_child(doc.root(), html);
        doc.append_child(html, head);
        doc.append_child(html, body);
        doc.append_child(body, div1);
        doc.append_child(body, div2);
        doc.append_child(body, p);

        (doc, html, body, div1, div2)
    }

    #[test]
    fn test_type_selector() {
        let (doc, _html, body, div1, _div2) = build_test_doc();

        // div matches div1
        let selector = Selector::simple(CompoundSelector::with_type("div"));
        assert!(matches(&selector, div1, &doc));

        // div doesn't match body
        assert!(!matches(&selector, body, &doc));

        // body matches body
        let selector = Selector::simple(CompoundSelector::with_type("body"));
        assert!(matches(&selector, body, &doc));
    }

    #[test]
    fn test_class_selector() {
        let (doc, _html, body, div1, div2) = build_test_doc();

        // .container matches body
        let selector = Selector::simple(CompoundSelector::with_class("container"));
        assert!(matches(&selector, body, &doc));

        // .item matches div1 and div2
        let selector = Selector::simple(CompoundSelector::with_class("item"));
        assert!(matches(&selector, div1, &doc));
        assert!(matches(&selector, div2, &doc));

        // .active matches only div2
        let selector = Selector::simple(CompoundSelector::with_class("active"));
        assert!(!matches(&selector, div1, &doc));
        assert!(matches(&selector, div2, &doc));
    }

    #[test]
    fn test_id_selector() {
        let (doc, _html, body, div1, _div2) = build_test_doc();

        // #main matches body
        let selector = Selector::simple(CompoundSelector::with_id("main"));
        assert!(matches(&selector, body, &doc));
        assert!(!matches(&selector, div1, &doc));
    }

    #[test]
    fn test_universal_selector() {
        let (doc, html, body, div1, _div2) = build_test_doc();

        // * matches any element
        let selector = Selector::simple(CompoundSelector::universal());
        assert!(matches(&selector, html, &doc));
        assert!(matches(&selector, body, &doc));
        assert!(matches(&selector, div1, &doc));
    }

    #[test]
    fn test_compound_selector() {
        let (doc, _html, body, div1, div2) = build_test_doc();

        // div.item matches div1 and div2
        let compound = CompoundSelector {
            type_selector: Some(TypeSelector::Tag("div".to_string())),
            id: None,
            classes: vec!["item".to_string()],
            attributes: vec![],
            pseudo_classes: vec![],
        };
        let selector = Selector::simple(compound);
        assert!(matches(&selector, div1, &doc));
        assert!(matches(&selector, div2, &doc));
        assert!(!matches(&selector, body, &doc));

        // div.item.active matches only div2
        let compound = CompoundSelector {
            type_selector: Some(TypeSelector::Tag("div".to_string())),
            id: None,
            classes: vec!["item".to_string(), "active".to_string()],
            attributes: vec![],
            pseudo_classes: vec![],
        };
        let selector = Selector::simple(compound);
        assert!(!matches(&selector, div1, &doc));
        assert!(matches(&selector, div2, &doc));
    }

    #[test]
    fn test_descendant_combinator() {
        let (doc, _html, body, div1, _div2) = build_test_doc();

        // body div matches div1 (div is descendant of body)
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector::with_type("body")),
            SelectorComponent::Combinator(Combinator::Descendant),
            SelectorComponent::Compound(CompoundSelector::with_type("div")),
        ]);
        assert!(matches(&selector, div1, &doc));

        // html div matches div1 (div is descendant of html)
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector::with_type("html")),
            SelectorComponent::Combinator(Combinator::Descendant),
            SelectorComponent::Compound(CompoundSelector::with_type("div")),
        ]);
        assert!(matches(&selector, div1, &doc));

        // div body doesn't match body (body is not descendant of div)
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector::with_type("div")),
            SelectorComponent::Combinator(Combinator::Descendant),
            SelectorComponent::Compound(CompoundSelector::with_type("body")),
        ]);
        assert!(!matches(&selector, body, &doc));
    }

    #[test]
    fn test_child_combinator() {
        let (doc, _html, _body, div1, _div2) = build_test_doc();

        // body > div matches div1 (div is direct child of body)
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector::with_type("body")),
            SelectorComponent::Combinator(Combinator::Child),
            SelectorComponent::Compound(CompoundSelector::with_type("div")),
        ]);
        assert!(matches(&selector, div1, &doc));

        // html > div doesn't match div1 (div is not direct child of html)
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector::with_type("html")),
            SelectorComponent::Combinator(Combinator::Child),
            SelectorComponent::Compound(CompoundSelector::with_type("div")),
        ]);
        assert!(!matches(&selector, div1, &doc));
    }

    #[test]
    fn test_adjacent_sibling_combinator() {
        let (doc, _html, _body, div1, div2) = build_test_doc();

        // div + div matches div2 (preceded by div1)
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector::with_type("div")),
            SelectorComponent::Combinator(Combinator::AdjacentSibling),
            SelectorComponent::Compound(CompoundSelector::with_type("div")),
        ]);
        assert!(!matches(&selector, div1, &doc)); // div1 has no preceding div
        assert!(matches(&selector, div2, &doc)); // div2 is preceded by div1
    }

    #[test]
    fn test_general_sibling_combinator() {
        let (doc, _html, body, div1, _div2) = build_test_doc();

        // Get the p element
        let p = doc.children(body).find(|&id| {
            doc.get(id)
                .and_then(|n| n.as_element())
                .map(|e| e.tag_name == "p")
                .unwrap_or(false)
        });
        let p = p.unwrap();

        // div ~ p matches p (preceded by divs)
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector::with_type("div")),
            SelectorComponent::Combinator(Combinator::GeneralSibling),
            SelectorComponent::Compound(CompoundSelector::with_type("p")),
        ]);
        assert!(matches(&selector, p, &doc));
        assert!(!matches(&selector, div1, &doc));
    }

    #[test]
    fn test_first_child() {
        let (doc, _html, _body, div1, div2) = build_test_doc();

        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::FirstChild],
        };
        let selector = Selector::simple(compound);

        assert!(matches(&selector, div1, &doc)); // div1 is first child of body
        assert!(!matches(&selector, div2, &doc)); // div2 is second child
    }

    #[test]
    fn test_last_child() {
        let (doc, _html, body, div1, _div2) = build_test_doc();

        // Get the p element (last child)
        let p = doc.children(body).last().unwrap();

        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::LastChild],
        };
        let selector = Selector::simple(compound);

        assert!(!matches(&selector, div1, &doc));
        assert!(matches(&selector, p, &doc)); // p is last child
    }

    #[test]
    fn test_nth_child() {
        let (doc, _html, body, div1, div2) = build_test_doc();
        let p = doc.children(body).last().unwrap();

        // :nth-child(1) matches first child
        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::NthChild(NthFormula::new(0, 1))],
        };
        let selector = Selector::simple(compound);
        assert!(matches(&selector, div1, &doc));
        assert!(!matches(&selector, div2, &doc));

        // :nth-child(odd) matches 1st and 3rd
        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::NthChild(NthFormula::odd())],
        };
        let selector = Selector::simple(compound);
        assert!(matches(&selector, div1, &doc)); // 1st
        assert!(!matches(&selector, div2, &doc)); // 2nd
        assert!(matches(&selector, p, &doc)); // 3rd
    }

    #[test]
    fn test_root() {
        let (doc, html, body, _div1, _div2) = build_test_doc();

        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::Root],
        };
        let selector = Selector::simple(compound);

        assert!(matches(&selector, html, &doc)); // html is root
        assert!(!matches(&selector, body, &doc)); // body is not root
    }

    #[test]
    fn test_empty() {
        let mut doc = Document::new();
        let empty_div = doc.create_element("div");
        let non_empty_div = doc.create_element("div");
        let text = doc.create_text("Hello");

        doc.append_child(doc.root(), empty_div);
        doc.append_child(doc.root(), non_empty_div);
        doc.append_child(non_empty_div, text);

        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::Empty],
        };
        let selector = Selector::simple(compound);

        assert!(matches(&selector, empty_div, &doc));
        assert!(!matches(&selector, non_empty_div, &doc));
    }

    #[test]
    fn test_not() {
        let (doc, _html, body, div1, _div2) = build_test_doc();

        // :not(.container) matches divs but not body
        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::Not(Box::new(CompoundSelector::with_class(
                "container",
            )))],
        };
        let selector = Selector::simple(compound);

        assert!(matches(&selector, div1, &doc));
        assert!(!matches(&selector, body, &doc));
    }

    #[test]
    fn test_attribute_selector() {
        let mut doc = Document::new();
        let input = doc.create_element_with_attributes(
            "input",
            vec![("type".to_string(), "text".to_string())],
        );
        let checkbox = doc.create_element_with_attributes(
            "input",
            vec![("type".to_string(), "checkbox".to_string())],
        );
        doc.append_child(doc.root(), input);
        doc.append_child(doc.root(), checkbox);

        // [type="text"] matches input
        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![AttributeSelector::exact("type", "text")],
            pseudo_classes: vec![],
        };
        let selector = Selector::simple(compound);

        assert!(matches(&selector, input, &doc));
        assert!(!matches(&selector, checkbox, &doc));

        // [type] matches both (presence)
        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![AttributeSelector::presence("type")],
            pseudo_classes: vec![],
        };
        let selector = Selector::simple(compound);

        assert!(matches(&selector, input, &doc));
        assert!(matches(&selector, checkbox, &doc));
    }

    #[test]
    fn test_first_of_type() {
        let (doc, _html, body, div1, div2) = build_test_doc();
        let p = doc.children(body).last().unwrap();

        // :first-of-type
        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::FirstOfType],
        };
        let selector = Selector::simple(compound);

        assert!(matches(&selector, div1, &doc)); // first div
        assert!(!matches(&selector, div2, &doc)); // second div
        assert!(matches(&selector, p, &doc)); // first (and only) p
    }

    #[test]
    fn test_complex_selector() {
        let (doc, _html, _body, _div1, div2) = build_test_doc();

        // body#main > div.item.active
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector {
                type_selector: Some(TypeSelector::Tag("body".to_string())),
                id: Some("main".to_string()),
                classes: vec![],
                attributes: vec![],
                pseudo_classes: vec![],
            }),
            SelectorComponent::Combinator(Combinator::Child),
            SelectorComponent::Compound(CompoundSelector {
                type_selector: Some(TypeSelector::Tag("div".to_string())),
                id: None,
                classes: vec!["item".to_string(), "active".to_string()],
                attributes: vec![],
                pseudo_classes: vec![],
            }),
        ]);

        assert!(matches(&selector, div2, &doc));
    }
}
