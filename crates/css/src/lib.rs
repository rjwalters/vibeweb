//! CSS tokenizer, parser, and selector matching
//!
//! This crate provides CSS functionality for the vibeweb browser:
//!
//! ## Selector Matching (from PR #33)
//! - `selector`: CSS selector types (type, class, ID, attribute, pseudo-class)
//! - `specificity`: Specificity calculation for cascade ordering
//! - `matching`: Selector matching against DOM elements
//!
//! ## CSS Parsing (from PR #34)
//! - `tokenizer`: Lexical analysis of CSS text into tokens
//! - `values`: CSS value types (lengths, colors, keywords)
//! - `selectors`: Selector parsing (internal, re-exported through selector module)
//! - `parser`: Rule parser that produces a Stylesheet
//!
//! # Example - Selector Matching
//!
//! ```
//! use vw_css::selector::{Selector, CompoundSelector, SelectorComponent, Combinator};
//! use vw_css::matching::matches;
//! use vw_css::specificity;
//! use vw_dom::Document;
//!
//! // Create a simple DOM
//! let mut doc = Document::new();
//! let div = doc.create_element_with_attributes(
//!     "div",
//!     vec![("class".to_string(), "container".to_string())],
//! );
//! doc.append_child(doc.root(), div);
//!
//! // Create a selector: div.container
//! let selector = Selector::simple(CompoundSelector {
//!     type_selector: Some(vw_css::selector::TypeSelector::Tag("div".to_string())),
//!     id: None,
//!     classes: vec!["container".to_string()],
//!     attributes: vec![],
//!     pseudo_classes: vec![],
//! });
//!
//! // Check if the selector matches the element
//! assert!(matches(&selector, div, &doc));
//!
//! // Calculate specificity
//! let spec = specificity::calculate(&selector);
//! assert_eq!(spec, vw_css::specificity::Specificity::new(0, 1, 1));
//! ```
//!
//! # Example - CSS Parsing
//!
//! ```
//! use vw_css::{Stylesheet, parse};
//!
//! let css = r#"
//!     body { color: red; }
//!     .container { width: 100%; }
//! "#;
//!
//! let stylesheet = parse(css);
//! assert_eq!(stylesheet.rules.len(), 2);
//! ```

// Selector matching modules (from PR #33)
pub mod matching;
pub mod selector;
pub mod specificity;

// CSS parsing modules (from PR #34)
mod parser;
mod tokenizer;
mod values;

// Re-export selector matching items
pub use matching::{matches, matches_with_context, PseudoClassContext};
pub use selector::{
    AttributeMatcher, AttributeSelector, Combinator, CompoundSelector, NthFormula, PseudoClass,
    Selector, SelectorComponent, TypeSelector,
};
pub use specificity::Specificity;

