//! CSS selector types
//!
//! This module defines the types representing CSS selectors. These types
//! will be produced by the CSS parser and consumed by the selector matching
//! engine.

/// A complete selector, consisting of one or more compound selectors
/// connected by combinators.
///
/// Example: `div.container > p.intro` is a selector with two compound
/// selectors connected by a child combinator.
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    /// The compound selectors and combinators that make up this selector.
    /// Always starts with a compound selector, then alternates combinator/compound.
    pub components: Vec<SelectorComponent>,
}

impl Selector {
    /// Creates a new selector from a single compound selector.
    pub fn simple(compound: CompoundSelector) -> Self {
        Self {
            components: vec![SelectorComponent::Compound(compound)],
        }
    }

    /// Creates a new selector from components.
    pub fn new(components: Vec<SelectorComponent>) -> Self {
        Self { components }
    }

    /// Returns the compound selectors in this selector (filtering out combinators).
    pub fn compound_selectors(&self) -> impl Iterator<Item = &CompoundSelector> {
        self.components.iter().filter_map(|c| match c {
            SelectorComponent::Compound(cs) => Some(cs),
            SelectorComponent::Combinator(_) => None,
        })
    }
}

/// A component of a selector: either a compound selector or a combinator.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectorComponent {
    /// A compound selector (e.g., `div.class#id`).
    Compound(CompoundSelector),
    /// A combinator connecting compound selectors.
    Combinator(Combinator),
}

/// A compound selector is a sequence of simple selectors without combinators.
///
/// Example: `div.container#main` is a compound selector with a type selector,
/// a class selector, and an ID selector.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompoundSelector {
    /// Type selector (e.g., `div`, `p`). None means universal or no type constraint.
    pub type_selector: Option<TypeSelector>,
    /// ID selector (e.g., `#main`).
    pub id: Option<String>,
    /// Class selectors (e.g., `.container`, `.active`).
    pub classes: Vec<String>,
    /// Attribute selectors (e.g., `[href]`, `[type="text"]`).
    pub attributes: Vec<AttributeSelector>,
    /// Pseudo-class selectors (e.g., `:hover`, `:first-child`).
    pub pseudo_classes: Vec<PseudoClass>,
}

impl CompoundSelector {
    /// Creates a new empty compound selector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a compound selector with just a type selector.
    pub fn with_type(tag_name: impl Into<String>) -> Self {
        Self {
            type_selector: Some(TypeSelector::Tag(tag_name.into())),
            ..Default::default()
        }
    }

    /// Creates a compound selector with just a class.
    pub fn with_class(class_name: impl Into<String>) -> Self {
        Self {
            classes: vec![class_name.into()],
            ..Default::default()
        }
    }

    /// Creates a compound selector with just an ID.
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            ..Default::default()
        }
    }

    /// Creates a universal selector (*).
    pub fn universal() -> Self {
        Self {
            type_selector: Some(TypeSelector::Universal),
            ..Default::default()
        }
    }

    /// Returns true if this is a universal selector with no other constraints.
    pub fn is_pure_universal(&self) -> bool {
        matches!(self.type_selector, Some(TypeSelector::Universal))
            && self.id.is_none()
            && self.classes.is_empty()
            && self.attributes.is_empty()
            && self.pseudo_classes.is_empty()
    }
}

/// Type selector: either a specific tag name or universal (*).
#[derive(Debug, Clone, PartialEq)]
pub enum TypeSelector {
    /// Universal selector (*) - matches any element.
    Universal,
    /// Tag name selector (e.g., `div`, `p`, `html`).
    Tag(String),
}

/// Combinator connecting compound selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant combinator (space): `A B` matches B that is a descendant of A.
    Descendant,
    /// Child combinator (>): `A > B` matches B that is a direct child of A.
    Child,
    /// Adjacent sibling combinator (+): `A + B` matches B immediately preceded by A.
    AdjacentSibling,
    /// General sibling combinator (~): `A ~ B` matches B preceded by A (not necessarily immediately).
    GeneralSibling,
}

/// Attribute selector.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSelector {
    /// The attribute name to match.
    pub name: String,
    /// The matching operation (if any).
    pub matcher: Option<AttributeMatcher>,
}

impl AttributeSelector {
    /// Creates an attribute presence selector (e.g., `[disabled]`).
    pub fn presence(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            matcher: None,
        }
    }

    /// Creates an exact value attribute selector (e.g., `[type="text"]`).
    pub fn exact(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            matcher: Some(AttributeMatcher::Exact(value.into())),
        }
    }

    /// Creates a whitespace-separated word attribute selector (e.g., `[class~="active"]`).
    pub fn word(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            matcher: Some(AttributeMatcher::Word(value.into())),
        }
    }

    /// Creates a prefix attribute selector (e.g., `[href^="https"]`).
    pub fn prefix(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            matcher: Some(AttributeMatcher::Prefix(value.into())),
        }
    }

    /// Creates a suffix attribute selector (e.g., `[src$=".png"]`).
    pub fn suffix(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            matcher: Some(AttributeMatcher::Suffix(value.into())),
        }
    }

    /// Creates a substring attribute selector (e.g., `[title*="hello"]`).
    pub fn substring(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            matcher: Some(AttributeMatcher::Substring(value.into())),
        }
    }
}

