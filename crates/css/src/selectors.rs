//! CSS Selector Types
//!
//! Represents CSS selectors including type, class, ID, attribute,
//! and pseudo-class selectors, as well as combinators.

use std::fmt;

/// A complete CSS selector (chain of simple selectors with combinators)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Selector {
    /// The simple selectors in this chain
    pub simple_selectors: Vec<SimpleSelector>,
    /// The combinators between simple selectors (one fewer than simple_selectors)
    pub combinators: Vec<Combinator>,
}

impl Selector {
    /// Create a new empty selector
    pub fn new() -> Self {
        Selector {
            simple_selectors: Vec::new(),
            combinators: Vec::new(),
        }
    }

    /// Add a simple selector with a combinator
    pub fn add(&mut self, combinator: Option<Combinator>, simple: SimpleSelector) {
        if let Some(c) = combinator {
            if !self.simple_selectors.is_empty() {
                self.combinators.push(c);
            }
        }
        self.simple_selectors.push(simple);
    }

    /// Calculate the specificity of this selector
    /// Returns (ids, classes, elements) tuple
    pub fn specificity(&self) -> (u32, u32, u32) {
        let mut ids = 0u32;
        let mut classes = 0u32;
        let mut elements = 0u32;

        for simple in &self.simple_selectors {
            let (i, c, e) = simple.specificity();
            ids += i;
            classes += c;
            elements += e;
        }

        (ids, classes, elements)
    }

    /// Compare specificities for ordering
    pub fn specificity_cmp(&self, other: &Selector) -> std::cmp::Ordering {
        let (a_ids, a_classes, a_elements) = self.specificity();
        let (b_ids, b_classes, b_elements) = other.specificity();

        a_ids
            .cmp(&b_ids)
            .then(a_classes.cmp(&b_classes))
            .then(a_elements.cmp(&b_elements))
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, simple) in self.simple_selectors.iter().enumerate() {
            if i > 0 {
                if let Some(comb) = self.combinators.get(i - 1) {
                    write!(f, "{}", comb)?;
                }
            }
            write!(f, "{}", simple)?;
        }
        Ok(())
    }
}

/// A simple selector (single element in a selector chain)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SimpleSelector {
    /// Universal selector (*)
    pub universal: bool,
    /// Tag name (e.g., "div", "p")
    pub tag_name: Option<String>,
    /// ID selector (e.g., #main)
    pub id: Option<String>,
    /// Class selectors (e.g., .container, .highlight)
    pub classes: Vec<String>,
    /// Attribute selectors (e.g., [disabled], [type="text"])
    pub attributes: Vec<AttributeSelector>,
    /// Pseudo-classes (e.g., :hover, :first-child)
    pub pseudo_classes: Vec<String>,
    /// Pseudo-elements (e.g., ::before, ::after)
    pub pseudo_elements: Vec<String>,
}

impl SimpleSelector {
    /// Create a new empty simple selector
    pub fn new() -> Self {
        SimpleSelector::default()
    }

    /// Create a universal selector (*)
    pub fn universal() -> Self {
        SimpleSelector {
            universal: true,
            ..Default::default()
        }
    }

    /// Create a type selector (tag name)
    pub fn tag(name: &str) -> Self {
        SimpleSelector {
            tag_name: Some(name.to_lowercase()),
            ..Default::default()
        }
    }

    /// Create a class selector
    pub fn class(name: &str) -> Self {
        SimpleSelector {
            classes: vec![name.to_string()],
            ..Default::default()
        }
    }

    /// Create an ID selector
    pub fn id(name: &str) -> Self {
        SimpleSelector {
            id: Some(name.to_string()),
            ..Default::default()
        }
    }

    /// Check if this selector is empty (no constraints)
    pub fn is_empty(&self) -> bool {
        !self.universal
            && self.tag_name.is_none()
            && self.id.is_none()
            && self.classes.is_empty()
            && self.attributes.is_empty()
            && self.pseudo_classes.is_empty()
            && self.pseudo_elements.is_empty()
    }

    /// Calculate the specificity of this simple selector
    /// Returns (ids, classes, elements) tuple
    pub fn specificity(&self) -> (u32, u32, u32) {
        let ids = if self.id.is_some() { 1 } else { 0 };

        let classes = self.classes.len() as u32
            + self.attributes.len() as u32
            + self.pseudo_classes.len() as u32;

        let elements = if self.tag_name.is_some() { 1 } else { 0 }
            + self.pseudo_elements.len() as u32;

        (ids, classes, elements)
    }
}

impl fmt::Display for SimpleSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.universal {
            write!(f, "*")?;
        }
        if let Some(ref tag) = self.tag_name {
            write!(f, "{}", tag)?;
        }
        if let Some(ref id) = self.id {
            write!(f, "#{}", id)?;
        }
        for class in &self.classes {
            write!(f, ".{}", class)?;
        }
        for attr in &self.attributes {
            write!(f, "{}", attr)?;
        }
        for pseudo in &self.pseudo_classes {
            write!(f, ":{}", pseudo)?;
        }
        for pseudo in &self.pseudo_elements {
            write!(f, "::{}", pseudo)?;
        }
        Ok(())
    }
}