// Re-export CSS parsing items
pub use parser::{parse, Declaration, Rule, Stylesheet};
pub use tokenizer::{Token, TokenKind, Tokenizer};
pub use values::{Color, CssValue, Length, LengthUnit};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_rule() {
        let css = "body { color: red; }";
        let stylesheet = parse(css);
        assert_eq!(stylesheet.rules.len(), 1);

        let rule = &stylesheet.rules[0];
        assert_eq!(rule.selectors.len(), 1);
        assert_eq!(rule.declarations.len(), 1);

        let decl = &rule.declarations[0];
        assert_eq!(decl.property, "color");
    }

    #[test]
    fn parse_multiple_rules() {
        let css = r#"
            body { margin: 0; }
            .container { width: 100%; }
            #header { background: blue; }
        "#;
        let stylesheet = parse(css);
        assert_eq!(stylesheet.rules.len(), 3);
    }

    #[test]
    fn parse_multiple_declarations() {
        let css = "div { color: red; background: blue; margin: 10px; }";
        let stylesheet = parse(css);
        assert_eq!(stylesheet.rules[0].declarations.len(), 3);
    }

    #[test]
    fn parse_selector_list() {
        let css = "h1, h2, h3 { font-weight: bold; }";
        let stylesheet = parse(css);
        assert_eq!(stylesheet.rules[0].selectors.len(), 3);
    }

    #[test]
    fn parse_descendant_combinator() {
        let css = "div p { color: red; }";
        let stylesheet = parse(css);
        let sel = &stylesheet.rules[0].selectors[0];
        let compounds: Vec<_> = sel.compound_selectors().collect();
        assert_eq!(compounds.len(), 2);
        // Check that there's a descendant combinator
        assert!(sel
            .components
            .iter()
            .any(|c| matches!(c, SelectorComponent::Combinator(Combinator::Descendant))));
    }

    #[test]
    fn parse_child_combinator() {
        let css = "div > p { color: red; }";
        let stylesheet = parse(css);
        let sel = &stylesheet.rules[0].selectors[0];
        let compounds: Vec<_> = sel.compound_selectors().collect();
        assert_eq!(compounds.len(), 2);
        assert!(sel
            .components
            .iter()
            .any(|c| matches!(c, SelectorComponent::Combinator(Combinator::Child))));
    }

    #[test]
    fn parse_class_selector() {
        let css = ".highlight { background: yellow; }";
        let stylesheet = parse(css);
        let sel = &stylesheet.rules[0].selectors[0];
        let compound = sel.compound_selectors().next().unwrap();
        assert_eq!(compound.classes.len(), 1);
        assert_eq!(compound.classes[0], "highlight");
    }

    #[test]
    fn parse_id_selector() {
        let css = "#main { width: 960px; }";
        let stylesheet = parse(css);
        let sel = &stylesheet.rules[0].selectors[0];
        let compound = sel.compound_selectors().next().unwrap();
        assert_eq!(compound.id, Some("main".to_string()));
    }

    #[test]
    fn parse_combined_selector() {
        let css = "div.container#main { padding: 10px; }";
        let stylesheet = parse(css);
        let sel = &stylesheet.rules[0].selectors[0];
        let compound = sel.compound_selectors().next().unwrap();
        assert_eq!(
            compound.type_selector,
            Some(TypeSelector::Tag("div".to_string()))
        );
        assert_eq!(compound.classes, vec!["container".to_string()]);
        assert_eq!(compound.id, Some("main".to_string()));
    }

    #[test]
    fn parse_length_values() {
        let css = "div { width: 100px; height: 50%; margin: 2em; }";
        let stylesheet = parse(css);
        let decls = &stylesheet.rules[0].declarations;

        assert!(matches!(
            &decls[0].value,
            CssValue::Length(Length {
                value: 100.0,
                unit: LengthUnit::Px
            })
        ));
        assert!(matches!(&decls[1].value, CssValue::Percentage(50.0)));
        assert!(matches!(
            &decls[2].value,
            CssValue::Length(Length {
                value: 2.0,
                unit: LengthUnit::Em
            })
        ));
    }

    #[test]
    fn parse_color_values() {
        let css = "div { color: red; background: #ff0000; border-color: rgb(255, 0, 0); }";
        let stylesheet = parse(css);
        let decls = &stylesheet.rules[0].declarations;

        // Named color
        assert!(matches!(&decls[0].value, CssValue::Color(_)));
        // Hex color
        assert!(matches!(&decls[1].value, CssValue::Color(_)));
        // RGB function
        assert!(matches!(&decls[2].value, CssValue::Color(_)));
    }

    #[test]
    fn parse_important() {
        let css = "div { color: red !important; }";
        let stylesheet = parse(css);
        assert!(stylesheet.rules[0].declarations[0].important);
    }

    #[test]
    fn handle_comments() {
        let css = r#"
            /* This is a comment */
            body { color: red; /* inline comment */ }
        "#;
        let stylesheet = parse(css);
        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn empty_stylesheet() {
        let css = "";
        let stylesheet = parse(css);
        assert!(stylesheet.rules.is_empty());
    }

    #[test]
    fn whitespace_only() {
        let css = "   \n\t  ";
        let stylesheet = parse(css);
        assert!(stylesheet.rules.is_empty());
    }

    #[test]
    fn universal_selector() {
        let css = "* { margin: 0; }";
        let stylesheet = parse(css);
        let sel = &stylesheet.rules[0].selectors[0];
        let compound = sel.compound_selectors().next().unwrap();
        assert_eq!(compound.type_selector, Some(TypeSelector::Universal));
    }

    #[test]
    fn attribute_selector() {
        let css = "[disabled] { opacity: 0.5; }";
        let stylesheet = parse(css);
        let sel = &stylesheet.rules[0].selectors[0];
        let compound = sel.compound_selectors().next().unwrap();
        assert_eq!(compound.attributes.len(), 1);
    }

    #[test]
    fn attribute_selector_with_value() {
        let css = r#"[type="text"] { border: 1px solid; }"#;
        let stylesheet = parse(css);
        let sel = &stylesheet.rules[0].selectors[0];
        let compound = sel.compound_selectors().next().unwrap();
        let attr = &compound.attributes[0];
        assert_eq!(attr.name, "type");
        assert!(matches!(
            &attr.matcher,
            Some(AttributeMatcher::Exact(v)) if v == "text"
        ));
    }

    #[test]
    fn pseudo_class() {
        let css = "a:hover { color: blue; }";
        let stylesheet = parse(css);
        let sel = &stylesheet.rules[0].selectors[0];
        let compound = sel.compound_selectors().next().unwrap();
        assert!(compound
            .pseudo_classes
            .iter()
            .any(|pc| matches!(pc, PseudoClass::Hover)));
    }
}