/// Attribute value matching operation.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeMatcher {
    /// Exact value match: `[attr="value"]`.
    Exact(String),
    /// Whitespace-separated word match: `[attr~="value"]`.
    Word(String),
    /// Prefix match: `[attr^="value"]`.
    Prefix(String),
    /// Suffix match: `[attr$="value"]`.
    Suffix(String),
    /// Substring match: `[attr*="value"]`.
    Substring(String),
    /// Hyphen-separated prefix match: `[attr|="value"]`.
    HyphenPrefix(String),
}

/// Pseudo-class selector.
#[derive(Debug, Clone, PartialEq)]
pub enum PseudoClass {
    /// :hover
    Hover,
    /// :active
    Active,
    /// :focus
    Focus,
    /// :visited
    Visited,
    /// :link
    Link,
    /// :first-child
    FirstChild,
    /// :last-child
    LastChild,
    /// :only-child
    OnlyChild,
    /// :first-of-type
    FirstOfType,
    /// :last-of-type
    LastOfType,
    /// :only-of-type
    OnlyOfType,
    /// :empty
    Empty,
    /// :root
    Root,
    /// :enabled
    Enabled,
    /// :disabled
    Disabled,
    /// :checked
    Checked,
    /// :nth-child(An+B)
    NthChild(NthFormula),
    /// :nth-last-child(An+B)
    NthLastChild(NthFormula),
    /// :nth-of-type(An+B)
    NthOfType(NthFormula),
    /// :nth-last-of-type(An+B)
    NthLastOfType(NthFormula),
    /// :not(selector)
    Not(Box<CompoundSelector>),
}

/// An+B formula for :nth-* pseudo-classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NthFormula {
    /// The 'A' coefficient (step size).
    pub a: i32,
    /// The 'B' offset.
    pub b: i32,
}

impl NthFormula {
    /// Creates a new An+B formula.
    pub fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    /// Creates an "odd" formula (2n+1).
    pub fn odd() -> Self {
        Self { a: 2, b: 1 }
    }

    /// Creates an "even" formula (2n).
    pub fn even() -> Self {
        Self { a: 2, b: 0 }
    }

    /// Checks if a 1-indexed position matches this formula.
    pub fn matches(&self, position: i32) -> bool {
        if self.a == 0 {
            // Just B: matches only when position == b
            position == self.b
        } else {
            // An+B: position = a*n + b for some non-negative integer n
            // Solve: n = (position - b) / a
            let diff = position - self.b;
            if self.a > 0 {
                diff >= 0 && diff % self.a == 0
            } else {
                // a < 0: need diff <= 0 and divisible
                diff <= 0 && diff % self.a == 0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nth_formula_odd() {
        let odd = NthFormula::odd();
        assert!(odd.matches(1));
        assert!(!odd.matches(2));
        assert!(odd.matches(3));
        assert!(!odd.matches(4));
        assert!(odd.matches(5));
    }

    #[test]
    fn test_nth_formula_even() {
        let even = NthFormula::even();
        assert!(!even.matches(1));
        assert!(even.matches(2));
        assert!(!even.matches(3));
        assert!(even.matches(4));
    }

    #[test]
    fn test_nth_formula_just_b() {
        let third = NthFormula::new(0, 3);
        assert!(!third.matches(1));
        assert!(!third.matches(2));
        assert!(third.matches(3));
        assert!(!third.matches(4));
    }

    #[test]
    fn test_nth_formula_3n() {
        let every_third = NthFormula::new(3, 0);
        assert!(!every_third.matches(1));
        assert!(!every_third.matches(2));
        assert!(every_third.matches(3));
        assert!(!every_third.matches(4));
        assert!(!every_third.matches(5));
        assert!(every_third.matches(6));
    }

    #[test]
    fn test_compound_selector() {
        let cs = CompoundSelector::with_type("div");
        assert_eq!(cs.type_selector, Some(TypeSelector::Tag("div".to_string())));
        assert!(cs.classes.is_empty());

        let cs = CompoundSelector::with_class("container");
        assert_eq!(cs.classes, vec!["container".to_string()]);
    }

    #[test]
    fn test_selector_components() {
        let selector = Selector::new(vec![
            SelectorComponent::Compound(CompoundSelector::with_type("div")),
            SelectorComponent::Combinator(Combinator::Child),
            SelectorComponent::Compound(CompoundSelector::with_class("item")),
        ]);

        let compounds: Vec<_> = selector.compound_selectors().collect();
        assert_eq!(compounds.len(), 2);
    }
}
