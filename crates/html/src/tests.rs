//! Integration tests for the HTML parser

use crate::parse;
use vw_dom::NodeData;

#[test]
fn test_full_html_document() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <meta charset="utf-8">
</head>
<body>
    <h1>Welcome</h1>
    <p>This is a <strong>test</strong> paragraph.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>
</body>
</html>"#;

    let doc = parse(html);

    // Verify basic structure
    let html_elem = doc.document_element().unwrap();
    assert_eq!(
        doc.get(html_elem).unwrap().as_element().unwrap().tag_name,
        "html"
    );

    // Check head
    let head = doc.get_element_by_tag_name("head").unwrap();
    assert!(doc.get(head).is_some());

    // Check title
    let title = doc.get_element_by_tag_name("title").unwrap();
    assert_eq!(doc.text_content(title), "Test Page");

    // Check body content
    let h1 = doc.get_element_by_tag_name("h1").unwrap();
    assert_eq!(doc.text_content(h1), "Welcome");

    // Check strong element inside p
    let strong = doc.get_element_by_tag_name("strong").unwrap();
    assert_eq!(doc.text_content(strong), "test");

    // Check list items
    let ul = doc.get_element_by_tag_name("ul").unwrap();
    let li_count = doc
        .children(ul)
        .filter(|&id| {
            doc.get(id)
                .and_then(|n| n.as_element())
                .map(|e| e.tag_name == "li")
                .unwrap_or(false)
        })
        .count();
    assert_eq!(li_count, 3);
}

#[test]
fn test_malformed_html() {
    // Missing closing tags
    let doc = parse("<div><p>Unclosed paragraph<span>text</div>");

    // Should still parse and create structure
    let div = doc.get_element_by_tag_name("div").unwrap();
    let p = doc.get_element_by_tag_name("p").unwrap();
    let span = doc.get_element_by_tag_name("span").unwrap();

    assert!(doc.get(div).is_some());
    assert!(doc.get(p).is_some());
    assert!(doc.get(span).is_some());
}

#[test]
fn test_multiple_text_nodes() {
    let doc = parse("<p>Hello <em>beautiful</em> world!</p>");

    let p = doc.get_element_by_tag_name("p").unwrap();
    assert_eq!(doc.text_content(p), "Hello beautiful world!");

    let em = doc.get_element_by_tag_name("em").unwrap();
    assert_eq!(doc.text_content(em), "beautiful");
}

