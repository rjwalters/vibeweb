//! CSS Parser
//!
//! Recursive descent parser that produces a Stylesheet from CSS text.
//! This is a simplified parser focused on common CSS patterns.

use crate::selectors::{AttributeOp, AttributeSelector, Combinator, Selector, SimpleSelector};
use crate::tokenizer::{Token, TokenKind, Tokenizer};
use crate::values::{Color, CssValue, Length, LengthUnit};

/// A parsed CSS stylesheet
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    /// The rules in this stylesheet
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    /// Create a new empty stylesheet
    pub fn new() -> Self {
        Stylesheet { rules: Vec::new() }
    }

    /// Debug dump of the stylesheet
    pub fn dump(&self) -> String {
        let mut out = String::new();
        for (i, rule) in self.rules.iter().enumerate() {
            out.push_str(&format!("Rule {}:\n", i));
            out.push_str(&format!("  Selectors: {:?}\n", rule.selectors));
            out.push_str(&format!("  Declarations: {:?}\n", rule.declarations));
        }
        out
    }
}

/// A CSS rule (selectors + declarations)
#[derive(Debug, Clone)]
pub struct Rule {
    /// The selectors that this rule matches
    pub selectors: Vec<Selector>,
    /// The declarations in this rule
    pub declarations: Vec<Declaration>,
}

impl Rule {
    /// Create a new rule
    pub fn new(selectors: Vec<Selector>, declarations: Vec<Declaration>) -> Self {
        Rule {
            selectors,
            declarations,
        }
    }
}

/// A CSS declaration (property: value)
#[derive(Debug, Clone)]
pub struct Declaration {
    /// The property name
    pub property: String,
    /// The property value
    pub value: CssValue,
    /// Whether this declaration has !important
    pub important: bool,
}

impl Declaration {
    /// Create a new declaration
    pub fn new(property: String, value: CssValue, important: bool) -> Self {
        Declaration {
            property,
            value,
            important,
        }
    }
}

/// Parse CSS text into a Stylesheet
pub fn parse(input: &str) -> Stylesheet {
    let mut parser = Parser::new(input);
    parser.parse_stylesheet()
}

