//! Integration tests for the DOM crate

use crate::{Document, NodeData};

#[test]
fn test_build_simple_document() {
    let mut doc = Document::new();

    // Build: <html><head><title>Test</title></head><body><p>Hello</p></body></html>
    let html = doc.create_element("html");
    let head = doc.create_element("head");
    let title = doc.create_element("title");
    let title_text = doc.create_text("Test");
    let body = doc.create_element("body");
    let p = doc.create_element("p");
    let p_text = doc.create_text("Hello");

    doc.append_child(doc.root(), html);
    doc.append_child(html, head);
    doc.append_child(head, title);
    doc.append_child(title, title_text);
    doc.append_child(html, body);
    doc.append_child(body, p);
    doc.append_child(p, p_text);

    // Verify structure
    assert_eq!(doc.document_element(), Some(html));
    assert_eq!(doc.get_element_by_tag_name("title"), Some(title));
    assert_eq!(doc.get_element_by_tag_name("p"), Some(p));
    assert_eq!(doc.text_content(title), "Test");
    assert_eq!(doc.text_content(p), "Hello");
    assert_eq!(doc.text_content(html), "TestHello");
}

#[test]
fn test_element_with_attributes() {
    let mut doc = Document::new();

    let div = doc.create_element_with_attributes(
        "div",
        vec![
            ("class".to_string(), "container".to_string()),
            ("id".to_string(), "main".to_string()),
        ],
    );
    doc.append_child(doc.root(), div);

    let elem = doc.get(div).unwrap().as_element().unwrap();
    assert_eq!(elem.get_attribute("class"), Some("container"));
    assert_eq!(elem.get_attribute("id"), Some("main"));
    assert_eq!(elem.get_attribute("nonexistent"), None);
}

#[test]
fn test_remove_child() {
    let mut doc = Document::new();

    let parent = doc.create_element("div");
    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    let child3 = doc.create_element("a");

    doc.append_child(doc.root(), parent);
    doc.append_child(parent, child1);
    doc.append_child(parent, child2);
    doc.append_child(parent, child3);

    // Remove middle child
    doc.remove_child(child2);

    let children: Vec<_> = doc.children(parent).collect();
    assert_eq!(children, vec![child1, child3]);
    assert_eq!(doc.get(child1).unwrap().next_sibling, Some(child3));
    assert_eq!(doc.get(child3).unwrap().prev_sibling, Some(child1));

    // Remove first child
    doc.remove_child(child1);
    assert_eq!(doc.get(parent).unwrap().first_child, Some(child3));

    // Remove last child
    doc.remove_child(child3);
    assert_eq!(doc.get(parent).unwrap().first_child, None);
    assert_eq!(doc.get(parent).unwrap().last_child, None);
}

#[test]
fn test_insert_before() {
    let mut doc = Document::new();

    let parent = doc.create_element("div");
    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    let new_child = doc.create_element("a");

    doc.append_child(doc.root(), parent);
    doc.append_child(parent, child1);
    doc.append_child(parent, child2);

    // Insert before child2
    doc.insert_before(parent, new_child, child2);

    let children: Vec<_> = doc.children(parent).collect();
    assert_eq!(children, vec![child1, new_child, child2]);

    // Insert at beginning
    let first = doc.create_element("first");
    doc.insert_before(parent, first, child1);

    let children: Vec<_> = doc.children(parent).collect();
    assert_eq!(children, vec![first, child1, new_child, child2]);
    assert_eq!(doc.get(parent).unwrap().first_child, Some(first));
}

#[test]
fn test_doctype_and_comment() {
    let mut doc = Document::new();

    let doctype = doc.create_doctype("html");
    let comment = doc.create_comment("This is a comment");

    doc.append_child(doc.root(), doctype);
    doc.append_child(doc.root(), comment);

    match &doc.get(doctype).unwrap().data {
        NodeData::Doctype { name } => assert_eq!(name, "html"),
        _ => panic!("Expected doctype node"),
    }

    match &doc.get(comment).unwrap().data {
        NodeData::Comment(text) => assert_eq!(text, "This is a comment"),
        _ => panic!("Expected comment node"),
    }
}

#[test]
fn test_case_insensitive_tag_lookup() {
    let mut doc = Document::new();

    let html = doc.create_element("HTML");
    let body = doc.create_element("BODY");
    doc.append_child(doc.root(), html);
    doc.append_child(html, body);

    // Should find regardless of case
    assert!(doc.get_element_by_tag_name("html").is_some());
    assert!(doc.get_element_by_tag_name("HTML").is_some());
    assert!(doc.get_element_by_tag_name("Html").is_some());
    assert!(doc.get_element_by_tag_name("body").is_some());
    assert!(doc.get_element_by_tag_name("BODY").is_some());
}
