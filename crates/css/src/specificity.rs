//! CSS selector specificity calculation
//!
//! Specificity determines which CSS rule takes precedence when multiple
//! rules match the same element. It is calculated as a tuple (A, B, C) where:
//! - A = count of ID selectors
//! - B = count of class selectors, attribute selectors, and pseudo-classes
//! - C = count of type selectors and pseudo-elements

use crate::selector::{
    CompoundSelector, PseudoClass, Selector, SelectorComponent, TypeSelector,
};

/// CSS selector specificity.
///
/// Represented as (IDs, Classes, Elements) tuple. Higher values take precedence.
/// Comparison is done from left to right: first compare IDs, then classes, then elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Specificity {
    /// Count of ID selectors (#id).
    pub ids: u32,
    /// Count of class selectors (.class), attribute selectors ([attr]), and pseudo-classes (:hover).
    pub classes: u32,
    /// Count of type selectors (div, p) and pseudo-elements (::before).
    pub elements: u32,
}

impl Specificity {
    /// Creates a new specificity with the given values.
    pub fn new(ids: u32, classes: u32, elements: u32) -> Self {
        Self {
            ids,
            classes,
            elements,
        }
    }

    /// Returns a specificity of zero.
    pub fn zero() -> Self {
        Self::default()
    }

    /// Adds another specificity to this one.
    pub fn add(&mut self, other: Specificity) {
        self.ids += other.ids;
        self.classes += other.classes;
        self.elements += other.elements;
    }

    /// Returns a new specificity that is the sum of this and another.
    pub fn plus(self, other: Specificity) -> Self {
        Self {
            ids: self.ids + other.ids,
            classes: self.classes + other.classes,
            elements: self.elements + other.elements,
        }
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare IDs first, then classes, then elements
        self.ids
            .cmp(&other.ids)
            .then_with(|| self.classes.cmp(&other.classes))
            .then_with(|| self.elements.cmp(&other.elements))
    }
}

/// Calculates the specificity of a selector.
pub fn calculate(selector: &Selector) -> Specificity {
    let mut spec = Specificity::zero();

    for component in &selector.components {
        match component {
            SelectorComponent::Compound(compound) => {
                spec.add(calculate_compound(compound));
            }
            SelectorComponent::Combinator(_) => {
                // Combinators don't contribute to specificity
            }
        }
    }

    spec
}

/// Calculates the specificity of a compound selector.
pub fn calculate_compound(compound: &CompoundSelector) -> Specificity {
    let mut spec = Specificity::zero();

    // Type selector: +1 element (unless universal)
    if let Some(type_sel) = &compound.type_selector {
        match type_sel {
            TypeSelector::Universal => {
                // Universal selector has no specificity
            }
            TypeSelector::Tag(_) => {
                spec.elements += 1;
            }
        }
    }

    // ID selector: +1 id
    if compound.id.is_some() {
        spec.ids += 1;
    }

    // Class selectors: +1 class each
    spec.classes += compound.classes.len() as u32;

    // Attribute selectors: +1 class each
    spec.classes += compound.attributes.len() as u32;

    // Pseudo-classes: +1 class each (with exceptions)
    for pseudo in &compound.pseudo_classes {
        match pseudo {
            // :not() has the specificity of its argument
            PseudoClass::Not(inner) => {
                spec.add(calculate_compound(inner));
            }
            // All other pseudo-classes: +1 class
            _ => {
                spec.classes += 1;
            }
        }
    }

    spec
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selector::Combinator;

    #[test]
    fn test_specificity_ordering() {
        // (1,0,0) > (0,10,10)
        assert!(Specificity::new(1, 0, 0) > Specificity::new(0, 10, 10));

        // (0,1,0) > (0,0,10)
        assert!(Specificity::new(0, 1, 0) > Specificity::new(0, 0, 10));

        // (0,0,1) > (0,0,0)
        assert!(Specificity::new(0, 0, 1) > Specificity::new(0, 0, 0));

        // Equal specificities
        assert_eq!(Specificity::new(1, 2, 3), Specificity::new(1, 2, 3));
    }

    #[test]
    fn test_type_selector() {
        // div -> (0,0,1)
        let selector = Selector::simple(CompoundSelector::with_type("div"));
        assert_eq!(calculate(&selector), Specificity::new(0, 0, 1));
    }

    #[test]
    fn test_class_selector() {
        // .container -> (0,1,0)
        let selector = Selector::simple(CompoundSelector::with_class("container"));
        assert_eq!(calculate(&selector), Specificity::new(0, 1, 0));
    }

    #[test]
    fn test_id_selector() {
        // #main -> (1,0,0)
        let selector = Selector::simple(CompoundSelector::with_id("main"));
        assert_eq!(calculate(&selector), Specificity::new(1, 0, 0));
    }

    #[test]
    fn test_universal_selector() {
        // * -> (0,0,0)
        let selector = Selector::simple(CompoundSelector::universal());
        assert_eq!(calculate(&selector), Specificity::new(0, 0, 0));
    }

    #[test]
    fn test_compound_selector() {
        // div.container#main -> (1,1,1)
        let compound = CompoundSelector {
            type_selector: Some(TypeSelector::Tag("div".to_string())),
            id: Some("main".to_string()),
            classes: vec!["container".to_string()],
            attributes: vec![],
            pseudo_classes: vec![],
        };
        let selector = Selector::simple(compound);
        assert_eq!(calculate(&selector), Specificity::new(1, 1, 1));
    }

    #[test]
    fn test_attribute_selector() {
        // [type="text"] -> (0,1,0)
        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![crate::selector::AttributeSelector::exact("type", "text")],
            pseudo_classes: vec![],
        };
        let selector = Selector::simple(compound);
        assert_eq!(calculate(&selector), Specificity::new(0, 1, 0));
    }

    #[test]
    fn test_pseudo_class() {
        // :hover -> (0,1,0)
        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::Hover],
        };
        let selector = Selector::simple(compound);
        assert_eq!(calculate(&selector), Specificity::new(0, 1, 0));
    }

    #[test]
    fn test_complex_selector() {
        // div.container > p.intro -> (0,2,2)
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector {
                type_selector: Some(TypeSelector::Tag("div".to_string())),
                id: None,
                classes: vec!["container".to_string()],
                attributes: vec![],
                pseudo_classes: vec![],
            }),
            SelectorComponent::Combinator(Combinator::Child),
            SelectorComponent::Compound(CompoundSelector {
                type_selector: Some(TypeSelector::Tag("p".to_string())),
                id: None,
                classes: vec!["intro".to_string()],
                attributes: vec![],
                pseudo_classes: vec![],
            }),
        ]);
        assert_eq!(calculate(&selector), Specificity::new(0, 2, 2));
    }

    #[test]
    fn test_not_pseudo_class() {
        // :not(.hidden) -> (0,1,0) - specificity of the argument
        let compound = CompoundSelector {
            type_selector: None,
            id: None,
            classes: vec![],
            attributes: vec![],
            pseudo_classes: vec![PseudoClass::Not(Box::new(CompoundSelector::with_class(
                "hidden",
            )))],
        };
        let selector = Selector::simple(compound);
        assert_eq!(calculate(&selector), Specificity::new(0, 1, 0));
    }

    #[test]
    fn test_specificity_addition() {
        let a = Specificity::new(1, 2, 3);
        let b = Specificity::new(0, 1, 2);
        assert_eq!(a.plus(b), Specificity::new(1, 3, 5));
    }
}