/// Combinator between simple selectors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant combinator (space): "div p"
    Descendant,
    /// Child combinator (>): "div > p"
    Child,
    /// Adjacent sibling combinator (+): "div + p"
    AdjacentSibling,
    /// General sibling combinator (~): "div ~ p"
    GeneralSibling,
}

impl fmt::Display for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Combinator::Descendant => write!(f, " "),
            Combinator::Child => write!(f, " > "),
            Combinator::AdjacentSibling => write!(f, " + "),
            Combinator::GeneralSibling => write!(f, " ~ "),
        }
    }
}

/// An attribute selector
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSelector {
    /// The attribute name
    pub name: String,
    /// The comparison operator (if any)
    pub op: Option<AttributeOp>,
    /// The value to compare against (if any)
    pub value: Option<String>,
    /// Case-insensitive flag (i)
    pub case_insensitive: bool,
}

impl AttributeSelector {
    /// Create a presence selector [attr]
    pub fn presence(name: &str) -> Self {
        AttributeSelector {
            name: name.to_string(),
            op: None,
            value: None,
            case_insensitive: false,
        }
    }

    /// Create an exact match selector [attr=value]
    pub fn exact(name: &str, value: &str) -> Self {
        AttributeSelector {
            name: name.to_string(),
            op: Some(AttributeOp::Exact),
            value: Some(value.to_string()),
            case_insensitive: false,
        }
    }
}

impl fmt::Display for AttributeSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}", self.name)?;
        if let (Some(op), Some(val)) = (&self.op, &self.value) {
            write!(f, "{}\"{}\"", op, val)?;
            if self.case_insensitive {
                write!(f, " i")?;
            }
        }
        write!(f, "]")
    }
}

/// Attribute selector comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOp {
    /// Exact match: [attr=value]
    Exact,
    /// Word match: [attr~=value]
    Includes,
    /// Dash match: [attr|=value]
    DashMatch,
    /// Prefix match: [attr^=value]
    Prefix,
    /// Suffix match: [attr$=value]
    Suffix,
    /// Substring match: [attr*=value]
    Substring,
}

impl fmt::Display for AttributeOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeOp::Exact => write!(f, "="),
            AttributeOp::Includes => write!(f, "~="),
            AttributeOp::DashMatch => write!(f, "|="),
            AttributeOp::Prefix => write!(f, "^="),
            AttributeOp::Suffix => write!(f, "$="),
            AttributeOp::Substring => write!(f, "*="),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_selector_tag() {
        let sel = SimpleSelector::tag("div");
        assert_eq!(sel.tag_name, Some("div".to_string()));
        assert_eq!(format!("{}", sel), "div");
    }

    #[test]
    fn simple_selector_class() {
        let sel = SimpleSelector::class("container");
        assert_eq!(sel.classes, vec!["container".to_string()]);
        assert_eq!(format!("{}", sel), ".container");
    }

    #[test]
    fn simple_selector_id() {
        let sel = SimpleSelector::id("main");
        assert_eq!(sel.id, Some("main".to_string()));
        assert_eq!(format!("{}", sel), "#main");
    }

    #[test]
    fn simple_selector_universal() {
        let sel = SimpleSelector::universal();
        assert!(sel.universal);
        assert_eq!(format!("{}", sel), "*");
    }

    #[test]
    fn specificity_id() {
        let sel = SimpleSelector::id("main");
        assert_eq!(sel.specificity(), (1, 0, 0));
    }

    #[test]
    fn specificity_class() {
        let sel = SimpleSelector::class("container");
        assert_eq!(sel.specificity(), (0, 1, 0));
    }

    #[test]
    fn specificity_tag() {
        let sel = SimpleSelector::tag("div");
        assert_eq!(sel.specificity(), (0, 0, 1));
    }

    #[test]
    fn specificity_combined() {
        // div.container#main has specificity (1, 1, 1)
        let sel = SimpleSelector {
            tag_name: Some("div".to_string()),
            id: Some("main".to_string()),
            classes: vec!["container".to_string()],
            ..Default::default()
        };
        assert_eq!(sel.specificity(), (1, 1, 1));
    }

    #[test]
    fn selector_chain_specificity() {
        // div > p.highlight = (0, 1, 2)
        let mut selector = Selector::new();
        selector.add(None, SimpleSelector::tag("div"));
        selector.add(Some(Combinator::Child), SimpleSelector {
            tag_name: Some("p".to_string()),
            classes: vec!["highlight".to_string()],
            ..Default::default()
        });
        assert_eq!(selector.specificity(), (0, 1, 2));
    }

    #[test]
    fn combinator_display() {
        assert_eq!(format!("{}", Combinator::Descendant), " ");
        assert_eq!(format!("{}", Combinator::Child), " > ");
        assert_eq!(format!("{}", Combinator::AdjacentSibling), " + ");
        assert_eq!(format!("{}", Combinator::GeneralSibling), " ~ ");
    }

    #[test]
    fn attribute_selector_presence() {
        let sel = AttributeSelector::presence("disabled");
        assert_eq!(format!("{}", sel), "[disabled]");
    }

    #[test]
    fn attribute_selector_exact() {
        let sel = AttributeSelector::exact("type", "text");
        assert_eq!(format!("{}", sel), "[type=\"text\"]");
    }
}
