//! CSS tokenizer, parser, and selector matching
//!
//! This crate provides CSS functionality for the vibeweb browser:
//! - `selector`: CSS selector types (type, class, ID, attribute, pseudo-class)
//! - `specificity`: Specificity calculation for cascade ordering
//! - `matching`: Selector matching against DOM elements
//!
//! # Example
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

pub mod matching;
pub mod selector;
pub mod specificity;

// Re-export commonly used items
pub use matching::{matches, matches_with_context, PseudoClassContext};
pub use selector::{Selector, CompoundSelector, Combinator, TypeSelector};
pub use specificity::Specificity;