#[test]
fn test_self_closing_elements() {
    let doc = parse(r#"<p>Line 1<br>Line 2<br/>Line 3</p>"#);

    let p = doc.get_element_by_tag_name("p").unwrap();

    // Count br elements under p
    let br_count = doc
        .descendants(p)
        .filter(|&id| {
            doc.get(id)
                .and_then(|n| n.as_element())
                .map(|e| e.tag_name == "br")
                .unwrap_or(false)
        })
        .count();
    assert_eq!(br_count, 2);
}

#[test]
fn test_form_elements() {
    let html = r#"
    <form action="/submit" method="post">
        <input type="text" name="username" placeholder="Username">
        <input type="password" name="password">
        <button type="submit">Login</button>
    </form>
    "#;

    let doc = parse(html);

    let form = doc.get_element_by_tag_name("form").unwrap();
    let form_elem = doc.get(form).unwrap().as_element().unwrap();
    assert_eq!(form_elem.get_attribute("action"), Some("/submit"));
    assert_eq!(form_elem.get_attribute("method"), Some("post"));

    // Count input elements
    let input_count = doc
        .descendants(form)
        .filter(|&id| {
            doc.get(id)
                .and_then(|n| n.as_element())
                .map(|e| e.tag_name == "input")
                .unwrap_or(false)
        })
        .count();
    assert_eq!(input_count, 2);
}

#[test]
fn test_empty_elements() {
    let doc = parse("<div></div><span></span>");

    let div = doc.get_element_by_tag_name("div").unwrap();
    let span = doc.get_element_by_tag_name("span").unwrap();

    // Both should exist but have no children
    assert!(doc.children(div).next().is_none());
    assert!(doc.children(span).next().is_none());
}

#[test]
fn test_img_element() {
    let doc = parse(r#"<img src="photo.jpg" alt="A photo" width="100" height="200">"#);

    let img = doc.get_element_by_tag_name("img").unwrap();
    let elem = doc.get(img).unwrap().as_element().unwrap();

    assert_eq!(elem.get_attribute("src"), Some("photo.jpg"));
    assert_eq!(elem.get_attribute("alt"), Some("A photo"));
    assert_eq!(elem.get_attribute("width"), Some("100"));
    assert_eq!(elem.get_attribute("height"), Some("200"));
}

#[test]
fn test_links() {
    let doc = parse(r#"<a href="https://example.com" target="_blank">Click here</a>"#);

    let a = doc.get_element_by_tag_name("a").unwrap();
    let elem = doc.get(a).unwrap().as_element().unwrap();

    assert_eq!(elem.get_attribute("href"), Some("https://example.com"));
    assert_eq!(elem.get_attribute("target"), Some("_blank"));
    assert_eq!(doc.text_content(a), "Click here");
}

#[test]
fn test_deeply_nested() {
    let doc = parse("<div><div><div><div><p>Deep</p></div></div></div></div>");

    let p = doc.get_element_by_tag_name("p").unwrap();
    assert_eq!(doc.text_content(p), "Deep");

    // Count ancestor divs
    let div_count = doc
        .ancestors(p)
        .filter(|&id| {
            doc.get(id)
                .and_then(|n| n.as_element())
                .map(|e| e.tag_name == "div")
                .unwrap_or(false)
        })
        .count();
    assert_eq!(div_count, 4);
}

#[test]
fn test_comments_preserved() {
    let doc = parse("<div><!-- Comment 1 -->Text<!-- Comment 2 --></div>");

    let div = doc.get_element_by_tag_name("div").unwrap();

    let comment_count = doc
        .children(div)
        .filter(|&id| {
            doc.get(id)
                .map(|n| matches!(n.data, NodeData::Comment(_)))
                .unwrap_or(false)
        })
        .count();
    assert_eq!(comment_count, 2);
}

#[test]
fn test_text_with_special_chars() {
    let doc = parse("<p>Hello &amp; goodbye</p>");

    // HTML entities should be decoded
    let p = doc.get_element_by_tag_name("p").unwrap();
    let text = doc.text_content(p);
    assert_eq!(text, "Hello & goodbye");
}

#[test]
fn test_minimal_html() {
    let doc = parse("Hello, World!");

    // Should create implied html/body structure
    let body = doc.get_element_by_tag_name("body").unwrap();
    assert_eq!(doc.text_content(body), "Hello, World!");
}

#[test]
fn test_script_and_style_in_head() {
    let html = r#"
    <html>
    <head>
        <style>body { color: red; }</style>
        <script>console.log('hello');</script>
    </head>
    <body></body>
    </html>
    "#;

    let doc = parse(html);

    let style = doc.get_element_by_tag_name("style").unwrap();
    let script = doc.get_element_by_tag_name("script").unwrap();
    let head = doc.get_element_by_tag_name("head").unwrap();

    // Both should be in head
    assert_eq!(doc.get(style).unwrap().parent, Some(head));
    assert_eq!(doc.get(script).unwrap().parent, Some(head));
}

// Entity decoding tests

#[test]
fn test_entity_decoding_basic() {
    let doc = parse("<p>&amp;</p>");
    let p = doc.get_element_by_tag_name("p").unwrap();
    assert_eq!(doc.text_content(p), "&");
}

#[test]
fn test_entity_decoding_in_attributes() {
    let doc = parse(r#"<a href="?a=1&amp;b=2">link</a>"#);
    let a = doc.get_element_by_tag_name("a").unwrap();
    let href = &doc
        .get(a)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes
        .iter()
        .find(|(k, _)| k == "href")
        .unwrap()
        .1;
    assert_eq!(href, "?a=1&b=2");
}

#[test]
fn test_entity_decoding_numeric_decimal() {
    let doc = parse("<p>&#60;script&#62;</p>");
    let p = doc.get_element_by_tag_name("p").unwrap();
    assert_eq!(doc.text_content(p), "<script>");
}

#[test]
fn test_entity_decoding_numeric_hex() {
    let doc = parse("<p>&#x3C;script&#x3E;</p>");
    let p = doc.get_element_by_tag_name("p").unwrap();
    assert_eq!(doc.text_content(p), "<script>");
}

#[test]
fn test_entity_decoding_nbsp() {
    let doc = parse("<p>&nbsp;</p>");
    let p = doc.get_element_by_tag_name("p").unwrap();
    assert_eq!(doc.text_content(p), "\u{00A0}");
}

#[test]
fn test_entity_decoding_unknown() {
    // Unknown entities should be passed through unchanged
    let doc = parse("<p>&unknown;</p>");
    let p = doc.get_element_by_tag_name("p").unwrap();
    assert_eq!(doc.text_content(p), "&unknown;");
}

#[test]
fn test_entity_decoding_multiple() {
    let doc = parse("<p>&lt;div&gt;&amp;&lt;/div&gt;</p>");
    let p = doc.get_element_by_tag_name("p").unwrap();
    assert_eq!(doc.text_content(p), "<div>&</div>");
}