/// CSS parser state
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        let tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize();
        Parser { tokens, pos: 0 }
    }

    /// Peek at the current token
    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    /// Consume the current token
    fn advance(&mut self) -> TokenKind {
        let token = self
            .tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof);
        self.pos += 1;
        token
    }

    /// Skip whitespace tokens
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), TokenKind::Whitespace) {
            self.advance();
        }
    }

    /// Consume a specific token kind
    fn expect(&mut self, expected: &TokenKind) -> bool {
        self.skip_whitespace();
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Parse a complete stylesheet
    fn parse_stylesheet(&mut self) -> Stylesheet {
        let mut stylesheet = Stylesheet::new();

        loop {
            self.skip_whitespace();

            match self.peek() {
                TokenKind::Eof => break,
                TokenKind::AtKeyword(_) => {
                    // Skip @rules for now (could add @import, @media support later)
                    self.skip_at_rule();
                }
                _ => {
                    if let Some(rule) = self.parse_rule() {
                        stylesheet.rules.push(rule);
                    } else {
                        // Skip to next rule on error
                        self.skip_to_next_rule();
                    }
                }
            }
        }

        stylesheet
    }

    /// Skip an @rule
    fn skip_at_rule(&mut self) {
        self.advance(); // Consume @keyword

        let mut brace_depth = 0;

        loop {
            match self.peek() {
                TokenKind::Eof => break,
                TokenKind::Semicolon if brace_depth == 0 => {
                    self.advance();
                    break;
                }
                TokenKind::LeftBrace => {
                    brace_depth += 1;
                    self.advance();
                }
                TokenKind::RightBrace => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                        self.advance();
                        if brace_depth == 0 {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Skip to the next rule (error recovery)
    fn skip_to_next_rule(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Eof => break,
                TokenKind::RightBrace => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Parse a single rule
    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = self.parse_selector_list()?;

        self.skip_whitespace();
        if !self.expect(&TokenKind::LeftBrace) {
            return None;
        }

        let declarations = self.parse_declarations();

        self.skip_whitespace();
        self.expect(&TokenKind::RightBrace);

        Some(Rule::new(selectors, declarations))
    }

    /// Parse a comma-separated list of selectors
    fn parse_selector_list(&mut self) -> Option<Vec<Selector>> {
        let mut selectors = Vec::new();

        loop {
            self.skip_whitespace();

            if let Some(selector) = self.parse_selector() {
                selectors.push(selector);
            } else if selectors.is_empty() {
                return None;
            } else {
                break;
            }

            self.skip_whitespace();

            if matches!(self.peek(), TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        if selectors.is_empty() {
            None
        } else {
            Some(selectors)
        }
    }

    /// Parse a single selector (chain of simple selectors)
    fn parse_selector(&mut self) -> Option<Selector> {
        let mut selector = Selector::new();

        loop {
            self.skip_whitespace();

            // Check for combinator
            let combinator = match self.peek() {
                TokenKind::GreaterThan => {
                    self.advance();
                    Some(Combinator::Child)
                }
                TokenKind::Plus => {
                    self.advance();
                    Some(Combinator::AdjacentSibling)
                }
                TokenKind::Tilde => {
                    self.advance();
                    Some(Combinator::GeneralSibling)
                }
                _ if !selector.simple_selectors.is_empty() => {
                    // Implicit descendant combinator if we already have selectors
                    Some(Combinator::Descendant)
                }
                _ => None,
            };

            self.skip_whitespace();

            // Check for end of selector
            match self.peek() {
                TokenKind::LeftBrace | TokenKind::Comma | TokenKind::Eof => break,
                _ => {}
            }

            // Parse simple selector
            if let Some(simple) = self.parse_simple_selector() {
                selector.add(combinator, simple);
            } else {
                break;
            }
        }

        if selector.simple_selectors.is_empty() {
            None
        } else {
            Some(selector)
        }
    }

    /// Parse a simple selector
    fn parse_simple_selector(&mut self) -> Option<SimpleSelector> {
        let mut simple = SimpleSelector::new();

        loop {
            match self.peek() {
                TokenKind::Asterisk => {
                    self.advance();
                    simple.universal = true;
                }
                TokenKind::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    simple.tag_name = Some(name.to_lowercase());
                }
                TokenKind::Hash(name) => {
                    let name = name.clone();
                    self.advance();
                    simple.id = Some(name);
                }
                TokenKind::Period => {
                    self.advance();
                    if let TokenKind::Ident(name) = self.peek().clone() {
                        self.advance();
                        simple.classes.push(name);
                    }
                }
                TokenKind::LeftBracket => {
                    if let Some(attr) = self.parse_attribute_selector() {
                        simple.attributes.push(attr);
                    }
                }
                TokenKind::Colon => {
                    self.advance();
                    // Check for :: (pseudo-element)
                    if matches!(self.peek(), TokenKind::Colon) {
                        self.advance();
                        if let TokenKind::Ident(name) = self.peek().clone() {
                            self.advance();
                            simple.pseudo_elements.push(name);
                        }
                    } else if let TokenKind::Ident(name) = self.peek().clone() {
                        self.advance();
                        // Handle pseudo-classes with functions like :nth-child(n)
                        if matches!(self.peek(), TokenKind::LeftParen) {
                            self.advance();
                            // Skip function arguments for now
                            let mut paren_depth = 1;
                            while paren_depth > 0 {
                                match self.advance() {
                                    TokenKind::LeftParen => paren_depth += 1,
                                    TokenKind::RightParen => paren_depth -= 1,
                                    TokenKind::Eof => break,
                                    _ => {}
                                }
                            }
                        }
                        simple.pseudo_classes.push(name);
                    }
                }
                _ => break,
            }
        }

        if simple.is_empty() {
            None
        } else {
            Some(simple)
        }
    }

    /// Parse an attribute selector
    fn parse_attribute_selector(&mut self) -> Option<AttributeSelector> {
        self.advance(); // Consume [

        self.skip_whitespace();

        let name = match self.peek().clone() {
            TokenKind::Ident(n) => {
                self.advance();
                n
            }
            _ => return None,
        };

        self.skip_whitespace();

        // Check for operator
        let op = match self.peek() {
            TokenKind::Equals => {
                self.advance();
                Some(AttributeOp::Exact)
            }
            TokenKind::Tilde => {
                self.advance();
                self.expect(&TokenKind::Equals);
                Some(AttributeOp::Includes)
            }
            TokenKind::Pipe => {
                self.advance();
                self.expect(&TokenKind::Equals);
                Some(AttributeOp::DashMatch)
            }
            TokenKind::Caret => {
                self.advance();
                self.expect(&TokenKind::Equals);
                Some(AttributeOp::Prefix)
            }
            TokenKind::Dollar => {
                self.advance();
                self.expect(&TokenKind::Equals);
                Some(AttributeOp::Suffix)
            }
            TokenKind::Asterisk => {
                self.advance();
                self.expect(&TokenKind::Equals);
                Some(AttributeOp::Substring)
            }
            _ => None,
        };

        let value = if op.is_some() {
            self.skip_whitespace();
            match self.peek().clone() {
                TokenKind::String(s) => {
                    self.advance();
                    Some(s)
                }
                TokenKind::Ident(s) => {
                    self.advance();
                    Some(s)
                }
                _ => None,
            }
        } else {
            None
        };

        self.skip_whitespace();

        // Check for case-insensitive flag
        let case_insensitive = if let TokenKind::Ident(flag) = self.peek() {
            if flag == "i" || flag == "I" {
                self.advance();
                true
            } else {
                false
            }
        } else {
            false
        };

        self.skip_whitespace();
        self.expect(&TokenKind::RightBracket);

        Some(AttributeSelector {
            name,
            op,
            value,
            case_insensitive,
        })
    }

    /// Parse declarations inside a rule block
    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        loop {
            self.skip_whitespace();

            match self.peek() {
                TokenKind::RightBrace | TokenKind::Eof => break,
                _ => {
                    if let Some(decl) = self.parse_declaration() {
                        declarations.push(decl);
                    } else {
                        // Skip to next declaration on error
                        self.skip_to_semicolon();
                    }
                }
            }
        }

        declarations
    }

    /// Skip to the next semicolon
    fn skip_to_semicolon(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Semicolon => {
                    self.advance();
                    break;
                }
                TokenKind::RightBrace | TokenKind::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Parse a single declaration
    fn parse_declaration(&mut self) -> Option<Declaration> {
        self.skip_whitespace();

        // Property name
        let property = match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.advance();
                name.to_lowercase()
            }
            _ => return None,
        };

        self.skip_whitespace();

        if !self.expect(&TokenKind::Colon) {
            return None;
        }

        self.skip_whitespace();

        // Parse value
        let (value, important) = self.parse_value();

        self.skip_whitespace();

        // Optional semicolon
        self.expect(&TokenKind::Semicolon);

        Some(Declaration::new(property, value, important))
    }

    /// Parse a declaration value
    fn parse_value(&mut self) -> (CssValue, bool) {
        let mut values: Vec<CssValue> = Vec::new();
        let mut important = false;

        loop {
            self.skip_whitespace();

            match self.peek() {
                TokenKind::Semicolon | TokenKind::RightBrace | TokenKind::Eof => break,
                TokenKind::Exclamation => {
                    self.advance();
                    self.skip_whitespace();
                    if let TokenKind::Ident(ident) = self.peek() {
                        if ident.to_lowercase() == "important" {
                            self.advance();
                            important = true;
                        }
                    }
                    break;
                }
                _ => {
                    if let Some(value) = self.parse_single_value() {
                        values.push(value);
                    } else {
                        break;
                    }
                }
            }
        }

        let value = match values.len() {
            0 => CssValue::Keyword("initial".to_string()),
            1 => values.remove(0),
            _ => CssValue::Multiple(values),
        };

        (value, important)
    }

    /// Parse a single value component
    fn parse_single_value(&mut self) -> Option<CssValue> {
        match self.peek().clone() {
            TokenKind::Number(n) => {
                self.advance();
                Some(CssValue::Number(n))
            }
            TokenKind::Dimension(n, unit) => {
                self.advance();
                if let Some(length_unit) = LengthUnit::parse(&unit) {
                    Some(CssValue::Length(Length::new(n, length_unit)))
                } else {
                    // Unknown unit, treat as raw
                    Some(CssValue::Raw(format!("{}{}", n, unit)))
                }
            }
            TokenKind::Percentage(n) => {
                self.advance();
                Some(CssValue::Percentage(n))
            }
            TokenKind::String(s) => {
                self.advance();
                Some(CssValue::String(s))
            }
            TokenKind::Hash(h) => {
                self.advance();
                if let Some(color) = Color::from_hex(&h) {
                    Some(CssValue::Color(color))
                } else {
                    Some(CssValue::Raw(format!("#{}", h)))
                }
            }
            TokenKind::Ident(ident) => {
                self.advance();
                // Check if it's a named color
                if let Some(color) = Color::from_name(&ident) {
                    Some(CssValue::Color(color))
                } else {
                    Some(CssValue::Keyword(ident.to_lowercase()))
                }
            }
            TokenKind::Function(name) => {
                self.advance();
                self.advance(); // Consume (
                let args = self.parse_function_args();
                self.expect(&TokenKind::RightParen);

                // Handle common functions
                match name.to_lowercase().as_str() {
                    "rgb" | "rgba" => {
                        if let Some(color) = self.parse_rgb_args(&args) {
                            Some(CssValue::Color(color))
                        } else {
                            Some(CssValue::Function(name, args))
                        }
                    }
                    "url" => {
                        if let Some(CssValue::String(url)) = args.first() {
                            Some(CssValue::Url(url.clone()))
                        } else if let Some(CssValue::Keyword(url)) = args.first() {
                            Some(CssValue::Url(url.clone()))
                        } else {
                            Some(CssValue::Function(name, args))
                        }
                    }
                    _ => Some(CssValue::Function(name, args)),
                }
            }
            TokenKind::Comma => {
                // Skip commas in value lists
                self.advance();
                self.parse_single_value()
            }
            _ => None,
        }
    }

    /// Parse function arguments
    fn parse_function_args(&mut self) -> Vec<CssValue> {
        let mut args = Vec::new();

        loop {
            self.skip_whitespace();

            match self.peek() {
                TokenKind::RightParen | TokenKind::Eof => break,
                TokenKind::Comma => {
                    self.advance();
                }
                _ => {
                    if let Some(value) = self.parse_single_value() {
                        args.push(value);
                    } else {
                        break;
                    }
                }
            }
        }

        args
    }

    /// Parse RGB/RGBA function arguments into a Color
    fn parse_rgb_args(&self, args: &[CssValue]) -> Option<Color> {
        // rgb(r, g, b) or rgba(r, g, b, a)
        let r = self.extract_color_component(args.first()?)?;
        let g = self.extract_color_component(args.get(1)?)?;
        let b = self.extract_color_component(args.get(2)?)?;

        let a = if let Some(alpha) = args.get(3) {
            match alpha {
                CssValue::Number(n) => *n,
                CssValue::Percentage(p) => p / 100.0,
                _ => 1.0,
            }
        } else {
            1.0
        };

        Some(Color::rgba(r, g, b, a))
    }

    /// Extract a color component (0-255) from a CSS value
    fn extract_color_component(&self, value: &CssValue) -> Option<u8> {
        match value {
            CssValue::Number(n) => Some(n.clamp(0.0, 255.0) as u8),
            CssValue::Percentage(p) => Some((p.clamp(0.0, 100.0) / 100.0 * 255.0) as u8),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        let stylesheet = parse("");
        assert!(stylesheet.rules.is_empty());
    }

    #[test]
    fn parse_simple_rule() {
        let stylesheet = parse("body { color: red; }");
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors.len(), 1);
        assert_eq!(stylesheet.rules[0].declarations.len(), 1);
    }

    #[test]
    fn parse_multiple_selectors() {
        let stylesheet = parse("h1, h2, h3 { font-weight: bold; }");
        assert_eq!(stylesheet.rules[0].selectors.len(), 3);
    }

    #[test]
    fn parse_multiple_declarations() {
        let stylesheet = parse("div { color: red; background: blue; }");
        assert_eq!(stylesheet.rules[0].declarations.len(), 2);
    }

    #[test]
    fn parse_class_selector() {
        let stylesheet = parse(".container { width: 100%; }");
        let sel = &stylesheet.rules[0].selectors[0];
        assert_eq!(
            sel.simple_selectors[0].classes,
            vec!["container".to_string()]
        );
    }

    #[test]
    fn parse_id_selector() {
        let stylesheet = parse("#main { padding: 10px; }");
        let sel = &stylesheet.rules[0].selectors[0];
        assert_eq!(sel.simple_selectors[0].id, Some("main".to_string()));
    }

    #[test]
    fn parse_descendant() {
        let stylesheet = parse("div p { color: red; }");
        let sel = &stylesheet.rules[0].selectors[0];
        assert_eq!(sel.simple_selectors.len(), 2);
        assert!(matches!(sel.combinators[0], Combinator::Descendant));
    }

    #[test]
    fn parse_child() {
        let stylesheet = parse("div > p { color: red; }");
        let sel = &stylesheet.rules[0].selectors[0];
        assert!(matches!(sel.combinators[0], Combinator::Child));
    }

    #[test]
    fn parse_adjacent_sibling() {
        let stylesheet = parse("div + p { color: red; }");
        let sel = &stylesheet.rules[0].selectors[0];
        assert!(matches!(sel.combinators[0], Combinator::AdjacentSibling));
    }

    #[test]
    fn parse_general_sibling() {
        let stylesheet = parse("div ~ p { color: red; }");
        let sel = &stylesheet.rules[0].selectors[0];
        assert!(matches!(sel.combinators[0], Combinator::GeneralSibling));
    }

    #[test]
    fn parse_important() {
        let stylesheet = parse("div { color: red !important; }");
        assert!(stylesheet.rules[0].declarations[0].important);
    }

    #[test]
    fn parse_rgb_function() {
        let stylesheet = parse("div { color: rgb(255, 0, 0); }");
        let decl = &stylesheet.rules[0].declarations[0];
        assert!(matches!(&decl.value, CssValue::Color(_)));
    }

    #[test]
    fn parse_hex_color() {
        let stylesheet = parse("div { color: #ff0000; }");
        let decl = &stylesheet.rules[0].declarations[0];
        assert!(matches!(&decl.value, CssValue::Color(_)));
    }

    #[test]
    fn parse_dimension() {
        let stylesheet = parse("div { width: 100px; }");
        let decl = &stylesheet.rules[0].declarations[0];
        assert!(matches!(&decl.value, CssValue::Length(_)));
    }

    #[test]
    fn parse_percentage() {
        let stylesheet = parse("div { width: 50%; }");
        let decl = &stylesheet.rules[0].declarations[0];
        assert!(matches!(&decl.value, CssValue::Percentage(50.0)));
    }

    #[test]
    fn skip_at_rules() {
        let stylesheet = parse("@import url('test.css'); body { color: red; }");
        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn parse_multiple_values() {
        let stylesheet = parse("div { margin: 10px 20px; }");
        let decl = &stylesheet.rules[0].declarations[0];
        assert!(matches!(&decl.value, CssValue::Multiple(_)));
    }

    #[test]
    fn parse_attribute_presence() {
        let stylesheet = parse("[disabled] { opacity: 0.5; }");
        let sel = &stylesheet.rules[0].selectors[0];
        assert_eq!(sel.simple_selectors[0].attributes.len(), 1);
    }

    #[test]
    fn parse_attribute_exact() {
        let stylesheet = parse(r#"[type="text"] { border: 1px; }"#);
        let sel = &stylesheet.rules[0].selectors[0];
        let attr = &sel.simple_selectors[0].attributes[0];
        assert_eq!(attr.name, "type");
        assert_eq!(attr.value, Some("text".to_string()));
    }

    #[test]
    fn parse_pseudo_class() {
        let stylesheet = parse("a:hover { color: blue; }");
        let sel = &stylesheet.rules[0].selectors[0];
        assert!(sel.simple_selectors[0]
            .pseudo_classes
            .contains(&"hover".to_string()));
    }

    #[test]
    fn parse_universal() {
        let stylesheet = parse("* { margin: 0; }");
        let sel = &stylesheet.rules[0].selectors[0];
        assert!(sel.simple_selectors[0].universal);
    }

    #[test]
    fn parse_combined_selector() {
        let stylesheet = parse("div.container#main { padding: 10px; }");
        let sel = &stylesheet.rules[0].selectors[0];
        let simple = &sel.simple_selectors[0];
        assert_eq!(simple.tag_name, Some("div".to_string()));
        assert_eq!(simple.classes, vec!["container".to_string()]);
        assert_eq!(simple.id, Some("main".to_string()));
    }

    #[test]
    fn parse_complex_css() {
        let css = r#"
            /* Reset */
            * { margin: 0; padding: 0; box-sizing: border-box; }

            body {
                font-family: sans-serif;
                line-height: 1.5;
                color: #333;
            }

            .container {
                max-width: 1200px;
                margin: 0 auto;
                padding: 0 20px;
            }

            header > nav ul li {
                display: inline-block;
            }

            a:hover {
                color: #0066cc;
            }

            @media screen and (max-width: 768px) {
                .container { padding: 0 10px; }
            }
        "#;

        let stylesheet = parse(css);
        // Should have parsed several rules (excluding @media content for now)
        assert!(stylesheet.rules.len() >= 4);
    }

    #[test]
    fn stylesheet_dump() {
        let stylesheet = parse("body { color: red; }");
        let dump = stylesheet.dump();
        assert!(dump.contains("Rule 0:"));
        assert!(dump.contains("Selectors:"));
        assert!(dump.contains("Declarations:"));
    }
}
